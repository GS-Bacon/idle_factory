#!/bin/bash
# 新アーキテクチャ移行状況チェック
#
# 使い方:
#   ./scripts/migration-status.sh        # 全体サマリー
#   ./scripts/migration-status.sh -v     # 詳細表示
#   ./scripts/migration-status.sh -ci    # CI用（しきい値超えでexit 1）

set -euo pipefail
cd "$(dirname "$0")/.."

VERBOSE=false
CI_MODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose) VERBOSE=true; shift ;;
        -ci|--ci) CI_MODE=true; shift ;;
        *) shift ;;
    esac
done

echo "=============================================="
echo " 新アーキテクチャ移行状況"
echo "=============================================="
echo ""

# ---------------------------------------------
# 1. BlockType → ItemId 移行
# ---------------------------------------------
BLOCKTYPE_COUNT=$(grep -r 'BlockType' src --include='*.rs' 2>/dev/null | grep -v '//.*BlockType' | wc -l | tr -d '[:space:]' || echo 0)
ITEMID_COUNT=$(grep -r 'ItemId' src --include='*.rs' 2>/dev/null | wc -l | tr -d '[:space:]' || echo 0)

echo "## D.2: 動的ID移行"
echo ""
echo "  BlockType 直接参照: $BLOCKTYPE_COUNT 箇所"
echo "  ItemId 使用:        $ITEMID_COUNT 箇所"

if [ "$BLOCKTYPE_COUNT" -gt 0 ]; then
    echo "  ステータス: 🔄 移行中"
else
    echo "  ステータス: ✅ 完了"
fi

if $VERBOSE; then
    echo ""
    echo "  --- BlockType使用ファイル (上位10) ---"
    grep -r 'BlockType' src --include='*.rs' -l 2>/dev/null | \
        xargs -I {} sh -c 'echo "$(grep -c BlockType {} 2>/dev/null || echo 0) {}"' | \
        sort -rn | head -10 | sed 's/^/    /'
fi
echo ""

# ---------------------------------------------
# 2. セーブデータ形式
# ---------------------------------------------
ENUM_SAVE=$(grep -r 'BlockTypeSave' src --include='*.rs' 2>/dev/null | wc -l | tr -d '[:space:]' || echo 0)
STRING_SAVE=$(grep -r 'block_id.*String\|id.*String' src/save --include='*.rs' 2>/dev/null | wc -l | tr -d '[:space:]' || echo 0)

echo "## セーブデータ形式"
echo ""
echo "  enum形式 (BlockTypeSave): $ENUM_SAVE 箇所"
echo "  文字列ID形式:             $STRING_SAVE 箇所"

if [ "$ENUM_SAVE" -gt 0 ]; then
    echo "  ステータス: 🔨 旧形式のまま"
else
    echo "  ステータス: ✅ 文字列ID化完了"
fi
echo ""

# ---------------------------------------------
# 3. 本体アイテムのMod化
# ---------------------------------------------
HARDCODED_ITEMS=$(grep -c 'BlockType::' src/game_spec/registry.rs 2>/dev/null || echo 0)
MOD_ITEMS=$(find mods -name 'items.toml' 2>/dev/null | xargs cat 2>/dev/null | grep -c '\[\[item\]\]' | tr -d '[:space:]' || echo 0)
[ -z "$MOD_ITEMS" ] && MOD_ITEMS=0

echo "## 本体アイテムのMod化"
echo ""
echo "  Rustハードコード: $HARDCODED_ITEMS 個"
echo "  TOML定義:         $MOD_ITEMS 個"

if [ "$HARDCODED_ITEMS" -gt 0 ] && [ "$MOD_ITEMS" -eq 0 ]; then
    echo "  ステータス: ❌ 未着手"
elif [ "$HARDCODED_ITEMS" -gt 0 ]; then
    echo "  ステータス: 🔄 移行中"
else
    echo "  ステータス: ✅ 完了"
fi
echo ""

# ---------------------------------------------
# 4. イベントシステム
# ---------------------------------------------
EVENT_DEFS=$(grep -r '#\[derive(Event)\]' src --include='*.rs' 2>/dev/null | wc -l | tr -d '[:space:]' || echo 0)
# Observer or EventReader for game events
OBSERVER_USAGE=$(grep -r 'add_observer\|Trigger<' src --include='*.rs' 2>/dev/null | wc -l | tr -d '[:space:]' || echo 0)
[ -z "$OBSERVER_USAGE" ] && OBSERVER_USAGE=0
# Count EventReader usage for game events (MachineCompleted, ItemDelivered, etc.)
EVENT_READER_USAGE=$(grep -r 'EventReader<.*\(MachineCompleted\|MachineStarted\|ItemDelivered\|BlockPlaced\|MachineSpawned\|ConveyorTransfer\)>' src --include='*.rs' 2>/dev/null | wc -l | tr -d '[:space:]' || echo 0)
[ -z "$EVENT_READER_USAGE" ] && EVENT_READER_USAGE=0
TOTAL_EVENT_USAGE=$((OBSERVER_USAGE + EVENT_READER_USAGE))

echo "## D.1: イベントシステム"
echo ""
echo "  イベント定義:      $EVENT_DEFS 個"
echo "  EventReader使用:   $EVENT_READER_USAGE 箇所"
echo "  Observer使用:      $OBSERVER_USAGE 箇所"

if [ "$EVENT_DEFS" -gt 0 ] && [ "$TOTAL_EVENT_USAGE" -eq 0 ]; then
    echo "  ステータス: 🔨 基盤のみ（イベント購読未実装）"
elif [ "$TOTAL_EVENT_USAGE" -gt 0 ]; then
    echo "  ステータス: ✅ イベント購読実装済み"
else
    echo "  ステータス: ❌ 未着手"
fi
echo ""

# ---------------------------------------------
# 5. レガシーコード
# ---------------------------------------------
LEGACY_MACHINES=$(grep -r 'pub struct Miner\|pub struct Furnace\|pub struct Crusher' src/components --include='*.rs' 2>/dev/null | wc -l | tr -d '[:space:]' || echo 0)
[ -z "$LEGACY_MACHINES" ] && LEGACY_MACHINES=0
LEGACY_INTERACTING=$(grep -r 'InteractingFurnace\|InteractingCrusher\|InteractingMiner' src --include='*.rs' 2>/dev/null | wc -l | tr -d '[:space:]' || echo 0)
[ -z "$LEGACY_INTERACTING" ] && LEGACY_INTERACTING=0

echo "## レガシーコード"
echo ""
echo "  旧機械コンポーネント: $LEGACY_MACHINES 個"
echo "  旧Interacting*:       $LEGACY_INTERACTING 箇所"

if [ "$LEGACY_MACHINES" -eq 0 ] && [ "$LEGACY_INTERACTING" -eq 0 ]; then
    echo "  ステータス: ✅ 削除済み"
else
    echo "  ステータス: 🔄 削除中"
fi
echo ""

# ---------------------------------------------
# サマリー
# ---------------------------------------------
echo "=============================================="
echo " サマリー"
echo "=============================================="
echo ""

ISSUES=0

if [ "$BLOCKTYPE_COUNT" -gt 100 ]; then
    echo "  ⚠️  BlockType参照が多い ($BLOCKTYPE_COUNT箇所)"
    ISSUES=$((ISSUES + 1))
fi

if [ "$ENUM_SAVE" -gt 0 ]; then
    echo "  ⚠️  セーブ形式がenum"
    ISSUES=$((ISSUES + 1))
fi

if [ "$HARDCODED_ITEMS" -gt 0 ] && [ "$MOD_ITEMS" -eq 0 ]; then
    echo "  ⚠️  本体アイテムがMod化されていない"
    ISSUES=$((ISSUES + 1))
fi

if [ "$TOTAL_EVENT_USAGE" -eq 0 ] && [ "$EVENT_DEFS" -gt 0 ]; then
    echo "  ⚠️  イベント定義はあるが購読未使用"
    ISSUES=$((ISSUES + 1))
fi

if [ "$LEGACY_MACHINES" -gt 0 ]; then
    echo "  ⚠️  レガシー機械コンポーネントが残存"
    ISSUES=$((ISSUES + 1))
fi

if [ "$ISSUES" -eq 0 ]; then
    echo "  ✅ 全ての移行が完了しています"
fi

echo ""
echo "  問題数: $ISSUES"
echo ""

# CI用: しきい値チェック
if $CI_MODE && [ "$ISSUES" -gt 0 ]; then
    echo "CI: 未完了の移行があります"
    exit 1
fi
