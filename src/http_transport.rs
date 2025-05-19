use gloo::net::http::{
    Headers,
    Method,
    RequestBuilder,
};
use metrics_exporter_wasm_core::Asn1Encode;
use std::{
    future::Future,
    io,
    time::Duration,
};
use web_sys::AbortController;

#[derive(Debug, Clone, Copy)]
pub enum Compression {
    #[cfg(feature = "compress-zstd-external")]
    Zstd { level: u8 },
    #[cfg(feature = "compress-brotli")]
    Brotli,
}

pub trait Transport {
    fn send(&self, payload: &impl Asn1Encode) -> impl Future<Output = io::Result<()>>;
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

#[doc(hidden)]
pub struct EndpointUndefined;
#[doc(hidden)]
pub struct EndpointDefined(String);

#[derive(Default, Debug)]
pub struct HttpPostTransport<T> {
    timeout: Duration,
    compression: Option<Compression>,
    self_metrics: bool,
    endpoint: T,
}

impl Default for HttpPostTransport<EndpointUndefined> {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            compression: None,
            self_metrics: false,
            endpoint: EndpointUndefined,
        }
    }
}

impl HttpPostTransport<EndpointUndefined> {
    /// Create a new metrics sender.
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the compression algorithm to use.
    pub fn compression(mut self, compression: Option<Compression>) -> Self {
        self.compression = compression;
        self
    }

    /// Set whether to emit internal metrics.
    pub fn self_metrics(mut self, self_metrics: bool) -> Self {
        self.self_metrics = self_metrics;
        self
    }

    /// How long to retry sending the payload before giving up.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the endpoint for the metrics transport.
    pub fn endpoint(self, endpoint: impl ToString) -> HttpPostTransport<EndpointDefined> {
        HttpPostTransport {
            timeout: self.timeout,
            compression: self.compression,
            self_metrics: self.self_metrics,
            endpoint: EndpointDefined(endpoint.to_string()),
        }
    }
}

impl HttpPostTransport<EndpointDefined> {
    /// Set whether to emit internal metrics.
    pub fn self_metrics(mut self, self_metrics: bool) -> Self {
        self.self_metrics = self_metrics;
        self
    }
}

impl Transport for HttpPostTransport<EndpointDefined> {
    fn send(&self, events: &impl Asn1Encode) -> impl Future<Output = io::Result<()>> {
        let timeout = self.timeout;
        let EndpointDefined(endpoint) = &self.endpoint;
        let compression = self.compression;
        let self_metrics = self.self_metrics;

        let controller = AbortController::new().unwrap();
        let signal = controller.signal();

        let headers = Headers::new();
        headers.set("content-type", "application/octet-stream");

        let body = match compression {
            #[cfg(feature = "compress-zstd-external")]
            Some(Compression::Zstd { level }) => {
                headers.set("content-encoding", "zstd");
                events.encode_and_compress_zstd_external(level)
            }
            #[cfg(feature = "compress-brotli")]
            Some(Compression::Brotli) => {
                headers.set("content-encoding", "br");
                events.encode_and_compress_br()
            }
            None => events.encode().map_err(err),
        };

        let req = RequestBuilder::new(endpoint.as_str())
            .method(Method::POST)
            .headers(headers)
            .abort_signal(Some(&signal));

        async move {
            let body = body?;
            let body_size = body.len();
            let req = req.body(body).map_err(err)?;

            let fut = async {
                let res = req.send().await.map_err(err)?;
                if !res.ok() {
                    let text = res.text().await.map_err(|err| err.to_string()).unwrap_or_default();
                    let status = res.status();
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
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
    }
}

fn err(err: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}
