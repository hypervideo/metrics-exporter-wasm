use crate::{
    Event,
    MetricOperation,
    MetricType,
    WasmRecorder,
};
use metrics::{
    Key,
    KeyName,
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

/// A builder for a [`WasmRecorder`].
pub struct MetricsHttpSender<T> {
    max_chunk_size: Option<usize>,
    send_frequency: Duration,
    compression: Option<Compression>,
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

    /// Set the endpoint for the metrics transport.
    pub fn endpoint(self, endpoint: impl ToString) -> MetricsHttpSender<EndpointDefined> {
        MetricsHttpSender {
            max_chunk_size: self.max_chunk_size,
            send_frequency: self.send_frequency,
            compression: self.compression,
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

    /// Start sending metrics to the endpoint specified. Returns a guard that will stop the transport when dropped.
    pub fn start_with(self, recorder: &WasmRecorder) -> DropGuard {
        let Self {
            max_chunk_size: buffer_size,
            send_frequency,
            compression,
            endpoint: EndpointDefined(endpoint),
        } = self;

        let rx = recorder.subscribe();
        let token = CancellationToken::new();

        wasm_bindgen_futures::spawn_local({
            let token = token.clone();
            async move {
                run_transport(rx, token, endpoint, buffer_size, send_frequency, compression).await;
            }
        });

        token.drop_guard()
    }
}

async fn run_transport(
    mut rx: broadcast::Receiver<Event>,
    token: CancellationToken,
    endpoint: String,
    buffer_size: Option<usize>,
    send_frequency: Duration,
    compression: Option<Compression>,
) {
    use backoff::backoff::Backoff as _;

    debug!(%endpoint, "starting metrics transport");
    defer! {
        debug!(%endpoint, "metrics transport stopped");
    }

    // Initial connection, send internal metadata
    {
        let mut backoff = backoff::ExponentialBackoff::default();

        let metrics_processed = RecordedEvents::from(vec![Event::Description {
            name: KeyName::from_const_str("metrics_processed"),
            metric_type: MetricType::Counter,
            unit: None,
            description: "metrics-exporter-wasm internal counter".into(),
        }]);
        while let Err(err) = post_metrics(Duration::from_secs(5), &metrics_processed, &endpoint, compression).await {
            error!(?err, "failed to send initial metadata");
            if let Some(backoff) = backoff.next_backoff() {
                sleep(backoff).await;
            }
        }
    }

    // Time-batched metrics transport
    let mut batch_start_time = util_time::now();
    let mut time_to_send: Option<wasmtimer::tokio::Sleep> = None;
    let mut events = VecDeque::new();
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
                let n = events.len();
                trace!(%n, "sending metrics");
                time_to_send = None;
                events.push_back(RecordedEvent::from(Event::Metric { key: Key::from_static_name("metrics_processed"), op: MetricOperation::IncrementCounter(events.len() as _) }));

                let mut backoff = backoff::ExponentialBackoffBuilder::new()
                    .with_max_elapsed_time(Some(Duration::from_secs(60)))
                    .build();
                let recorded_events = RecordedEvents::new(batch_start_time, events.drain(..).collect());
                loop {
                    match post_metrics(Duration::from_secs(5), &recorded_events, &endpoint, compression).await {
                        Ok(_) => {
                            batch_start_time = util_time::now();
                            break
                        },
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

            event = rx.recv() => {
                match event {
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    },

                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        warn!(?endpoint, "metrics transport lagged");
                    },

                    Ok(event) => {
                        if buffer_size.is_some_and(|buffer_size| events.len() >= buffer_size) {
                            if last_warning.is_none_or(|last_warning| last_warning.elapsed() >= Duration::from_secs(5)) {
                                warn!("metrics chunk size exceeded, dropping metrics");
                                last_warning = Some(Instant::now());
                            }
                            events.pop_front();
                        };
                        events.push_back(RecordedEvent::from(event));
                        if time_to_send.is_none() {
                            time_to_send = Some(sleep(send_frequency));
                        }
                    },
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
