#!/bin/bash
# カーソル・UI関連のログのみ抽出してゲームを実行
#
# 使い方:
#   ./scripts/cursor-log.sh          # ゲーム実行 + カーソル/UIログのみ表示
#   ./scripts/cursor-log.sh --filter # ログファイルから抽出のみ（ゲーム実行なし）
#
# 出力例:
#   [Cursor] lock_cursor called
#   [Cursor] release_cursor called
#   [UI] Action: Cancel, current: Gameplay
#   [UIState] push: PauseMenu

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ "$1" = "--filter" ]; then
    # ログファイルからフィルタのみ
    LOG_FILE="$PROJECT_DIR/logs/game_latest.log"
    if [ -f "$LOG_FILE" ]; then
        grep -E '\[Cursor\]|\[UI\]|\[UIState\]' "$LOG_FILE" | tail -100
    else
        echo "Error: Log file not found: $LOG_FILE"
        exit 1
    fi
else
    # ゲーム実行 + フィルタ
    cd "$PROJECT_DIR"
    RUST_LOG=warn,idle_factory::systems::cursor=debug,idle_factory::systems::ui_navigation=info,idle_factory::components::ui_state=info \
        cargo run 2>&1 | grep -E '\[Cursor\]|\[UI\]|\[UIState\]|error|panic' || true
fi
