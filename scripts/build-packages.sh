#!/bin/bash
# 配布用パッケージビルドスクリプト
# Usage: ./scripts/build-packages.sh [--windows-only] [--linux-only]

set -e

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

# バージョン取得（Cargo.tomlから）
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
DIST_DIR="$PROJECT_ROOT/dist"

# 色付き出力
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# 引数解析
BUILD_WINDOWS=true
BUILD_LINUX=true

UPLOAD=false

for arg in "$@"; do
    case $arg in
        --windows-only) BUILD_LINUX=false ;;
        --linux-only) BUILD_WINDOWS=false ;;
        --upload) UPLOAD=true ;;
    esac
done

# distディレクトリ作成
mkdir -p "$DIST_DIR"

# アセットをコピーする関数
copy_assets() {
    local dest="$1"
    info "アセットをコピー中..."

    # 必要なアセットフォルダをコピー
    cp -r assets/data "$dest/assets/" 2>/dev/null || true
    cp -r assets/fonts "$dest/assets/" 2>/dev/null || true
    cp -r assets/locales "$dest/assets/" 2>/dev/null || true
    cp -r assets/models "$dest/assets/" 2>/dev/null || true
    cp -r assets/shaders "$dest/assets/" 2>/dev/null || true
    cp -r assets/sounds "$dest/assets/" 2>/dev/null || true
    cp -r assets/textures "$dest/assets/" 2>/dev/null || true

    # 不要ファイルを除外
    find "$dest/assets" -name "*.vox" -delete 2>/dev/null || true
    find "$dest/assets" -name ".gitkeep" -delete 2>/dev/null || true
}

# Linux用ビルド
if [ "$BUILD_LINUX" = true ]; then
    info "=== Linux用パッケージをビルド中 ==="

    cargo build --release

    LINUX_DIR="$DIST_DIR/idle_factory_${VERSION}_linux"
    rm -rf "$LINUX_DIR"
    mkdir -p "$LINUX_DIR/assets"

    cp target/release/idle_factory "$LINUX_DIR/"
    copy_assets "$LINUX_DIR"

    # 起動スクリプト
    cat > "$LINUX_DIR/run.sh" << 'EOF'
#!/bin/bash
cd "$(dirname "$0")"
./idle_factory "$@"
EOF
    chmod +x "$LINUX_DIR/run.sh"
    chmod +x "$LINUX_DIR/idle_factory"

    # tarball作成
    (cd "$DIST_DIR" && tar -czvf "idle_factory_${VERSION}_linux.tar.gz" "idle_factory_${VERSION}_linux")
    rm -rf "$LINUX_DIR"

    info "Linux用パッケージ完了: dist/idle_factory_${VERSION}_linux.tar.gz"
fi

# Windows用ビルド
if [ "$BUILD_WINDOWS" = true ]; then
    info "=== Windows用パッケージをビルド中 ==="

    # クロスコンパイル
    cargo build --release --target x86_64-pc-windows-gnu

    WINDOWS_DIR="$DIST_DIR/idle_factory_${VERSION}_windows"
    rm -rf "$WINDOWS_DIR"
    mkdir -p "$WINDOWS_DIR/assets"

    cp target/x86_64-pc-windows-gnu/release/idle_factory.exe "$WINDOWS_DIR/"
    copy_assets "$WINDOWS_DIR"

    # zip作成
    (cd "$DIST_DIR" && zip -r "idle_factory_${VERSION}_windows.zip" "idle_factory_${VERSION}_windows")
    rm -rf "$WINDOWS_DIR"

    info "Windows用パッケージ完了: dist/idle_factory_${VERSION}_windows.zip"
fi

info "=== 全パッケージビルド完了 ==="
ls -lh "$DIST_DIR"/*.{tar.gz,zip} 2>/dev/null || true

# --uploadオプションでGitHub Releaseにアップロード
if [ "$UPLOAD" = true ]; then
    info "=== GitHub Releaseにアップロード中 ==="

    # latestリリースが存在しない場合は作成
    if ! gh release view latest &>/dev/null; then
        gh release create latest --title "Latest Build" --prerelease --notes "Automatically updated latest build."
    fi

    # アップロード（既存ファイルは上書き）
    gh release upload latest "$DIST_DIR"/*.tar.gz "$DIST_DIR"/*.zip --clobber

    info "アップロード完了: https://github.com/GS-Bacon/idle_factory/releases/tag/latest"
fi
