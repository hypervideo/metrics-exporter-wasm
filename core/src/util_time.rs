use chrono::prelude::*;

#[cfg(target_arch = "wasm32")]
pub fn now() -> DateTime<Utc> {
    let now = wasmtimer::std::SystemTime::now();
    let duration = now
        .duration_since(wasmtimer::std::UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH");
    Utc.timestamp_opt(duration.as_secs() as i64, duration.subsec_nanos())
        .unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn now() -> DateTime<Utc> {
    Utc::now()
}
