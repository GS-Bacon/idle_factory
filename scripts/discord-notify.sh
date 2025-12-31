#!/bin/bash
# Discord Webhook通知スクリプト
# 使用例: ./discord-notify.sh "作業完了メッセージ"

WEBHOOK_URL="https://discord.com/api/webhooks/1455035529753006162/ZLP1OD_B3Ow7JFSGb0LftsFRrYajdPeEKSBoK8wNHjqm7fb21zh8PgKtT831bgO3xOO6"

if [ -z "$1" ]; then
    echo "Usage: ./discord-notify.sh \"message\""
    exit 1
fi

# メッセージをJSONセーフにエスケープ（jqを使用）
MESSAGE=$(echo "$1" | jq -Rs '.')

# jqがない場合のフォールバック
if [ -z "$MESSAGE" ]; then
    MESSAGE="\"$1\""
fi

HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$WEBHOOK_URL" \
    -H "Content-Type: application/json" \
    -d "{\"content\": $MESSAGE}")

if [ "$HTTP_CODE" = "204" ]; then
    echo "✅ Discord送信成功 (HTTP $HTTP_CODE)"
else
    echo "❌ Discord送信失敗 (HTTP $HTTP_CODE)"
    exit 1
fi
