#!/bin/bash
# parallel-plan.sh - 並列計画の実行管理
#
# 使い方:
#   ./scripts/parallel-plan.sh run <plan.json>    # 計画を実行
#   ./scripts/parallel-plan.sh validate <plan.json>  # 計画を検証
#   ./scripts/parallel-plan.sh status             # 実行中計画の状態
#   ./scripts/parallel-plan.sh example            # 計画例を出力

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CURRENT_PLAN="$PROJECT_ROOT/.claude/current-plan.json"
PLAN_STATUS="$PROJECT_ROOT/.claude/plan-status.json"
WORKTREES_DIR="/mnt/build/worktrees"

# 色定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_success() { echo -e "${GREEN}[OK]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }
log_phase() { echo -e "${CYAN}[PHASE]${NC} $*"; }

check_jq() {
    if ! command -v jq &> /dev/null; then
        log_error "jq がインストールされていません: sudo apt-get install jq"
        exit 1
    fi
}

# 計画の検証
validate_plan() {
    local plan_file="$1"
    check_jq

    if [[ ! -f "$plan_file" ]]; then
        log_error "計画ファイルが見つかりません: $plan_file"
        return 1
    fi

    # JSON形式チェック
    if ! jq empty "$plan_file" 2>/dev/null; then
        log_error "無効なJSON形式です"
        return 1
    fi

    # 必須フィールドチェック
    local name=$(jq -r '.name // empty' "$plan_file")
    if [[ -z "$name" ]]; then
        log_error "name フィールドが必要です"
        return 1
    fi

    local phases=$(jq -r '.phases | length' "$plan_file")
    if [[ "$phases" -eq 0 ]]; then
        log_error "phases が空です"
        return 1
    fi

    # 依存関係チェック
    log_info "依存関係をチェック中..."
    local all_ids=$(jq -r '.phases[].tasks[]?.id // empty, .phases[].groups[]?.tasks[]?.id // empty' "$plan_file" | sort -u)
    local all_deps=$(jq -r '.phases[].groups[]?.depends_on[]? // empty' "$plan_file" | sort -u)

    for dep in $all_deps; do
        if ! echo "$all_ids" | grep -q "^${dep}$"; then
            # グループIDかチェック
            local group_ids=$(jq -r '.phases[].groups[]?.id // empty' "$plan_file")
            if ! echo "$group_ids" | grep -q "^${dep}$"; then
                log_error "依存先が見つかりません: $dep"
                return 1
            fi
        fi
    done

    # ファイル衝突チェック
    log_info "ファイル衝突をチェック中..."
    local files=$(jq -r '.phases[].groups[]?.tasks[]?.file // empty' "$plan_file" | sort)
    local duplicates=$(echo "$files" | uniq -d)
    if [[ -n "$duplicates" ]]; then
        log_error "同一ファイルが複数タスクで指定されています:"
        echo "$duplicates"
        return 1
    fi

    log_success "計画は有効です: $name"
    echo ""
    echo "=== 計画サマリー ==="
    echo "名前: $name"
    echo "フェーズ数: $phases"
    jq -r '.phases[] | "  - \(.name) (\(.type)): \(.tasks // .groups | length) タスク/グループ"' "$plan_file"

    return 0
}

# 計画からClaude実行コマンドを生成
generate_commands() {
    local plan_file="$1"
    check_jq

    echo "# 生成されたコマンド"
    echo "# 計画: $(jq -r '.name' "$plan_file")"
    echo ""

    local worktree=$(jq -r '.worktree // "feature-plan"' "$plan_file")

    # 各フェーズを処理
    jq -c '.phases[]' "$plan_file" | while read -r phase; do
        local phase_name=$(echo "$phase" | jq -r '.name')
        local phase_type=$(echo "$phase" | jq -r '.type')
        local parallel=$(echo "$phase" | jq -r '.parallel // true')

        echo "# === Phase: $phase_name ($phase_type) ==="
        echo ""

        case "$phase_type" in
            investigate)
                echo "# 調査フェーズ - Task(Explore) を並列実行"
                echo "# worktree不要、masterを直接読む"
                echo ""
                echo "$phase" | jq -r '.tasks[] | "# Task(Explore): \(.prompt)"'
                echo ""
                ;;
            implement)
                echo "# 実装フェーズ - worktree内でファイル分割並列"
                echo "./scripts/parallel-run.sh start $worktree"
                echo ""

                # グループ順に処理
                echo "$phase" | jq -c '.groups[]' | while read -r group; do
                    local group_id=$(echo "$group" | jq -r '.id')
                    local depends=$(echo "$group" | jq -r '.depends_on // [] | join(", ")')

                    echo "# --- Group $group_id (依存: ${depends:-なし}) ---"
                    if [[ "$depends" != "" ]]; then
                        echo "# 前のグループ完了を待ってから実行"
                    fi
                    echo ""

                    echo "$group" | jq -r '.tasks[] | "# Task(general-purpose) @ worktrees/'"$worktree"': \(.file) - \(.prompt)"'
                    echo ""
                done
                ;;
            verify)
                echo "# 検証フェーズ - 直列実行"
                echo "$phase" | jq -r '.commands[]'
                echo ""
                echo "./scripts/parallel-run.sh finish $worktree"
                ;;
        esac
        echo ""
    done
}

# 計画例を出力
show_example() {
    cat << 'EOF'
{
  "name": "UI表示制御の統一",
  "worktree": "refactor-ui-visibility",
  "phases": [
    {
      "name": "調査",
      "type": "investigate",
      "parallel": true,
      "tasks": [
        {"id": "R1", "prompt": "現在のUI表示ロジックを調査。src/ui/, src/systems/ を確認", "agent": "Explore"},
        {"id": "R2", "prompt": "InputStateの使われ方を調査。src/components/input.rs を確認", "agent": "Explore"},
        {"id": "R3", "prompt": "機械UIの表示パターンを調査。src/ui/machine_ui.rs を確認", "agent": "Explore"}
      ]
    },
    {
      "name": "実装",
      "type": "implement",
      "parallel": true,
      "groups": [
        {
          "id": "A",
          "tasks": [
            {"id": "I1", "file": "src/ui/visibility.rs", "action": "create", "prompt": "UiVisibility型を定義。show/hide/toggleメソッドを持つ"},
            {"id": "I4", "file": "tests/ui_visibility_test.rs", "action": "create", "prompt": "UiVisibilityのテストを作成"}
          ]
        },
        {
          "id": "B",
          "depends_on": ["A"],
          "tasks": [
            {"id": "I2", "file": "src/systems/ui_visibility.rs", "action": "modify", "prompt": "UiVisibilityを使った表示システムを実装"},
            {"id": "I3", "file": "src/ui/machine_ui.rs", "action": "modify", "prompt": "機械UIをUiVisibilityで統合"}
          ]
        },
        {
          "id": "C",
          "depends_on": ["B"],
          "tasks": [
            {"id": "I5", "file": "src/ui/mod.rs", "action": "modify", "prompt": "pub mod visibility; を追加"}
          ]
        }
      ]
    },
    {
      "name": "検証",
      "type": "verify",
      "parallel": false,
      "commands": ["cargo build", "cargo test", "cargo clippy"]
    }
  ]
}
EOF
}

# 計画の状態表示
show_status() {
    if [[ ! -f "$PLAN_STATUS" ]]; then
        log_info "実行中の計画はありません"
        return 0
    fi

    check_jq

    echo ""
    echo "=== 計画実行状態 ==="
    jq -r '"計画: \(.name)\n現在のフェーズ: \(.current_phase)\n進捗: \(.completed_tasks)/\(.total_tasks)"' "$PLAN_STATUS"

    echo ""
    echo "=== 完了タスク ==="
    jq -r '.tasks[] | select(.status == "completed") | "  ✅ \(.id): \(.file // .prompt)"' "$PLAN_STATUS"

    echo ""
    echo "=== 実行中タスク ==="
    jq -r '.tasks[] | select(.status == "in_progress") | "  🔄 \(.id): \(.file // .prompt)"' "$PLAN_STATUS"

    echo ""
    echo "=== 待機中タスク ==="
    jq -r '.tasks[] | select(.status == "pending") | "  ⏳ \(.id): \(.file // .prompt)"' "$PLAN_STATUS"
}

# 計画を実行（ステータス初期化）
init_plan() {
    local plan_file="$1"
    check_jq

    if ! validate_plan "$plan_file"; then
        return 1
    fi

    # 計画をコピー
    cp "$plan_file" "$CURRENT_PLAN"

    # ステータス初期化
    local name=$(jq -r '.name' "$plan_file")
    local tasks=$(jq -c '[
        .phases[].tasks[]? | {id: .id, prompt: .prompt, status: "pending"},
        .phases[].groups[]?.tasks[]? | {id: .id, file: .file, prompt: .prompt, status: "pending"}
    ]' "$plan_file")
    local total=$(echo "$tasks" | jq 'length')

    jq -n \
        --arg name "$name" \
        --argjson tasks "$tasks" \
        --argjson total "$total" \
        '{
            name: $name,
            current_phase: "investigate",
            completed_tasks: 0,
            total_tasks: $total,
            tasks: $tasks
        }' > "$PLAN_STATUS"

    log_success "計画を初期化しました: $name"
    echo ""
    generate_commands "$plan_file"
}

# ヘルプ
show_help() {
    echo "並列計画の実行管理"
    echo ""
    echo "使い方: $0 <command> [args]"
    echo ""
    echo "コマンド:"
    echo "  validate <plan.json>  計画ファイルを検証"
    echo "  run <plan.json>       計画を初期化して実行コマンドを生成"
    echo "  commands <plan.json>  実行コマンドのみ生成"
    echo "  status                実行中計画の状態表示"
    echo "  example               計画JSONの例を出力"
    echo "  help                  このヘルプを表示"
    echo ""
    echo "ワークフロー:"
    echo "  1. 計画JSONを作成 (.claude/current-plan.json)"
    echo "  2. $0 validate で検証"
    echo "  3. $0 run で実行開始"
    echo "  4. 生成されたコマンドに従ってTask toolを実行"
    echo ""
    echo "計画形式: .claude/plan-template.md 参照"
}

# メイン
case "${1:-help}" in
    validate)
        if [[ -z "${2:-}" ]]; then
            log_error "計画ファイルを指定してください"
            exit 1
        fi
        validate_plan "$2"
        ;;
    run)
        if [[ -z "${2:-}" ]]; then
            log_error "計画ファイルを指定してください"
            exit 1
        fi
        init_plan "$2"
        ;;
    commands)
        if [[ -z "${2:-}" ]]; then
            log_error "計画ファイルを指定してください"
            exit 1
        fi
        generate_commands "$2"
        ;;
    status)
        show_status
        ;;
    example)
        show_example
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        log_error "不明なコマンド: $1"
        show_help
        exit 1
        ;;
esac
