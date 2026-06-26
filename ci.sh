#!/usr/bin/env bash
#
# Local CI for simple-network (Linux + macOS).
#
# Runs the host-native gates as hard failures. Crucially it lints with
# `-D warnings` on the REAL cargo exit code and runs the security test suite
# under `--features pqc`, which a plain `cargo test` does not (those tests are
# feature-gated and would otherwise be silently skipped).
set -euo pipefail

echo "Running CI for simple-network..."

echo "==> Checking formatting..."
cargo fmt -- --check

echo "==> Clippy (default features)..."
cargo clippy --all-targets -- -D warnings

echo "==> Clippy (all features)..."
cargo clippy --all-targets --all-features -- -D warnings

echo "==> Tests (default features)..."
cargo test

echo "==> Tests (security suite, --features pqc)..."
cargo test --features pqc

echo "CI completed successfully!"
