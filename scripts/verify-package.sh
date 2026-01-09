#!/bin/bash
# 配布パッケージの必須ファイル検証スクリプト
# Usage: ./scripts/verify-package.sh <package_dir>
#        ./scripts/verify-package.sh --check-source  # ソースの必須ファイル確認

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

# 必須ファイル/ディレクトリのリスト
# 新しい必須ファイルはここに追加する
REQUIRED_FILES=(
    # Base Mod（ゲーム起動に必須）
    "mods/base/mod.toml"
    "mods/base/items.toml"
    "mods/base/machines.toml"
    "mods/base/recipes.toml"

    # アセット
    "assets/data/quests.json"
    "assets/textures/block_atlas_default.png"
    "assets/fonts/NotoSansJP-Regular.ttf"
    "assets/locales/ja.ron"
    "assets/locales/en.ron"
)

REQUIRED_DIRS=(
    "mods/base"
    "assets/data"
    "assets/fonts"
    "assets/locales"
    "assets/models"
    "assets/textures"
)

# ソースディレクトリのチェック（開発時用）
check_source() {
    local root="${1:-.}"
    local errors=0

    echo "=== ソースディレクトリの必須ファイル確認 ==="
    echo ""

    for file in "${REQUIRED_FILES[@]}"; do
        if [ -f "$root/$file" ]; then
            info "$file"
        else
            error "$file が見つかりません"
            ((errors++))
        fi
    done

    echo ""
    for dir in "${REQUIRED_DIRS[@]}"; do
        if [ -d "$root/$dir" ]; then
            info "$dir/"
        else
            error "$dir/ ディレクトリが見つかりません"
            ((errors++))
        fi
    done

    echo ""
    if [ $errors -gt 0 ]; then
        error "=== $errors 個のエラーがあります ==="
        return 1
    else
        info "=== 全ての必須ファイルが存在します ==="
        return 0
    fi
}

# パッケージディレクトリのチェック（ビルド後用）
check_package() {
    local pkg_dir="$1"
    local errors=0

    if [ ! -d "$pkg_dir" ]; then
        error "パッケージディレクトリが見つかりません: $pkg_dir"
        return 1
    fi

    echo "=== パッケージ検証: $pkg_dir ==="
    echo ""

    # 実行ファイル確認
    if [ -f "$pkg_dir/idle_factory" ] || [ -f "$pkg_dir/idle_factory.exe" ]; then
        info "実行ファイル"
    else
        error "実行ファイルが見つかりません"
        ((errors++))
    fi

    # 必須ファイル確認
    for file in "${REQUIRED_FILES[@]}"; do
        if [ -f "$pkg_dir/$file" ]; then
            info "$file"
        else
            error "$file が見つかりません"
            ((errors++))
        fi
    done

    # 必須ディレクトリ確認
    for dir in "${REQUIRED_DIRS[@]}"; do
        if [ -d "$pkg_dir/$dir" ]; then
            local count=$(find "$pkg_dir/$dir" -type f | wc -l)
            info "$dir/ ($count files)"
        else
            error "$dir/ ディレクトリが見つかりません"
            ((errors++))
        fi
    done

    echo ""
    if [ $errors -gt 0 ]; then
        error "=== $errors 個のエラーがあります ==="
        echo ""
        echo "修正方法:"
        echo "1. scripts/build-packages.sh の copy_assets() を確認"
        echo "2. 必須ファイルが REQUIRED_FILES に含まれているか確認"
        return 1
    else
        info "=== パッケージは有効です ==="
        return 0
    fi
}

# メイン
case "$1" in
    --check-source)
        check_source "${2:-.}"
        ;;
    --help|-h)
        echo "Usage: $0 <package_dir>         # パッケージを検証"
        echo "       $0 --check-source [dir]  # ソースの必須ファイル確認"
        ;;
    *)
        if [ -z "$1" ]; then
            echo "Usage: $0 <package_dir>"
            exit 1
        fi
        check_package "$1"
        ;;
esac
