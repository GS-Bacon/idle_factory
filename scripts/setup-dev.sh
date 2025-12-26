#!/bin/bash
# 開発環境セットアップスクリプト
# Usage: ./scripts/setup-dev.sh

set -e

echo "=== Idle Factory 開発環境セットアップ ==="

# OS検出
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
    OS="windows"
else
    echo "未対応のOS: $OSTYPE"
    exit 1
fi

echo "検出されたOS: $OS"

# Rust インストール確認
if ! command -v rustc &> /dev/null; then
    echo "Rustをインストール中..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rust: $(rustc --version)"
fi

# Linux用セットアップ
if [[ "$OS" == "linux" ]]; then
    echo ""
    echo "=== Linux用依存関係インストール ==="

    # パッケージマネージャ検出
    if command -v apt &> /dev/null; then
        PKG_MGR="apt"
        INSTALL_CMD="sudo apt install -y"
    elif command -v dnf &> /dev/null; then
        PKG_MGR="dnf"
        INSTALL_CMD="sudo dnf install -y"
    elif command -v pacman &> /dev/null; then
        PKG_MGR="pacman"
        INSTALL_CMD="sudo pacman -S --noconfirm"
    else
        echo "警告: パッケージマネージャが見つかりません。手動でインストールしてください。"
        PKG_MGR="unknown"
    fi

    echo "パッケージマネージャ: $PKG_MGR"

    if [[ "$PKG_MGR" != "unknown" ]]; then
        # 高速リンカ (mold) とclang
        echo "mold (高速リンカ) をインストール中..."
        if [[ "$PKG_MGR" == "apt" ]]; then
            sudo apt update
            $INSTALL_CMD mold clang
        elif [[ "$PKG_MGR" == "dnf" ]]; then
            $INSTALL_CMD mold clang
        elif [[ "$PKG_MGR" == "pacman" ]]; then
            $INSTALL_CMD mold clang
        fi

        # Bevy依存関係 (Linux)
        echo "Bevy依存関係をインストール中..."
        if [[ "$PKG_MGR" == "apt" ]]; then
            $INSTALL_CMD g++ pkg-config libx11-dev libasound2-dev libudev-dev libxkbcommon-x11-0 libwayland-dev libxkbcommon-dev
        elif [[ "$PKG_MGR" == "dnf" ]]; then
            $INSTALL_CMD gcc-c++ libX11-devel alsa-lib-devel systemd-devel libxkbcommon-devel wayland-devel
        elif [[ "$PKG_MGR" == "pacman" ]]; then
            $INSTALL_CMD base-devel libx11 alsa-lib systemd libxkbcommon wayland
        fi
    fi
fi

# Windows用セットアップ (Git Bashなどから実行時)
if [[ "$OS" == "windows" ]]; then
    echo ""
    echo "=== Windows用セットアップ ==="
    echo "必要なもの:"
    echo "  - Visual Studio Build Tools (C++ビルドツール)"
    echo "  - rust-lld は Cargo.toml で設定済み"
    echo ""
    echo "Visual Studio Build Toolsがない場合:"
    echo "  https://visualstudio.microsoft.com/visual-cpp-build-tools/"
fi

# ビルド確認
echo ""
echo "=== ビルドテスト ==="
echo "cargo check を実行中..."
cargo check

echo ""
echo "=== セットアップ完了 ==="
echo ""
echo "次のコマンドでゲームを起動できます:"
echo "  cargo run"
