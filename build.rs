use asn1rs::{converter::Converter, model::generate::RustCodeGenerator};
use std::{env::var, path::PathBuf};

pub fn main() {
    generate_asn1_types();
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
        |_generator: &mut RustCodeGenerator| {
            // generator.add_global_derive("serde::Serialize");
            // generator.add_global_derive("serde::Deserialize");
        },
    ) {
        panic!("Conversion to rust failed: {:?}", error);
    }
}
