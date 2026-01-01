#!/bin/bash
# WASM E2E Quick Test - 基本動作確認のみ
#
# 使い方: ./scripts/e2e-wasm-quick.sh

set -e

export DISPLAY=${DISPLAY:-:10}

GAME_DIR="/home/bacon/idle_factory"
SCREENSHOTS_DIR="$GAME_DIR/screenshots/e2e_wasm"
WEB_DIR="$GAME_DIR/web"
PORT=8888

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

log() { echo -e "${GREEN}[WASM]${NC} $1"; }
err() { echo -e "${RED}[ERR]${NC} $1"; }

cleanup() {
    pkill -f "simple-http-server" 2>/dev/null || true
    pkill -f "chromium" 2>/dev/null || true
}
trap cleanup EXIT

mkdir -p "$SCREENSHOTS_DIR"

# WASMビルド確認
if [ ! -f "$WEB_DIR/idle_factory_bg.wasm" ]; then
    log "WASMビルド中..."
    cd "$GAME_DIR"
    ./scripts/build-wasm.sh || {
        err "WASMビルド失敗"
        exit 1
    }
fi

# HTTPサーバー起動
log "HTTPサーバー起動 (port $PORT)..."
cd "$WEB_DIR"
simple-http-server --port $PORT --silent &
sleep 2

# ブラウザでスクリーンショット
log "ブラウザ起動..."
chromium-browser --headless --screenshot="$SCREENSHOTS_DIR/wasm_test.png" \
    --window-size=1280,720 \
    --disable-gpu \
    --no-sandbox \
    "http://localhost:$PORT" 2>/dev/null &

# 30秒待機
sleep 30

# スクリーンショット確認
if [ -f "$SCREENSHOTS_DIR/wasm_test.png" ]; then
    log "✅ WASMスクリーンショット取得成功"
    log "ファイル: $SCREENSHOTS_DIR/wasm_test.png"
    ls -la "$SCREENSHOTS_DIR/wasm_test.png"
else
    err "❌ WASMスクリーンショット取得失敗"
    exit 1
fi

log "WASM E2Eクイックテスト完了"
