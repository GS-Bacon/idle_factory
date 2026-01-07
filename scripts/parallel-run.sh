#!/bin/bash
# parallel-run.sh - タスクの並列実行管理
#
# 使い方:
#   ./scripts/parallel-run.sh list          # 並列実行可能なタスクを表示
#   ./scripts/parallel-run.sh start <id>    # タスク開始（worktree作成）
#   ./scripts/parallel-run.sh status        # 実行中タスクの状態
#   ./scripts/parallel-run.sh finish <id>   # タスク完了（マージ）
#   ./scripts/parallel-run.sh abort <id>    # タスク中止（worktree削除）
#   ./scripts/parallel-run.sh auto          # 並列実行可能なタスクを自動開始

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TASKS_FILE="$PROJECT_ROOT/.claude/parallel-tasks.json"
WORKTREES_DIR="/mnt/build/worktrees"  # 300GB ディスク

# 色定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_success() { echo -e "${GREEN}[OK]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

# jqがあるか確認
check_jq() {
    if ! command -v jq &> /dev/null; then
        log_error "jq がインストールされていません: sudo apt-get install jq"
        exit 1
    fi
}

# ディスク容量チェック
# 戻り値: 0=OK, 1=容量不足
WORKTREE_SIZE_GB=7  # worktree 1つあたりの推定サイズ（GB）
MIN_FREE_GB=10      # 最低限確保する空き容量（GB）

check_disk_space() {
    local num_worktrees="${1:-1}"

    # 現在の空き容量を取得（GB単位）- worktreeディスクを確認
    local free_gb=$(df -BG "$WORKTREES_DIR" | tail -1 | awk '{print $4}' | sed 's/G//')

    # 必要な容量を計算
    local needed_gb=$((num_worktrees * WORKTREE_SIZE_GB + MIN_FREE_GB))

    log_info "ディスク容量チェック:"
    log_info "  空き容量: ${free_gb}GB"
    log_info "  必要容量: ${needed_gb}GB (worktree ${num_worktrees}個 × ${WORKTREE_SIZE_GB}GB + 予備${MIN_FREE_GB}GB)"

    if [[ $free_gb -lt $needed_gb ]]; then
        log_error "容量不足！${needed_gb}GB必要ですが、${free_gb}GBしかありません"
        log_info "対策: ./scripts/parallel-run.sh cleanup で古いworktreeを削除"
        return 1
    fi

    log_success "容量OK (残り: $((free_gb - needed_gb))GB の余裕)"
    return 0
}

# 放置worktreeの警告・クリーンアップ
cleanup_worktrees() {
    log_info "放置されたworktreeを確認中..."

    local cleaned=0
    for wt in "$WORKTREES_DIR"/*/; do
        [[ -d "$wt" ]] || continue
        local task_id=$(basename "$wt")
        local status=$(jq -r --arg id "$task_id" '.tasks[] | select(.id == $id) | .status' "$TASKS_FILE" 2>/dev/null || echo "unknown")

        if [[ "$status" == "completed" ]] || [[ "$status" == "unknown" ]]; then
            log_warn "放置worktree発見: $task_id (status: $status)"
            log_info "削除中: $wt"
            git worktree remove --force "$wt" 2>/dev/null || rm -rf "$wt"
            ((cleaned++)) || true
        fi
    done

    if [[ $cleaned -gt 0 ]]; then
        log_success "${cleaned}個のworktreeを削除しました"
        # 削除後の容量を表示
        local free_gb=$(df -BG "$WORKTREES_DIR" | tail -1 | awk '{print $4}' | sed 's/G//')
        log_info "現在の空き容量: ${free_gb}GB"
    else
        log_info "クリーンアップ対象なし"
    fi
}

# タスク一覧表示
list_tasks() {
    check_jq
    echo ""
    echo "=== 並列実行可能なタスク ==="
    echo ""

    # ステータス別に表示
    echo -e "${GREEN}▼ 実行可能（pending）${NC}"
    jq -r '.tasks[] | select(.status == "pending") | "  [\(.parallel_group)] \(.id): \(.name)"' "$TASKS_FILE"

    echo ""
    echo -e "${YELLOW}▼ 実行中（in_progress）${NC}"
    jq -r '.tasks[] | select(.status == "in_progress") | "  [\(.parallel_group)] \(.id): \(.name) → \(.branch)"' "$TASKS_FILE"

    echo ""
    echo -e "${BLUE}▼ 完了（completed）${NC}"
    jq -r '.tasks[] | select(.status == "completed") | "  [\(.parallel_group)] \(.id): \(.name)"' "$TASKS_FILE"

    echo ""
    echo "=== 並列グループ ==="
    jq -r '.parallel_groups | to_entries[] | "  \(.key): \(.value.description) (max: \(.value.max_concurrent))"' "$TASKS_FILE"
}

# ファイル衝突チェック
check_file_conflicts() {
    local task_id="$1"
    check_jq

    # 対象タスクのファイルパターンを取得
    local task_files=$(jq -r --arg id "$task_id" '.tasks[] | select(.id == $id) | .files[]' "$TASKS_FILE" 2>/dev/null)

    # 実行中タスクのファイルパターンと比較
    local running_files=$(jq -r '.tasks[] | select(.status == "in_progress") | .files[]' "$TASKS_FILE" 2>/dev/null)

    # 簡易衝突チェック（完全なglob展開はしない）
    for tf in $task_files; do
        for rf in $running_files; do
            # 同じディレクトリを触っていたら警告
            local tf_dir=$(dirname "$tf")
            local rf_dir=$(dirname "$rf")
            if [[ "$tf_dir" == "$rf_dir" ]]; then
                log_warn "ファイル衝突の可能性: $tf vs $rf"
                return 1
            fi
        done
    done
    return 0
}

# タスク開始
start_task() {
    local task_id="$1"
    check_jq

    # ディスク容量チェック
    if ! check_disk_space 1; then
        exit 1
    fi

    # タスク存在確認
    local task=$(jq -r --arg id "$task_id" '.tasks[] | select(.id == $id)' "$TASKS_FILE")
    if [[ -z "$task" ]]; then
        log_error "タスク '$task_id' が見つかりません"
        exit 1
    fi

    # ステータス確認
    local status=$(echo "$task" | jq -r '.status')
    if [[ "$status" != "pending" ]]; then
        log_error "タスクは既に $status です"
        exit 1
    fi

    # 依存関係チェック
    local deps=$(echo "$task" | jq -r '.depends_on[]' 2>/dev/null || true)
    for dep in $deps; do
        local dep_status=$(jq -r --arg id "$dep" '.tasks[] | select(.id == $id) | .status' "$TASKS_FILE")
        if [[ "$dep_status" != "completed" ]]; then
            log_error "依存タスク '$dep' が未完了です（$dep_status）"
            exit 1
        fi
    done

    # ファイル衝突チェック（情報表示のみ、ブロックしない）
    if ! check_file_conflicts "$task_id"; then
        log_info "マージ時にコンフリクトの可能性あり（worktree作業中は問題なし）"
    fi

    local branch=$(echo "$task" | jq -r '.branch')
    local name=$(echo "$task" | jq -r '.name')

    # worktreeディレクトリ作成
    mkdir -p "$WORKTREES_DIR"
    local worktree_path="$WORKTREES_DIR/$task_id"

    # ブランチ作成 & worktree追加
    log_info "ブランチ '$branch' を作成..."
    cd "$PROJECT_ROOT"
    git branch "$branch" 2>/dev/null || true

    log_info "worktree を作成: $worktree_path"
    git worktree add "$worktree_path" "$branch"

    # ステータス更新
    local tmp=$(mktemp)
    jq --arg id "$task_id" '(.tasks[] | select(.id == $id) | .status) = "in_progress"' "$TASKS_FILE" > "$tmp"
    mv "$tmp" "$TASKS_FILE"

    log_success "タスク '$name' を開始しました"
    echo ""
    echo "作業ディレクトリ: $worktree_path"
    echo "ブランチ: $branch"
    echo ""
    echo "作業完了後: ./scripts/parallel-run.sh finish $task_id"
}

# 実行中タスクの状態表示
show_status() {
    check_jq

    echo ""
    echo "=== Git Worktrees ==="
    git worktree list

    echo ""
    echo "=== 実行中タスク ==="
    jq -r '.tasks[] | select(.status == "in_progress") | "\(.id): \(.name)\n  Branch: \(.branch)\n  Files: \(.files | join(", "))\n"' "$TASKS_FILE"
}

# タスク完了（マージ）
finish_task() {
    local task_id="$1"
    check_jq

    local task=$(jq -r --arg id "$task_id" '.tasks[] | select(.id == $id)' "$TASKS_FILE")
    if [[ -z "$task" ]]; then
        log_error "タスク '$task_id' が見つかりません"
        exit 1
    fi

    local status=$(echo "$task" | jq -r '.status')
    if [[ "$status" != "in_progress" ]]; then
        log_error "タスクは $status です（in_progress でないとfinishできません）"
        exit 1
    fi

    local branch=$(echo "$task" | jq -r '.branch')
    local name=$(echo "$task" | jq -r '.name')
    local worktree_path="$WORKTREES_DIR/$task_id"

    cd "$PROJECT_ROOT"

    # worktree内の変更をコミット確認
    if [[ -d "$worktree_path" ]]; then
        cd "$worktree_path"
        if [[ -n $(git status --porcelain) ]]; then
            log_warn "未コミットの変更があります"
            git status --short
            read -p "コミットせずに続行しますか？ [y/N] " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                exit 1
            fi
        fi
        cd "$PROJECT_ROOT"
    fi

    # 重複コミット検出
    cd "$worktree_path" 2>/dev/null || cd "$PROJECT_ROOT"
    local last_commit_msg=$(git log -1 --format="%s" "$branch" 2>/dev/null)
    cd "$PROJECT_ROOT"

    if [[ -n "$last_commit_msg" ]]; then
        local duplicate=$(git log master --oneline -20 --format="%s" | grep -F "$last_commit_msg" || true)
        if [[ -n "$duplicate" ]]; then
            log_warn "同名コミットを検出: '$last_commit_msg'"
            log_warn "重複コミットの可能性があります。並列作業で同じ変更が行われた可能性があります。"
            read -p "続行しますか？ [y/N] " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log_info "中止しました"
                exit 1
            fi
        fi
    fi

    # masterにマージ
    log_info "master に '$branch' をマージ..."
    git checkout master
    git merge "$branch" --no-edit

    # worktree削除
    log_info "worktree を削除..."
    git worktree remove "$worktree_path" --force 2>/dev/null || true

    # ブランチ削除
    git branch -d "$branch" 2>/dev/null || true

    # ステータス更新
    local tmp=$(mktemp)
    jq --arg id "$task_id" '(.tasks[] | select(.id == $id) | .status) = "completed"' "$TASKS_FILE" > "$tmp"
    mv "$tmp" "$TASKS_FILE"

    log_success "タスク '$name' を完了しました"
}

# タスク中止
abort_task() {
    local task_id="$1"
    check_jq

    local task=$(jq -r --arg id "$task_id" '.tasks[] | select(.id == $id)' "$TASKS_FILE")
    if [[ -z "$task" ]]; then
        log_error "タスク '$task_id' が見つかりません"
        exit 1
    fi

    local branch=$(echo "$task" | jq -r '.branch')
    local name=$(echo "$task" | jq -r '.name')
    local worktree_path="$WORKTREES_DIR/$task_id"

    cd "$PROJECT_ROOT"

    # worktree削除
    log_info "worktree を削除..."
    git worktree remove "$worktree_path" --force 2>/dev/null || true

    # ブランチ削除
    git branch -D "$branch" 2>/dev/null || true

    # ステータスをpendingに戻す
    local tmp=$(mktemp)
    jq --arg id "$task_id" '(.tasks[] | select(.id == $id) | .status) = "pending"' "$TASKS_FILE" > "$tmp"
    mv "$tmp" "$TASKS_FILE"

    log_warn "タスク '$name' を中止しました"
}

# タスク追加
add_task() {
    check_jq

    echo "新規タスク追加"
    read -p "ID (例: fix-ui-bug): " task_id
    read -p "名前 (例: UIバグ修正): " task_name
    read -p "説明: " task_desc
    read -p "並列グループ (ui/logistics/machines/core): " parallel_group
    read -p "ブランチ名 (例: fix/ui-bug): " branch
    read -p "関連ファイル (カンマ区切り, 例: src/ui/*.rs,src/setup/ui/*.rs): " files_str
    read -p "依存タスク (カンマ区切り, 空でOK): " deps_str

    # ファイルを配列に変換
    IFS=',' read -ra files_arr <<< "$files_str"
    files_json=$(printf '%s\n' "${files_arr[@]}" | jq -R . | jq -s .)

    # 依存を配列に変換
    if [[ -n "$deps_str" ]]; then
        IFS=',' read -ra deps_arr <<< "$deps_str"
        deps_json=$(printf '%s\n' "${deps_arr[@]}" | jq -R . | jq -s .)
    else
        deps_json="[]"
    fi

    # タスク追加
    local tmp=$(mktemp)
    jq --arg id "$task_id" \
       --arg name "$task_name" \
       --arg desc "$task_desc" \
       --arg group "$parallel_group" \
       --arg branch "$branch" \
       --argjson files "$files_json" \
       --argjson deps "$deps_json" \
       '.tasks += [{
         id: $id,
         name: $name,
         description: $desc,
         parallel_group: $group,
         branch: $branch,
         files: $files,
         depends_on: $deps,
         status: "pending"
       }]' "$TASKS_FILE" > "$tmp"
    mv "$tmp" "$TASKS_FILE"

    log_success "タスク '$task_name' を追加しました"
}

# ヘルプ
show_help() {
    echo "並列タスク実行管理"
    echo ""
    echo "使い方: $0 <command> [args]"
    echo ""
    echo "コマンド:"
    echo "  list              並列実行可能なタスクを表示"
    echo "  start <id>        タスク開始（worktree作成）"
    echo "  status            実行中タスクの状態"
    echo "  finish <id>       タスク完了（masterにマージ）"
    echo "  abort <id>        タスク中止（worktree削除）"
    echo "  add               新規タスク追加（対話式）"
    echo "  cleanup           放置worktreeを削除"
    echo "  check [N]         N個のworktree用の容量があるか確認"
    echo "  help              このヘルプを表示"
    echo ""
    echo "例:"
    echo "  $0 list"
    echo "  $0 start fix-ui-bug"
    echo "  $0 finish fix-ui-bug"
}

# メイン
case "${1:-help}" in
    list)
        list_tasks
        ;;
    start)
        if [[ -z "${2:-}" ]]; then
            log_error "タスクIDを指定してください"
            exit 1
        fi
        start_task "$2"
        ;;
    status)
        show_status
        ;;
    finish)
        if [[ -z "${2:-}" ]]; then
            log_error "タスクIDを指定してください"
            exit 1
        fi
        finish_task "$2"
        ;;
    abort)
        if [[ -z "${2:-}" ]]; then
            log_error "タスクIDを指定してください"
            exit 1
        fi
        abort_task "$2"
        ;;
    add)
        add_task
        ;;
    cleanup)
        check_jq
        cleanup_worktrees
        ;;
    check)
        check_disk_space "${2:-1}"
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
