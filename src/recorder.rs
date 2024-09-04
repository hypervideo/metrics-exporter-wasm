#![allow(unused_variables)]

use metrics::{
    Counter, CounterFn, Gauge, GaugeFn, Histogram, HistogramFn, Key, KeyName, Metadata, Recorder,
    SetRecorderError, SharedString, Unit,
};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
#[cfg(target_pointer_width = "32")]
pub use portable_atomic::AtomicU64;
#[cfg(not(target_pointer_width = "32"))]
pub use std::sync::atomic::AtomicU64;

use crate::types::{Event, MetricOperation, MetricType};

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

struct State {
    client_count: AtomicU64,
    should_send: AtomicBool,
    tx: Sender<Event>,
}

impl State {
    pub fn new(tx: Sender<Event>) -> State {
        State {
            client_count: AtomicU64::new(0),
            should_send: AtomicBool::new(false),
            tx,
        }
    }

    pub fn should_send(&self) -> bool {
        self.should_send.load(Ordering::Acquire)
    }

    pub fn increment_clients(&self) {
        self.client_count.fetch_add(1, Ordering::AcqRel);
        self.should_send.store(true, Ordering::Release);
    }

    pub fn decrement_clients(&self) {
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
        let _ = self.tx.try_send(Event::Metadata {
            name: key_name,
            metric_type,
            unit,
            description,
        });
    }

    fn push_metric(&self, key: &Key, op: MetricOperation) {
        if self.should_send() {
            let _ = self.tx.try_send(Event::Metric {
                key: key.clone(),
                op,
            });
        }
    }
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

pub struct WasmRecorderBuilder {
    buffer_size: Option<usize>,
}

impl WasmRecorderBuilder {
    pub fn build(self) -> Result<WasmRecorder, SetRecorderError<WasmRecorder>> {
        let (tx, rx) = if let Some(size) = self.buffer_size {
            bounded(size)
        } else {
            unbounded()
        };

        // thread::spawn(move || run_transport(poll, listener, rx, state, buffer_size));

        Ok(WasmRecorder::with_sender(tx))
    }

    pub fn install(self) -> Result<(), SetRecorderError<WasmRecorder>> {
        let recorder = self.build()?;
        metrics::set_global_recorder(recorder)
    }

    pub fn buffer_size(mut self, size: Option<usize>) -> Self {
        self.buffer_size = size;
        self
    }
}

pub struct WasmRecorder {
    state: Arc<State>,
    handle: Option<Handle>,
}

impl WasmRecorder {
    pub fn builder() -> WasmRecorderBuilder {
        WasmRecorderBuilder { buffer_size: None }
    }

    pub fn with_sender(tx: Sender<Event>) -> Self {
        Self {
            state: Arc::new(State::new(tx)),
            handle: None,
        }
    }
}

impl Recorder for WasmRecorder {
    fn describe_counter(&self, key: KeyName, unit: Option<Unit>, description: SharedString) {
        self.state
            .register_metric(key, MetricType::Counter, unit, description);
    }

    fn describe_gauge(&self, key: KeyName, unit: Option<Unit>, description: SharedString) {
        self.state
            .register_metric(key, MetricType::Gauge, unit, description);
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
        self.state
            .push_metric(&self.key, MetricOperation::SetCounter(value))
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
        self.state
            .push_metric(&self.key, MetricOperation::SetGauge(value))
    }
}

impl HistogramFn for Handle {
    fn record(&self, value: f64) {
        self.state
            .push_metric(&self.key, MetricOperation::RecordHistogram(value))
    }
}
