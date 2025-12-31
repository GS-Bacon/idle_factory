#!/bin/bash
# モジュールサイズチェックスクリプト
# 500行以上のファイルを警告表示
# Usage: ./tools/check_module_sizes.sh

WARNING_THRESHOLD=500
ERROR_THRESHOLD=1000

echo "=== Module Size Check ==="
echo ""

# 全Rustファイルの行数を取得
large_files=()
very_large_files=()

while IFS= read -r line; do
    count=$(echo "$line" | awk '{print $1}')
    file=$(echo "$line" | awk '{print $2}')

    if [ "$count" -ge "$ERROR_THRESHOLD" ]; then
        very_large_files+=("$count $file")
    elif [ "$count" -ge "$WARNING_THRESHOLD" ]; then
        large_files+=("$count $file")
    fi
done < <(find src -name "*.rs" -exec wc -l {} \; | sort -rn)

# 結果表示
if [ ${#very_large_files[@]} -gt 0 ]; then
    echo "🔴 ERROR: ファイルが大きすぎます (>${ERROR_THRESHOLD}行) - 分割を検討:"
    for f in "${very_large_files[@]}"; do
        echo "   $f"
    done
    echo ""
fi

if [ ${#large_files[@]} -gt 0 ]; then
    echo "🟡 WARNING: ファイルが大きめです (>${WARNING_THRESHOLD}行) - 監視が必要:"
    for f in "${large_files[@]}"; do
        echo "   $f"
    done
    echo ""
fi

if [ ${#very_large_files[@]} -eq 0 ] && [ ${#large_files[@]} -eq 0 ]; then
    echo "✅ 全てのモジュールは適切なサイズです (<${WARNING_THRESHOLD}行)"
fi

echo ""
echo "=== 現在のモジュール構成 ==="
find src -name "*.rs" -exec wc -l {} \; | sort -rn | head -20

echo ""
echo "=== 推奨される分割基準 ==="
echo "- 500行以上: 分割を検討"
echo "- 1000行以上: 優先的に分割"
echo "- 主な分割単位: システム関数、コンポーネント、ユーティリティ"
