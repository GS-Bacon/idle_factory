#!/bin/bash
# Core Mod ビルドスクリプト

set -e

MOD_NAME=${1:-sample_core_mod}
BUILD_TYPE=${2:-release}

echo "Building mod: $MOD_NAME ($BUILD_TYPE)"

# WASMターゲットがなければ追加
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# target-dirを取得（.cargo/config.tomlで設定されている場合）
TARGET_DIR=$(cargo metadata --format-version 1 2>/dev/null | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
if [ -z "$TARGET_DIR" ]; then
    TARGET_DIR="target"
fi

# ビルド
if [ "$BUILD_TYPE" = "release" ]; then
    cargo build -p "$MOD_NAME" --target wasm32-unknown-unknown --release
    WASM_PATH="${TARGET_DIR}/wasm32-unknown-unknown/release/${MOD_NAME}.wasm"
else
    cargo build -p "$MOD_NAME" --target wasm32-unknown-unknown
    WASM_PATH="${TARGET_DIR}/wasm32-unknown-unknown/debug/${MOD_NAME}.wasm"
fi

# コピー（存在する場合）
if [ -f "$WASM_PATH" ]; then
    cp "$WASM_PATH" "mods/${MOD_NAME}/"
    echo "Built: mods/${MOD_NAME}/${MOD_NAME}.wasm"
    ls -lh "mods/${MOD_NAME}/${MOD_NAME}.wasm"
else
    echo "Error: WASM file not found at $WASM_PATH"
    exit 1
fi
