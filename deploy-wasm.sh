#!/bin/bash
# WASM自動ビルド＆デプロイスクリプト
# コミット後に実行すると、WASMをビルドしてサーバーを再起動する

set -e
cd /home/bacon/idle_factory

echo "=== WASM Deploy Script ==="

# 既存のサーバーを停止
echo "Stopping existing server..."
pkill -f "python3 -m http.server 8080" 2>/dev/null || true
sleep 1

# WASMビルド
echo "Building WASM..."
cargo build --release --target wasm32-unknown-unknown

# JSバインディング生成
echo "Generating JS bindings..."
wasm-bindgen --out-dir web --target web \
    target/wasm32-unknown-unknown/release/idle_factory.wasm

# wasm-optで最適化（サイズ＆速度）
echo "Optimizing WASM..."
wasm-opt -O3 --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals \
    web/idle_factory_bg.wasm -o web/idle_factory_bg_opt.wasm
mv web/idle_factory_bg_opt.wasm web/idle_factory_bg.wasm

# assetsをwebにコピー
echo "Copying assets..."
mkdir -p web/assets/fonts
cp assets/fonts/NotoSansJP-Regular.ttf web/assets/fonts/

# サーバー起動（バックグラウンド）
echo "Starting server..."
cd web
nohup python3 -m http.server 8080 --bind 0.0.0.0 > /tmp/wasm-server.log 2>&1 &
SERVER_PID=$!
echo "Server started with PID: $SERVER_PID"

sleep 1

# 確認
echo ""
echo "=== Access URLs ==="
echo "Local:     http://10.13.1.1:8080"
echo "Tailscale: http://100.84.170.32:8080"
echo ""
echo "Done!"
