pub fn main() {
    #[cfg(feature = "compress-zstd-external-from-source")]
    generate_zstd_wasm_bundle();
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

#[cfg(feature = "compress-zstd-external-from-source")]
fn generate_zstd_wasm_bundle() {
    use std::{
        env::var,
        path::PathBuf,
        process::Command,
    };
    let out_dir = var("OUT_DIR").unwrap();
    let build_script = PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap()).join("build-zstd-wasm.sh");
    println!("cargo:rerun-if-changed={}", build_script.to_str().unwrap());
    Command::new("sh")
        .arg(build_script)
        .current_dir(out_dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .expect("Failed to execute build-zstd-wasm.sh");
}
