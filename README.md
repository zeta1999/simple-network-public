# simple-network

A small async transport layer with an optional post-quantum secure channel and an optional Tor carrier.

Part of [**simple tools**](https://zeta1999.github.io/renoir42/simple-tools.html) — small, composable Rust libraries for building tooling fast from a harness.

## Features

The base library is intentionally lean; everything heavy is behind a feature flag.

- **default (`ffi`)** — TLS transport (rustls) over tokio, with C++ and Go bridges for existing consumers. Build pure-Rust with `--no-default-features` to skip the C++ toolchain.
- **`pqc`** — a hybrid post-quantum secure channel: ML-KEM key agreement + XChaCha20 AEAD + ML-DSA signatures, with replay/reflection protection and a hardened pairing KDF. Keys live in [`rust-secure-memory`](https://github.com/zeta1999/rust-secure-memory-public) and are zeroized on drop.
- **`tor`** — an onion carrier (client connect + onion-service hosting) via `arti`. Heavy dependency tree, off by default.

## Build

```sh
cargo build                          # base, with FFI bridges
cargo build --no-default-features    # pure Rust
cargo build --features pqc           # post-quantum secure channel
cargo build --features tor           # Tor onion carrier
```

## Dependencies

`pqc` pulls in [`rust-secure-memory-public`](https://github.com/zeta1999/rust-secure-memory-public) as a sibling crate.

## License

MIT OR Apache-2.0
