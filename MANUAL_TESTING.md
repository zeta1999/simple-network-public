# simple-network Manual Testing Guide

This guide provides step-by-step procedures to manually verify the core functionality of the `simple-network` library without requiring programmatic integration.

## Prerequisites
Ensure you have the Rust toolchain installed and are inside the project root directory.

Build all library components and examples:
```bash
cargo build --all-targets
```

## Procedure 1: Core Networking & Integration Tests
The library ships with a comprehensive integration test suite that spins up a local network, connects nodes via the `net_kernel`, and successfully exchanges messages over the asynchronous `gen_server` execution loop.

**Run the suite:**
```bash
cargo test
```

**Expected Outcome:**
You should see all unit tests and integration tests successfully pass:
```
test test_gen_server_rpc ... ok
test result: ok. X passed; 0 failed;
```

## Procedure 2: Rust Node Initialization
You can run the mock demo to ensure the `tokio` networking runtime and fundamental crates successfully initialize and don't panic.

**Run the example:**
```bash
cargo run --example rust_demo
```

**Expected Outcome:**
The standard output should confirm the node is running:
```
Local node started: node_1
Starting Raft protocol...
```

## Procedure 3: Cross-Language FFI Links
Verify that the `cgo` library linking is functional and that Go can interface with the Rust core.

**Run the Go Demo:**
```bash
cd go
go build -o demo main.go
./demo
```

**Expected Outcome:**
```
Simple Network Version: 0.1.0
Go node started successfully.
```
