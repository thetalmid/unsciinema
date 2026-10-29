#!/usr/bin/env bash
# Build the Rust core into frontend/src/generated/wasm (web target).
# Run from anywhere. Requires: rustup + wasm32-unknown-unknown + wasm-pack.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/core"
wasm-pack build --target web --out-dir ../frontend/src/generated/wasm --release
echo "WASM package generated at frontend/src/generated/wasm/"
