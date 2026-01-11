#!/bin/bash
# ビルドしてWindows (baconrogx13) に送信するスクリプト

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
TARGET_HOST="baconrogx13"
ZIP_NAME="idle_factory_build.zip"

# cargoのパスを通す
export PATH="$HOME/.cargo/bin:$PATH"

cd "$PROJECT_DIR"

# target-dirを.cargo/config.tomlから読み取る
TARGET_DIR="$(grep 'target-dir' .cargo/config.toml 2>/dev/null | sed 's/.*= *"\(.*\)"/\1/' || echo "target")"

# オプション解析
BUILD=true
RELEASE=true
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-build) BUILD=false; shift ;;
        --debug) RELEASE=false; shift ;;
        -h|--help)
            echo "Usage: $0 [--no-build] [--debug]"
            echo "  --no-build  ビルドをスキップして既存バイナリを送信"
            echo "  --debug     デバッグビルドを送信"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

# Windowsターゲット
WIN_TARGET="x86_64-pc-windows-gnu"

# ビルド
if [ "$BUILD" = true ]; then
    echo "==> Building for Windows..."
    if [ "$RELEASE" = true ]; then
        cargo build --release --target "$WIN_TARGET"
        BINARY="$TARGET_DIR/$WIN_TARGET/release/idle_factory.exe"
    else
        cargo build --target "$WIN_TARGET"
        BINARY="$TARGET_DIR/$WIN_TARGET/debug/idle_factory.exe"
    fi
else
    if [ "$RELEASE" = true ]; then
        BINARY="$TARGET_DIR/$WIN_TARGET/release/idle_factory.exe"
    else
        BINARY="$TARGET_DIR/$WIN_TARGET/debug/idle_factory.exe"
    fi
fi

# バイナリ存在確認
if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found: $BINARY"
    exit 1
fi

# zip作成 (バイナリ + assets)
echo "==> Creating zip package..."
TEMP_DIR="$(mktemp -d)"
mkdir -p "$TEMP_DIR/idle_factory"

cp "$BINARY" "$TEMP_DIR/idle_factory/"
cp -r assets "$TEMP_DIR/idle_factory/"

cd "$TEMP_DIR"
zip -rq "$ZIP_NAME" idle_factory

# 送信
echo "==> Sending to $TARGET_HOST..."
tailscale file cp "$ZIP_NAME" "$TARGET_HOST":

# クリーンアップ
rm -rf "$TEMP_DIR"

echo "==> Done! Check Downloads folder on Windows"
echo "   Extract idle_factory_build.zip and run idle_factory.exe"
