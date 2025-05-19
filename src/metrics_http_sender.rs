use crate::{
    Event,
    Transport,
    WasmRecorder,
};
use backon::{
    ExponentialBuilder,
    Retryable,
};
use metrics_exporter_wasm_core::{
    util_time,
    Asn1Encode,
    RecordedEvent,
    RecordedEvents,
};
use std::{
    collections::VecDeque,
    time::Duration,
};
use tokio::sync::broadcast;
use tokio_util::sync::{
    CancellationToken,
    DropGuard,
};
use wasmtimer::{
    std::Instant,
    tokio::sleep,
};

/// A generic batch to represent the data that gets accumulated and then sent using the [MetricsHttpSender].
pub trait Batch {
    type Item: Clone + 'static;
    type CompletedBatch: Asn1Encode;

    fn new() -> Self;
    fn pop_front(&mut self) -> Option<Self::Item>;
    fn push_back(&mut self, item: Self::Item);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn finalize(&mut self) -> Self::CompletedBatch;
}

struct BatchedEvents {
    batch_start_time: chrono::DateTime<chrono::Utc>,
    events: VecDeque<RecordedEvent>,
}

impl Batch for BatchedEvents {
    type Item = Event;

    type CompletedBatch = RecordedEvents;

    fn new() -> Self {
        Self {
            batch_start_time: util_time::now(),
            events: Default::default(),
        }
    }

    fn pop_front(&mut self) -> Option<Self::Item> {
        self.events.pop_front().map(Into::into)
    }

    fn push_back(&mut self, item: Self::Item) {
        self.events.push_back(RecordedEvent::from(item));
    }

    fn len(&self) -> usize {
        self.events.len()
    }

    fn finalize(&mut self) -> Self::CompletedBatch {
        let start_time = self.batch_start_time;
        let events = self.events.drain(..).collect();
        self.batch_start_time = util_time::now();
        RecordedEvents::new(start_time, events)
    }
}

/// A metrics exporter for a [WasmRecorder].
///
/// The payload that gets send is actually generic, see the [Batch] trait and [Self::start_with_receiver] method.
pub struct MetricsHttpSender<T> {
    max_chunk_size: Option<usize>,
    send_frequency: Duration,
    self_metrics: bool,
    transport: T,
}

impl<T> MetricsHttpSender<T> {
    pub fn new(transport: T) -> Self {
        Self {
            max_chunk_size: None,
            send_frequency: Duration::from_secs(15),
            self_metrics: false,
            transport,
        }
    }

    /// How many metrics events to maximally send in one request.
    pub fn max_chunk_size(mut self, size: Option<usize>) -> Self {
        self.max_chunk_size = size;
        self
    }

    /// Set the frequency at which metrics are sent to the transport.
    pub fn send_frequency(mut self, frequency: Duration) -> Self {
        self.send_frequency = frequency;
        self
    }

    /// Set whether to emit internal metrics.
    ///
    /// Currently this is a `metrics_processed` counter that counts how many metrics events (or other batch items) have
    /// been sent.
    pub fn self_metrics(mut self, self_metrics: bool) -> Self {
        self.self_metrics = self_metrics;
        self
    }
}

impl<T: Transport + Send + 'static> MetricsHttpSender<T> {
    /// Start sending metrics to the endpoint specified.
    ///
    /// Returns a guard that will stop the transport when dropped.
    pub fn start_with(self, recorder: &WasmRecorder) -> DropGuard {
        self.start_with_receiver::<BatchedEvents>(recorder.subscribe())
    }

    /// If you want send more data than just metrics event, this generic method allow you to provide a custom [Batch]
    /// implementation and channel for [Batch::Item]s.
    pub fn start_with_receiver<B: Batch>(self, rx: broadcast::Receiver<B::Item>) -> DropGuard {
        let token = CancellationToken::new();

        wasm_bindgen_futures::spawn_local({
            let token = token.clone();
            async move {
                self.run_transport::<B>(rx, token).await;
            }
        });

        token.drop_guard()
    }

    async fn run_transport<B: Batch>(self, mut rx: broadcast::Receiver<B::Item>, token: CancellationToken) {
        let Self {
            max_chunk_size: buffer_size,
            send_frequency,
            self_metrics,
            transport,
        } = self;

        debug!("starting metrics transport");
        defer! {
            debug!("metrics transport stopped");
        }

        // Initial connection, send internal metadata
        let metrics_processed_counter = if self_metrics {
            metrics::describe_counter!(
                "metrics_processed",
                metrics::Unit::Count,
                "metrics-exporter-wasm internal counter that counts how many events where processed."
            );
            metrics::describe_histogram!(
                "metrics_exporter_compressed_payload_size",
                metrics::Unit::Bytes,
                "metrics-exporter-wasm internal. Compressed payload size in bytes."
            );
            Some(metrics::counter!("metrics_processed"))
        } else {
            None
        };

        // Time-batched metrics transport
        let mut time_to_send: Option<wasmtimer::tokio::Sleep> = None;
        let mut batch = B::new();
        let mut last_warning = None::<Instant>;

        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    break;
                }

                _ = async {
                    if let Some(time_to_send) = &mut time_to_send {
                        time_to_send.await;
                    } else {
                        std::future::pending::<()>().await;
                    }

                } => {
                    let n = batch.len();
                    trace!(%n, "sending metrics");
                    time_to_send = None;

                    let completed_batch = batch.finalize();
                    let post = || async { transport.send(&completed_batch).await };
                    match post
                        .retry(
                            ExponentialBuilder::new()
                                .with_max_times(5)
                                .with_factor(2.0)
                                .with_min_delay(Duration::from_secs(1))
                                .with_max_delay(Duration::from_secs(60))
                                .with_total_delay(Some(Duration::from_secs(3 * 60))),
                        )
                        .notify(|err: &std::io::Error, dur: Duration| {
                            warn!(?err, "failed to send metrics, retrying in {dur:?}: {err}");
                        })
                        .await
                    {
                        Ok(_) => {
                            if let Some(metrics_processed_counter) = &metrics_processed_counter {
                                metrics_processed_counter.increment(n as _);
                            }
                            trace!(%n, "metrics send");
                        }
                        Err(err) => {
                            error!(?err, "failed to send metrics, giving up and loosing {n} metrics");
                        }
                    }
                }

                event = rx.recv() => {
                    match event {
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        },

                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            warn!("metrics transport lagged");
                        },

                        Ok(event) => {
                            if buffer_size.is_some_and(|buffer_size| batch.len() >= buffer_size) {
                                if last_warning.is_none_or(|last_warning| last_warning.elapsed() >= Duration::from_secs(5)) {
                                    warn!("metrics chunk size exceeded, dropping metrics");
                                    last_warning = Some(Instant::now());
                                }
                                batch.pop_front();
                            };
                            batch.push_back(event);
                            if time_to_send.is_none() {
                                time_to_send = Some(sleep(send_frequency));
                            }
                        },
                    }
                }
            }
        }
    }
}
