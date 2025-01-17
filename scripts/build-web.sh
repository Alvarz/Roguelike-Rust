#!/bin/bash
rustup target add wasm32-unknown-unknown
cargo install -f wasm-bindgen-cli --version 0.2.99
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/untitled-roguelike.wasm --out-dir wasm --no-modules --no-typescript
mv wasm/untitled-roguelike.js wasm/blob.js
mv wasm/untitled-roguelike_bg.wasm wasm/blob.wasm
