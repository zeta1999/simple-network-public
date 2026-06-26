fn main() {
    // Only build the C++ bridge when the `ffi` feature is enabled, so pure-Rust
    // consumers (e.g. depending only on `pqc`) don't need a C++ toolchain.
    if std::env::var("CARGO_FEATURE_FFI").is_ok() {
        cxx_build::bridge("src/ffi/cxx_bridge.rs").compile("simple_network_cxx");
        println!("cargo:rerun-if-changed=src/ffi/cxx_bridge.rs");
    }
}
