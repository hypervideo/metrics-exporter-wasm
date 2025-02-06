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
    collections::VecDeque,
    sync::{
        atomic::{
            AtomicBool,
            Ordering,
        },
        Arc,
    },
    time::Duration,
};
use tokio::sync::mpsc;
use wasmtimer::{
    std::Instant,
    tokio::sleep,
};

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

/// The state of the metrics recorder. Tracks the number of clients and holds
/// the sender for sending new metrics to the async processing handler. That
/// handler will forward the metrics to other subscribers inside the app itself
/// or send the metrics to a remote server.
struct State {
    client_count: AtomicU64,
    should_send: AtomicBool,
    tx: mpsc::UnboundedSender<Event>,
}

impl State {
    fn new(tx: mpsc::UnboundedSender<Event>) -> State {
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
        let _ = tx.send(Event::Metadata {
            name: key_name,
            metric_type,
            unit,
            description,
        });
    }

    fn push_metric(&self, key: &Key, op: MetricOperation) {
        trace!(?key, ?op, should_send = %self.should_send(), "pushing metric");
        let tx = self.tx.clone();
        let key = key.clone();
        if self.should_send() {
            let _ = tx.send(Event::Metric { key: key.clone(), op });
        }
    }
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

pub struct EndpointUndefined;
pub struct EndpointDefined(String);

/// A builder for a [`WasmRecorder`].
pub struct WasmRecorderBuilder<T> {
    buffer_size: Option<usize>,
    send_frequency: Duration,
    endpoint: T,
}

impl WasmRecorderBuilder<EndpointUndefined> {
    /// Set the buffer size for the metrics transport.
    pub fn buffer_size(mut self, size: Option<usize>) -> Self {
        self.buffer_size = size;
        self
    }

    /// Set the frequency at which metrics are sent to the transport.
    pub fn send_frequency(mut self, frequency: Duration) -> Self {
        self.send_frequency = frequency;
        self
    }

    /// Set the endpoint for the metrics transport.
    pub fn endpoint(self, endpoint: impl ToString) -> WasmRecorderBuilder<EndpointDefined> {
        WasmRecorderBuilder {
            buffer_size: self.buffer_size,
            send_frequency: self.send_frequency,
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

    /// Set the frequency at which metrics are sent to the transport.
    pub fn send_frequency(mut self, frequency: Duration) -> Self {
        self.send_frequency = frequency;
        self
    }

    /// Create a new builder for a [`WasmRecorder`].
    pub fn build(self) -> Result<WasmRecorder, SetRecorderError<WasmRecorder>> {
        let Self {
            buffer_size,
            send_frequency,
            endpoint: EndpointDefined(endpoint),
        } = self;

        let (tx, rx) = mpsc::unbounded_channel();

        let state = Arc::new(State::new(tx.clone()));

        wasm_bindgen_futures::spawn_local({
            let state = state.clone();
            async move {
                run_transport(rx, state, endpoint, buffer_size, send_frequency).await;
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
            send_frequency: Duration::from_secs(15),
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

async fn run_transport(
    mut rx: mpsc::UnboundedReceiver<Event>,
    state: Arc<State>,
    endpoint: String,
    buffer_size: Option<usize>,
    send_frequency: Duration,
) {
    use backoff::backoff::Backoff as _;

    debug!("starting metrics transport");

    state.increment_clients();
    defer! {
        state.decrement_clients();
    }

    // Initial connection, send internal metadata
    {
        let mut backoff = backoff::ExponentialBackoff::default();
        while let Err(err) = post_metrics(
            Duration::from_secs(5),
            &vec![Event::Metadata {
                name: KeyName::from_const_str("metrics_processed"),
                metric_type: MetricType::Counter,
                unit: None,
                description: "metrics-exporter-wasm internal counter".into(),
            }]
            .into(),
            &endpoint,
        )
        .await
        {
            error!(?err, "failed to send initial metadata");
            if let Some(backoff) = backoff.next_backoff() {
                sleep(backoff).await;
            }
        }
    }

    // Time-batched metrics transport
    let mut time_to_send: Option<wasmtimer::tokio::Sleep> = None;
    let mut events = VecDeque::new();
    let mut last_warning = None::<Instant>;

    loop {
        tokio::select! {
            _ = async {
                if let Some(time_to_send) = &mut time_to_send {
                    time_to_send.await;
                } else {
                    std::future::pending::<()>().await;
                }

            } => {
                let n = events.len();
                trace!(%n, "sending metrics");
                time_to_send = None;
                events.push_back(Event::Metric { key: Key::from_static_name("metrics_processed"), op: MetricOperation::IncrementCounter(events.len() as _) });

                let mut backoff = backoff::ExponentialBackoffBuilder::new()
                    .with_max_elapsed_time(Some(Duration::from_secs(60)))
                    .build();
                let events: Events = events.drain(..).collect::<Vec::<_>>().into();
                loop {
                    match post_metrics(Duration::from_secs(5), &events, &endpoint).await {
                        Ok(_) => break,
                        Err(err) => {
                            if let Some(backoff) = backoff.next_backoff() {
                                warn!(?err, "failed to send metrics, retrying in {backoff:?}");
                                sleep(backoff).await;
                            } else {
                                error!(?err, "failed to send metrics, giving up and loosing {n} metrics");
                                break;
                            }
                        }
                    }
                }
            }

            Some(event) = rx.recv() => {
                if buffer_size.is_some_and(|buffer_size| events.len() >= buffer_size) {
                    if last_warning.is_none_or(|last_warning| last_warning.elapsed() >= Duration::from_secs(5)) {
                        warn!("metrics buffer size exceeded, dropping metrics");
                        last_warning = Some(Instant::now());
                    }
                    events.pop_front();
                };
                events.push_back(event);
                if time_to_send.is_none() {
                    time_to_send = Some(sleep(send_frequency));
                }
            }
        }
    }
}

async fn post_metrics(timeout: Duration, events: &Events, endpoint: &str) -> std::io::Result<()> {
    use gloo::net::http::{
        Headers,
        Method,
        RequestBuilder,
    };
    use std::io::Write as _;
    use web_sys::AbortController;

    fn err(err: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, err)
    }

    let controller = AbortController::new().unwrap();
    let signal = controller.signal();

    let body = events.serialize_with_asn1rs().map_err(err)?;
    let headers = Headers::new();
    headers.set("content-type", "application/octet-stream");

    const COMPRESS: bool = true;
    let body = if COMPRESS {
        headers.set("Content-Encoding", "br");
        let mut compressed = Vec::new();
        {
            let mut writer = brotli::CompressorWriter::new(&mut compressed, 4096, 11, 22);
            writer.write_all(&body).map_err(err)?;
        }
        compressed
    } else {
        body
    };

    let req = RequestBuilder::new(endpoint)
        .method(Method::POST)
        .headers(headers)
        .abort_signal(Some(&signal))
        .body(body)
        .map_err(err)?;

    let fut = async {
        let res = req.send().await.map_err(err)?;
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

    tokio::select! {
        res = fut => res,
        _ = wasmtimer::tokio::sleep(timeout) => {
            controller.abort();
            Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "Timed out"))
        }
    }
}
