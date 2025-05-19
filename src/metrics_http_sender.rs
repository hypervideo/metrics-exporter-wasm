use crate::{
    Event,
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

#[doc(hidden)]
pub struct EndpointUndefined;
#[doc(hidden)]
pub struct EndpointDefined(String);

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
    compression: Option<Compression>,
    self_metrics: bool,
    endpoint: T,
}

#[derive(Debug, Clone, Copy)]
pub enum Compression {
    #[cfg(feature = "compress-zstd-external")]
    Zstd { level: u8 },
    #[cfg(feature = "compress-brotli")]
    Brotli,
}

impl Default for MetricsHttpSender<EndpointUndefined> {
    fn default() -> Self {
        Self {
            max_chunk_size: None,
            send_frequency: Duration::from_secs(15),
            compression: None,
            self_metrics: false,
            endpoint: EndpointUndefined,
        }
    }
}

impl MetricsHttpSender<EndpointUndefined> {
    /// Create a new metrics sender.
    pub fn new() -> Self {
        Default::default()
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

    /// Set the compression algorithm to use.
    pub fn compression(mut self, compression: Option<Compression>) -> Self {
        self.compression = compression;
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

    /// Set the endpoint for the metrics transport.
    pub fn endpoint(self, endpoint: impl ToString) -> MetricsHttpSender<EndpointDefined> {
        MetricsHttpSender {
            max_chunk_size: self.max_chunk_size,
            send_frequency: self.send_frequency,
            compression: self.compression,
            self_metrics: self.self_metrics,
            endpoint: EndpointDefined(endpoint.to_string()),
        }
    }
}

impl MetricsHttpSender<EndpointDefined> {
    /// Set the buffer size for the metrics transport.
    pub fn buffer_size(mut self, size: Option<usize>) -> Self {
        self.max_chunk_size = size;
        self
    }

    /// Set the frequency at which metrics are sent to the transport.
    pub fn send_frequency(mut self, frequency: Duration) -> Self {
        self.send_frequency = frequency;
        self
    }

    /// Set the compression algorithm to use.
    pub fn compression(mut self, compression: Option<Compression>) -> Self {
        self.compression = compression;
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
            compression,
            self_metrics,
            endpoint: EndpointDefined(endpoint),
        } = self;

        debug!(%endpoint, "starting metrics transport");
        defer! {
            debug!(%endpoint, "metrics transport stopped");
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
                    let post = || async { post_metrics(Duration::from_secs(5), &completed_batch, &endpoint, compression, self.self_metrics).await };
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
                            warn!(?endpoint, "metrics transport lagged");
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

async fn post_metrics(
    timeout: Duration,
    events: &impl Asn1Encode,
    endpoint: &str,
    compression: Option<Compression>,
    self_metrics: bool,
) -> std::io::Result<()> {
    use gloo::net::http::{
        Headers,
        Method,
        RequestBuilder,
    };
    use web_sys::AbortController;

    fn err(err: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, err)
    }

    let controller = AbortController::new().unwrap();
    let signal = controller.signal();

    let headers = Headers::new();
    headers.set("content-type", "application/octet-stream");

    let body = match compression {
        #[cfg(feature = "compress-zstd-external")]
        Some(Compression::Zstd { level }) => {
            headers.set("content-encoding", "zstd");
            events.encode_and_compress_zstd_external(level)?
        }
        #[cfg(feature = "compress-brotli")]
        Some(Compression::Brotli) => {
            headers.set("content-encoding", "br");
            events.encode_and_compress_br()?
        }
        None => events.encode().map_err(err)?,
    };

    let body_size = body.len();

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

        if self_metrics {
            metrics::histogram!("metrics_exporter_compressed_payload_size").record(body_size as f64);
        }

        Ok(())
    };

    tokio::select! {
        biased;
        res = fut => res,
        _ = wasmtimer::tokio::sleep(timeout) => {
            controller.abort();
            Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "Timed out"))
        }
    }
}
