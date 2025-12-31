#!/bin/bash
# WASM自動ビルド＆デプロイスクリプト
# コミット後に実行すると、WASMをビルドしてサーバーを再起動する

set -e
cd /home/bacon/idle_factory

echo "=== WASM Deploy Script ==="

# サーバーを一時停止（systemd管理）
echo "Stopping server..."
sudo systemctl stop idle-factory-web 2>/dev/null || true

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

# 3Dモデルをコピー
if [ -d "assets/models" ]; then
    echo "Copying 3D models..."
    cp -r assets/models web/assets/
fi

# サーバー再起動（systemd管理）
echo "Starting server..."
sudo systemctl start idle-factory-web

sleep 1

# 確認
echo ""
echo "=== Access URLs ==="
echo "Local:     http://10.13.1.1:8080"
echo "Tailscale: http://100.84.170.32:8080"
echo "Public:    https://serversmith.tail1b7bff.ts.net"
echo ""
echo "Done!"
