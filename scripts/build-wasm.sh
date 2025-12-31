#!/bin/bash
set -e

echo "Building WASM..."
cargo build --release --target wasm32-unknown-unknown

echo "Generating JS bindings with wasm-bindgen..."
wasm-bindgen --out-dir web --target web \
    target/wasm32-unknown-unknown/release/idle_factory.wasm

echo "Copying assets to web..."
mkdir -p web/assets/textures/items
cp assets/textures/items/*.png web/assets/textures/items/

echo "Done! Files are in web/"
echo "To test locally: cd web && python3 -m http.server 8080"
