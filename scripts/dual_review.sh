#!/bin/bash
# デュアルAIレビュースクリプト
# Claude と Gemini の両方にレビューを依頼し、結果を比較

set -e

REVIEW_DIR="/home/bacon/idle_factory/.ai_context"
mkdir -p "$REVIEW_DIR"

# レビュー対象ファイルを収集
TARGET_FILES="${1:-src/main.rs}"
REVIEW_TYPE="${2:-general}"  # general, architecture, safety, performance

echo "=== Dual AI Review ==="
echo "Target: $TARGET_FILES"
echo "Type: $REVIEW_TYPE"
echo ""

# コンテキストファイル作成
CONTEXT_FILE="$REVIEW_DIR/review_context_$(date +%Y%m%d_%H%M%S).md"

cat > "$CONTEXT_FILE" << EOF
# コードレビュー依頼

## レビュータイプ: $REVIEW_TYPE

## 評価基準
1. アーキテクチャ (10点): モジュール分割、依存関係
2. コード品質 (10点): 可読性、命名規則
3. Bevyパターン (10点): ECS活用、Plugin構成
4. 安全性 (10点): エラーハンドリング、unwrap使用
5. 拡張性 (10点): 将来の機能追加

## 出力形式
- 各項目のスコア
- 良い点3つ
- 改善点3つ
- 総合評価

---

## レビュー対象コード

EOF

# ファイル内容を追加
for file in $TARGET_FILES; do
    if [ -f "$file" ]; then
        echo "### $file" >> "$CONTEXT_FILE"
        echo '```rust' >> "$CONTEXT_FILE"
        head -200 "$file" >> "$CONTEXT_FILE"
        echo '```' >> "$CONTEXT_FILE"
        echo "" >> "$CONTEXT_FILE"
    fi
done

echo "Context file created: $CONTEXT_FILE"
echo ""

# Geminiにレビュー依頼
echo "Sending to Gemini..."
./scripts/ask_gemini.sh "このコードをレビューしてください。評価基準に従って10点満点で採点し、良い点3つ、改善点3つを挙げてください。" "$CONTEXT_FILE" &
GEMINI_PID=$!

echo "Gemini review requested (PID: $GEMINI_PID)"
echo ""
echo "Claude review: 直接会話で依頼してください"
echo "Gemini review: tmux attach -t ai_gemini で確認"
