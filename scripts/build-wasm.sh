#!/bin/bash
set -e

echo "Building WASM..."
cargo build --release --target wasm32-unknown-unknown

echo "Generating JS bindings with wasm-bindgen..."
wasm-bindgen --out-dir web --target web \
    target/wasm32-unknown-unknown/release/idle_factory.wasm

# Optimize WASM if wasm-opt is available
if command -v wasm-opt &> /dev/null; then
    echo "Optimizing WASM with wasm-opt..."
    wasm-opt -Oz web/idle_factory_bg.wasm -o web/idle_factory_bg.wasm
    echo "WASM optimized!"
else
    echo "Note: wasm-opt not found. Install binaryen for smaller WASM files."
fi

echo "Copying assets to web..."
mkdir -p web/assets/textures/items
cp assets/textures/items/*.png web/assets/textures/items/

# Show file sizes
echo ""
echo "=== Build Results ==="
ls -lh web/idle_factory_bg.wasm
echo ""
echo "Done! Files are in web/"
echo "To test locally: cd web && python3 -m http.server 8080"
