use asn1rs::{
    converter::Converter,
    model::generate::RustCodeGenerator,
};
use std::{
    env::var,
    path::PathBuf,
};

pub fn main() {
    generate_asn1_types();

    #[cfg(feature = "compress-zstd-external-from-source")]
    generate_zstd_wasm_bundle();
}

const ASN_FILES: &[&str] = &["./src/metrics.asn"];

fn generate_asn1_types() {
    let mut converter = Converter::default();
    for asn_file in ASN_FILES {
        println!("cargo:rerun-if-changed={}", asn_file);
        if let Err(error) = converter.load_file(asn_file) {
            panic!("Loading of .asn1 file failed {}: {:?}", asn_file, error);
        }
    }

    let out_dir = PathBuf::from(var("OUT_DIR").unwrap());
    if let Err(error) = converter.to_rust(
        out_dir.to_str().unwrap(),
        |#[allow(unused)] generator: &mut RustCodeGenerator| {
            #[cfg(feature = "serde")]
            {
                generator.add_global_derive("serde::Serialize");
                generator.add_global_derive("serde::Deserialize");
            }
            #[cfg(feature = "utoipa-schema")]
            {
                generator.add_global_derive("utoipa::ToSchema");
            }
        },
    ) {
        panic!("Conversion to rust failed: {:?}", error);
    }
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

#[cfg(feature = "compress-zstd-external-from-source")]
fn generate_zstd_wasm_bundle() {
    use std::process::Command;
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
