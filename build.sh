#!/bin/sh
cargo build --profile wasm-release --target wasm32-unknown-unknown
wasm-bindgen --out-name therm \
  --out-dir wasm/target \
  --target web target/wasm32-unknown-unknown/wasm-release/therm.wasm
wasm-opt -Oz --output wasm/target/therm_bg.wasm wasm/target/therm_bg.wasm