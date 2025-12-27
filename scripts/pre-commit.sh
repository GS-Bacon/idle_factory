#!/bin/bash
# Pre-commit hook: メッシュのワインディング順序を自動検証

# src/main.rs が変更された場合のみ実行
if git diff --cached --name-only | grep -q "src/main.rs"; then
    echo "Running winding order test..."

    if ! cargo test test_mesh_winding_order --quiet 2>/dev/null; then
        echo ""
        echo "ERROR: Winding order test failed!"
        echo "地面が透けるバグの可能性があります。"
        echo ""
        echo "確認: cargo test test_mesh_winding_order"
        echo "修正: src/main.rs の faces 配列の頂点順序を確認"
        echo ""
        exit 1
    fi

    echo "Winding order test passed."
fi

exit 0
