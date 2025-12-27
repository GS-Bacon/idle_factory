#!/bin/bash
# WASM自動リビルド＆デプロイ
# ファイル変更を監視して自動的にWASMをビルド・デプロイする

set -e
cd /home/bacon/idle_factory

echo "=== WASM Watch Mode ==="
echo "ファイル変更を監視中..."
echo "終了: Ctrl+C"
echo ""

# 初回デプロイ
./deploy-wasm.sh

# ファイル変更を監視してデプロイ
# -w: 監視するパス
# -c: 画面クリア
# -q: 静かなモード
# -d 500: 500ms待機（連続保存対策）
cargo watch \
    -w src \
    -w Cargo.toml \
    -w assets \
    -c \
    -d 500 \
    -s "./deploy-wasm.sh"
