use crate::{
    Event,
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
use std::sync::Arc;
use tokio::sync::broadcast;

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

/// The state of the metrics recorder. Tracks the number of clients and holds
/// the sender for sending new metrics to the async processing handler. That
/// handler will forward the metrics to other subscribers inside the app itself
/// or send the metrics to a remote server.
struct State {
    tx: broadcast::Sender<Event>,
}

impl State {
    fn new(tx: broadcast::Sender<Event>) -> State {
        State { tx }
    }

    fn should_send(&self) -> bool {
        self.tx.receiver_count() > 0
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
        let _ = tx.send(Event::Description {
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

/// A builder for a [`WasmRecorder`].
pub struct WasmRecorderBuilder {
    buffer_size: usize,
}

impl WasmRecorderBuilder {
    /// Set the buffer size for the metrics transport.
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Create a new builder for a [`WasmRecorder`].
    pub fn build(self) -> Result<WasmRecorder, SetRecorderError<WasmRecorder>> {
        let Self { buffer_size } = self;

        let (tx, _) = broadcast::channel(buffer_size);

        Ok(WasmRecorder {
            state: Arc::new(State::new(tx)),
        })
    }

    /// Install this recorder as the global recorder.
    pub fn install(self) -> Result<(), SetRecorderError<WasmRecorder>> {
        self.build()?.install()
    }
}

/// A metrics recorder for use in WASM environments.
#[derive(Clone)]
pub struct WasmRecorder {
    state: Arc<State>,
}

static GLOBAL_RECORDER: std::sync::LazyLock<std::sync::Mutex<Option<WasmRecorder>>> =
    std::sync::LazyLock::new(Default::default);

impl WasmRecorder {
    /// Create a new builder for a [`WasmRecorder`].
    pub fn builder() -> WasmRecorderBuilder {
        WasmRecorderBuilder { buffer_size: 1024 }
    }

    /// Subscribe to metrics events.
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.state.tx.subscribe()
    }

    /// Install this recorder as the global recorder.
    pub fn install(self) -> Result<(), SetRecorderError<Self>> {
        GLOBAL_RECORDER
            .lock()
            .expect("global recorder lock")
            .replace(self.clone());
        metrics::set_global_recorder(self)
    }

    pub fn global() -> Option<Self> {
        GLOBAL_RECORDER.lock().expect("global recorder lock").clone()
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
