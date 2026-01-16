#!/bin/bash
# architecture.mdとコードの整合性チェック
# コミット後に自動実行される

set -e
cd "$(dirname "$0")/.."

# 色定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=== Architecture Integrity Check ==="
echo ""

warnings=0
errors=0

# ID System
echo "### ID System ###"
item_id=$(grep -r 'ItemId' src --include='*.rs' 2>/dev/null | grep -v 'pub type\|//' | wc -l)
machine_type=$(grep -r 'MachineType::' src --include='*.rs' 2>/dev/null | wc -l)
machine_id=$(grep -r 'MachineId' src --include='*.rs' 2>/dev/null | grep -v 'pub type\|//' | wc -l)
recipe_id=$(grep -r 'RecipeId' src --include='*.rs' 2>/dev/null | grep -v 'pub type\|//' | wc -l)
ui_element_id=$(grep -r 'UIElementId' src --include='*.rs' 2>/dev/null | grep -v 'pub type\|//' | wc -l)

echo "ItemId usages: $item_id (expected: >400)"
if [ "$item_id" -lt 400 ]; then
  echo -e "${YELLOW}WARNING: ItemId usage below expected${NC}"
  warnings=$((warnings + 1))
fi

echo "MachineType usages: $machine_type (should be 0 after migration)"
if [ "$machine_type" -gt 0 ]; then
  echo -e "${YELLOW}INFO: MachineType still in use (migration pending)${NC}"
fi

echo "MachineId usages: $machine_id"
echo "RecipeId usages: $recipe_id"
echo "UIElementId usages: $ui_element_id"

# UI System
echo ""
echo "### UI System ###"
overlay=$(grep -r 'OverlayType' src --include='*.rs' 2>/dev/null | wc -l)
ui_element_registry=$(grep -r 'UIElementRegistry' src --include='*.rs' 2>/dev/null | wc -l)
echo "OverlayType: $overlay (0 = not implemented, documented as future)"
echo "UIElementRegistry: $ui_element_registry"

# Module Structure
echo ""
echo "### Module Structure ###"
wasm_files=$(ls src/modding/wasm/*.rs 2>/dev/null | wc -l)
handlers_files=$(ls src/modding/handlers/*.rs 2>/dev/null | wc -l)
server_files=$(ls src/modding/server/*.rs 2>/dev/null | wc -l)
machines_comp_files=$(ls src/components/machines/*.rs 2>/dev/null | wc -l)
machines_generic_files=$(ls src/machines/generic/*.rs 2>/dev/null | wc -l)

echo "modding/wasm: $wasm_files files (expected: 8)"
echo "modding/handlers: $handlers_files files (expected: 8)"
echo "modding/server: $server_files files (expected: 7)"
echo "components/machines: $machines_comp_files files (expected: 7)"
echo "machines/generic: $machines_generic_files files (expected: 9)"

# Legacy patterns check
echo ""
echo "### Legacy Pattern Check ###"
legacy_inventory=$(grep -r 'Res<PlayerInventory>' src --include='*.rs' 2>/dev/null | wc -l)
if [ "$legacy_inventory" -gt 0 ]; then
  echo -e "${RED}ERROR: Legacy Res<PlayerInventory> found: $legacy_inventory${NC}"
  errors=$((errors + 1))
else
  echo -e "${GREEN}OK: No legacy Res<PlayerInventory>${NC}"
fi

block_type=$(grep -r 'BlockType' src --include='*.rs' 2>/dev/null | grep -v '//' | wc -l)
if [ "$block_type" -gt 0 ]; then
  echo -e "${YELLOW}WARNING: BlockType references found: $block_type (should be in comments only)${NC}"
  warnings=$((warnings + 1))
else
  echo -e "${GREEN}OK: No BlockType references${NC}"
fi

# Summary
echo ""
echo "=== Summary ==="
if [ "$errors" -gt 0 ]; then
  echo -e "${RED}Found $errors error(s) - architecture violations!${NC}"
  exit 1
elif [ "$warnings" -gt 0 ]; then
  echo -e "${YELLOW}Found $warnings warning(s) - review recommended${NC}"
  exit 0
else
  echo -e "${GREEN}All checks passed!${NC}"
  exit 0
fi
