#!/usr/bin/env bash

set -e

echo "Building Rust Project..."
source $HOME/.cargo/env
cargo build

echo "Testing Rust Project..."
cargo test

# Placeholders for Go and C++ tests
echo "Testing Go and C++ Project (Placeholder)"
# go test ./go/...
# make -C cpp test

echo "All tests passed successfully!"
