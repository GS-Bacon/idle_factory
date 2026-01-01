#!/bin/bash
# AI間コンテキスト共有スクリプト
# Claude → Gemini へのコンテキスト同期

CONTEXT_DIR="/home/bacon/idle_factory/.ai_context"
mkdir -p "$CONTEXT_DIR"

# 現在のコード状態を要約
generate_context() {
    local output="$CONTEXT_DIR/current_state.md"

    cat > "$output" << 'EOF'
# Idle Factory - 現在の状態

## プロジェクト統計
EOF

    echo "- 総コード行数: $(find src -name '*.rs' -exec cat {} + | wc -l) 行" >> "$output"
    echo "- ファイル数: $(find src -name '*.rs' | wc -l)" >> "$output"
    echo "- テスト件数: $(cargo test 2>&1 | grep -oP '\d+ passed' | head -1)" >> "$output"
    echo "- Clippy警告: $(cargo clippy 2>&1 | grep -c 'warning:')" >> "$output"
    echo "" >> "$output"

    echo "## 最大ファイル (Top 10)" >> "$output"
    find src -name "*.rs" -exec wc -l {} \; 2>/dev/null | sort -rn | head -10 >> "$output"
    echo "" >> "$output"

    echo "## 直近のgitコミット" >> "$output"
    git log --oneline -5 >> "$output"
    echo "" >> "$output"

    echo "## 未コミットの変更" >> "$output"
    git status --short >> "$output"

    echo "Context saved to: $output"
}

# メイン処理
case "${1:-generate}" in
    generate)
        generate_context
        ;;
    show)
        cat "$CONTEXT_DIR/current_state.md"
        ;;
    *)
        echo "Usage: $0 [generate|show]"
        ;;
esac
