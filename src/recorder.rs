use crate::{
    Event,
    Events,
    MetricOperation,
    MetricType,
};
use metrics::{
    Counter,
    CounterFn,
    Gauge,
    GaugeFn,
    Histogram,
    HistogramFn,
    Key,
    KeyName,
    Metadata,
    Recorder,
    SetRecorderError,
    SharedString,
    Unit,
};
#[cfg(target_pointer_width = "32")]
pub use portable_atomic::AtomicU64;
#[cfg(not(target_pointer_width = "32"))]
pub use std::sync::atomic::AtomicU64;
use std::{
    sync::{
        atomic::{
            AtomicBool,
            Ordering,
        },
        Arc,
    },
    time::Duration,
};
use tokio::sync::mpsc::{
    channel,
    Receiver,
    Sender,
};

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

/// The state of the metrics recorder. Tracks the number of clients and holds
/// the sender for sending new metrics to the async processing handler. That
/// handler will forward the metrics to other subscribers inside the app itself
/// or send the metrics to a remote server.
struct State {
    client_count: AtomicU64,
    should_send: AtomicBool,
    tx: Sender<Event>,
}

impl State {
    fn new(tx: Sender<Event>) -> State {
        State {
            client_count: AtomicU64::new(0),
            should_send: AtomicBool::new(false),
            tx,
        }
    }

    fn should_send(&self) -> bool {
        self.should_send.load(Ordering::Acquire)
    }

    fn increment_clients(&self) {
        self.client_count.fetch_add(1, Ordering::AcqRel);
        self.should_send.store(true, Ordering::Release);
    }

    fn decrement_clients(&self) {
        let count = self.client_count.fetch_sub(1, Ordering::AcqRel);
        if count == 1 {
            self.should_send.store(false, Ordering::Release);
        }
    }

    fn register_metric(
        &self,
        key_name: KeyName,
        metric_type: MetricType,
        unit: Option<Unit>,
        description: SharedString,
    ) {
        trace!(?key_name, ?metric_type, ?unit, ?description, "registering metric");
        let tx = self.tx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = tx
                .send(Event::Metadata {
                    name: key_name,
                    metric_type,
                    unit,
                    description,
                })
                .await;
        });
    }

    fn push_metric(&self, key: &Key, op: MetricOperation) {
        trace!(?key, ?op, should_send = %self.should_send(), "pushing metric");
        let tx = self.tx.clone();
        let key = key.clone();
        if self.should_send() {
            wasm_bindgen_futures::spawn_local(async move {
                let _ = tx.send(Event::Metric { key: key.clone(), op }).await;
            });
        }
    }
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

pub struct EndpointUndefined;
pub struct EndpointDefined(String);

/// A builder for a [`WasmRecorder`].
pub struct WasmRecorderBuilder<T> {
    buffer_size: Option<usize>,
    endpoint: T,
}

impl WasmRecorderBuilder<EndpointUndefined> {
    /// Set the buffer size for the metrics transport.
    pub fn buffer_size(mut self, size: Option<usize>) -> Self {
        self.buffer_size = size;
        self
    }

    /// Set the endpoint for the metrics transport.
    pub fn endpoint(self, endpoint: impl ToString) -> WasmRecorderBuilder<EndpointDefined> {
        WasmRecorderBuilder {
            buffer_size: self.buffer_size,
            endpoint: EndpointDefined(endpoint.to_string()),
        }
    }
}

impl WasmRecorderBuilder<EndpointDefined> {
    /// Set the buffer size for the metrics transport.
    pub fn buffer_size(mut self, size: Option<usize>) -> Self {
        self.buffer_size = size;
        self
    }

    /// Create a new builder for a [`WasmRecorder`].
    pub fn build(self) -> Result<WasmRecorder, SetRecorderError<WasmRecorder>> {
        let Self {
            buffer_size: _,
            endpoint: EndpointDefined(endpoint),
        } = self;

        let (tx, rx) = if let Some(size) = self.buffer_size {
            channel(size)
        } else {
            channel(42)
        };

        let state = Arc::new(State::new(tx.clone()));

        wasm_bindgen_futures::spawn_local({
            let state = state.clone();
            let buffer_size = self.buffer_size;
            async move {
                run_transport(rx, state, endpoint, buffer_size).await;
            }
        });

        Ok(WasmRecorder { state })
    }

    /// Install this recorder as the global recorder.
    pub fn install(self) -> Result<(), SetRecorderError<WasmRecorder>> {
        self.build()?.install()
    }
}

/// A metrics recorder for use in WASM environments.
pub struct WasmRecorder {
    state: Arc<State>,
}

impl WasmRecorder {
    /// Create a new builder for a [`WasmRecorder`].
    pub fn builder() -> WasmRecorderBuilder<EndpointUndefined> {
        WasmRecorderBuilder {
            buffer_size: None,
            endpoint: EndpointUndefined,
        }
    }

    /// Install this recorder as the global recorder.
    pub fn install(self) -> Result<(), SetRecorderError<Self>> {
        metrics::set_global_recorder(self)
    }
}

impl Recorder for WasmRecorder {
    fn describe_counter(&self, key: KeyName, unit: Option<Unit>, description: SharedString) {
        self.state.register_metric(key, MetricType::Counter, unit, description);
    }

    fn describe_gauge(&self, key: KeyName, unit: Option<Unit>, description: SharedString) {
        self.state.register_metric(key, MetricType::Gauge, unit, description);
    }

    fn describe_histogram(&self, key: KeyName, unit: Option<Unit>, description: SharedString) {
        self.state
            .register_metric(key, MetricType::Histogram, unit, description);
    }

    fn register_counter(&self, key: &Key, _metadata: &Metadata<'_>) -> Counter {
        Counter::from_arc(Arc::new(Handle::new(key.clone(), self.state.clone())))
    }

    fn register_gauge(&self, key: &Key, _metadata: &Metadata<'_>) -> Gauge {
        Gauge::from_arc(Arc::new(Handle::new(key.clone(), self.state.clone())))
    }

    fn register_histogram(&self, key: &Key, _metadata: &Metadata<'_>) -> Histogram {
        Histogram::from_arc(Arc::new(Handle::new(key.clone(), self.state.clone())))
    }
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

struct Handle {
    key: Key,
    state: Arc<State>,
}

impl Handle {
    fn new(key: Key, state: Arc<State>) -> Handle {
        Handle { key, state }
    }
}

impl CounterFn for Handle {
    fn increment(&self, value: u64) {
        self.state
            .push_metric(&self.key, MetricOperation::IncrementCounter(value))
    }

    fn absolute(&self, value: u64) {
        self.state.push_metric(&self.key, MetricOperation::SetCounter(value))
    }
}

impl GaugeFn for Handle {
    fn increment(&self, value: f64) {
        self.state
            .push_metric(&self.key, MetricOperation::IncrementGauge(value))
    }

    fn decrement(&self, value: f64) {
        self.state
            .push_metric(&self.key, MetricOperation::DecrementGauge(value))
    }

    fn set(&self, value: f64) {
        self.state.push_metric(&self.key, MetricOperation::SetGauge(value))
    }
}

impl HistogramFn for Handle {
    fn record(&self, value: f64) {
        self.state
            .push_metric(&self.key, MetricOperation::RecordHistogram(value))
    }
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

async fn run_transport(mut rx: Receiver<Event>, state: Arc<State>, endpoint: String, _buffer_size: Option<usize>) {
    debug!("starting metrics transport");

    state.increment_clients();

    loop {
        let Some(event) = rx.recv().await else {
            debug!("metrics transport lost all senders");
            break;
        };

        info!(?event, "received metrics event");

        if let Err(err) = post_metrics(Duration::from_secs(5), vec![event].into(), &endpoint).await {
            error!(?err, "failed to send metrics");
        }
    }

    state.decrement_clients();

    debug!("metrics transport stopped");
}

async fn post_metrics(timeout: Duration, events: Events, endpoint: &str) -> std::io::Result<()> {
    use gloo::net::http::{
        Method,
        RequestBuilder,
    };
    use web_sys::AbortController;

    let controller = AbortController::new().unwrap();
    let signal = controller.signal();

    let body = events
        .serialize_with_asn1rs()
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;

    let req = RequestBuilder::new(endpoint)
        .method(Method::POST)
        .header("content-type", "application/octet-stream")
        .abort_signal(Some(&signal))
        .body(body)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;

    let fut = async {
        let res = req
            .send()
            .await
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

        if !res.ok() {
            let text = res.text().await.map_err(|err| err.to_string()).unwrap_or_default();
            let status = res.status();
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to fetch server info. status={status} {text}"),
            ));
        };

        Ok(())
    };

    let fut = std::pin::pin!(fut);
    match futures::future::select(fut, gloo::timers::future::sleep(timeout)).await {
        futures::future::Either::Left((res, _)) => res,
        futures::future::Either::Right(_) => {
            controller.abort();
            Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "Timed out"))
        }
    }
}
