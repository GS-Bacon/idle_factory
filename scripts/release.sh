#!/bin/bash
# リリーススクリプト
# Usage: ./scripts/release.sh <version>
# Example: ./scripts/release.sh 0.2.0

set -e

if [ -z "$1" ]; then
    echo "Usage: ./scripts/release.sh <version>"
    echo "Example: ./scripts/release.sh 0.2.0"
    exit 1
fi

VERSION="$1"
TAG="v$VERSION"

# バージョン形式チェック
if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "Error: Invalid version format. Use X.Y.Z (e.g., 0.2.0)"
    exit 1
fi

# 未コミットの変更がないか確認
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "Error: Uncommitted changes exist. Commit or stash them first."
    exit 1
fi

# タグが既に存在しないか確認
if git tag -l "$TAG" | grep -q "$TAG"; then
    echo "Error: Tag $TAG already exists."
    exit 1
fi

echo "=== Releasing $TAG ==="

# 1. Cargo.tomlのバージョンを更新
echo "[1/4] Updating Cargo.toml version..."
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# 2. コミット
echo "[2/4] Committing version change..."
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to $VERSION

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"

# 3. タグ作成
echo "[3/4] Creating tag $TAG..."
git tag "$TAG"

# 4. プッシュ
echo "[4/4] Pushing to remote..."
git push origin master
git push origin "$TAG"

echo ""
echo "=== Release $TAG complete ==="
echo "GitHub Actions will now build and publish the release."
echo "Check: https://github.com/GS-Bacon/idle_factory/releases"
