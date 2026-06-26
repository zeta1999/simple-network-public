pub mod algorithms;
pub mod patterns;
pub mod security;
pub mod transport;

// FFI (C++/Go bridges) is optional so pure-Rust consumers don't need a C++
// toolchain. On by default to preserve existing behavior.
#[cfg(feature = "ffi")]
pub mod ffi;
