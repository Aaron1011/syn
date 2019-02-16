use serde::Deserialize;

use std::fs;
use std::path::Path;

pub fn get() -> String {
    let codegen_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let syn_cargo_toml = codegen_root.join("../Cargo.toml");
    let manifest = fs::read_to_string(syn_cargo_toml).unwrap();
    let parsed: Manifest = toml::from_str(&manifest).unwrap();
    parsed.package.version
}

#[derive(Debug, Deserialize)]
struct Manifest {
    package: Package,
}

#[derive(Debug, Deserialize)]
struct Package {
    version: String,
}