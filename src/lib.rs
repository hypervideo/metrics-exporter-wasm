mod recorder;
pub mod types;

pub use recorder::{WasmRecorder, WasmRecorderBuilder};

#[macro_use]
extern crate tracing;
