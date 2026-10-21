#!/bin/sh
# Builds the Typst plugin into src/solveng.wasm (needs the wasm32-unknown-unknown target; wasm-opt is optional).
set -e
cd "$(dirname "$0")/.."
cargo build -p solveng-plugin --profile wasm --target wasm32-unknown-unknown
out=target/wasm32-unknown-unknown/wasm/solveng_plugin.wasm
if command -v wasm-opt >/dev/null; then
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext "$out" -o src/solveng.wasm
else
  cp "$out" src/solveng.wasm
fi
