#!/usr/bin/env python3
"""
Test Pattern Auto-Updater

このスクリプトはゲームコードを解析して、E2Eテストパターンを自動更新します。
新しいブロックタイプ、機械、UIが追加されたときに実行してください。

使い方:
    python3 tools/update_test_patterns.py
"""

import re
import os
from pathlib import Path
from datetime import datetime

# プロジェクトルート
PROJECT_ROOT = Path(__file__).parent.parent
MAIN_RS = PROJECT_ROOT / "src" / "main.rs"
BLOCK_TYPE_RS = PROJECT_ROOT / "src" / "block_type.rs"
GAME_SPEC_RS = PROJECT_ROOT / "src" / "game_spec.rs"
PATTERNS_FILE = PROJECT_ROOT / "tests" / "e2e_patterns" / "test_patterns.toml"


def extract_block_types():
    """block_type.rsからBlockType enumを抽出"""
    if not BLOCK_TYPE_RS.exists():
        print(f"Warning: {BLOCK_TYPE_RS} not found")
        return []

    content = BLOCK_TYPE_RS.read_text()

    # enum BlockType { ... } を探す
    enum_match = re.search(r'enum BlockType\s*\{([^}]+)\}', content, re.DOTALL)
    if not enum_match:
        return []

    enum_body = enum_match.group(1)

    # 各バリアントを抽出
    variants = re.findall(r'(\w+)(?:\s*=\s*\d+)?(?:\s*,)?', enum_body)

    # フィルタリング（コメント行などを除外）
    block_types = [v for v in variants if v and not v.startswith('//')]

    return block_types


def extract_initial_equipment():
    """game_spec.rsから初期装備を抽出"""
    if not GAME_SPEC_RS.exists():
        return []

    content = GAME_SPEC_RS.read_text()

    # INITIAL_EQUIPMENT配列を探す
    match = re.search(r'INITIAL_EQUIPMENT.*?=.*?\[([^\]]+)\]', content, re.DOTALL)
    if not match:
        return []

    items = re.findall(r'\(BlockType::(\w+),\s*(\d+)\)', match.group(1))
    return items


def extract_machines():
    """main.rsから機械タイプを抽出"""
    if not MAIN_RS.exists():
        return []

    content = MAIN_RS.read_text()

    machines = []

    # struct Miner, struct Furnace, etc. を探す
    for match in re.finditer(r'struct\s+(Miner|Furnace|Crusher|Conveyor)', content):
        machines.append(match.group(1))

    return list(set(machines))


def extract_ui_keys():
    """main.rsからUI関連のキーバインドを抽出"""
    if not MAIN_RS.exists():
        return {}

    content = MAIN_RS.read_text()

    keys = {}

    # KeyCode::E => inventory, etc.
    key_patterns = [
        (r'KeyCode::E.*?inventory', 'inventory', 'E'),
        (r'KeyCode::T.*?command', 'command', 'T'),
        (r'KeyCode::F3.*?debug', 'debug', 'F3'),
        (r'KeyCode::Escape.*?close', 'close_ui', 'Escape'),
    ]

    for pattern, name, key in key_patterns:
        if re.search(pattern, content, re.IGNORECASE | re.DOTALL):
            keys[name] = key

    return keys


def generate_block_placement_tests(block_types, initial_equipment):
    """ブロック設置テストを生成"""
    tests = []

    # 初期装備をホットバースロットにマッピング
    slot_map = {}
    for i, (block_type, count) in enumerate(initial_equipment):
        slot_map[block_type] = i + 1  # 1-indexed

    placeable_blocks = [
        'MinerBlock', 'ConveyorBlock', 'FurnaceBlock', 'CrusherBlock',
        'Stone', 'Dirt', 'Wood', 'Brick'
    ]

    for block_type in block_types:
        if block_type in placeable_blocks:
            slot = slot_map.get(block_type, 0)
            test = f'''
[[block_placement]]
name = "{block_type.lower()}_placement"
description = "{block_type}を設置"
block_type = "{block_type}"
hotbar_slot = {slot}
screenshot_check = true
'''
            tests.append(test)

    return tests


def generate_ui_tests(ui_keys):
    """UI表示テストを生成"""
    tests = []

    ui_definitions = [
        ('inventory', 'インベントリ開閉', 'E', True),
        ('command', 'コマンド入力', 'T', False),
        ('debug', 'デバッグHUD', 'F3', False),
    ]

    for name, desc, key, blocks_movement in ui_definitions:
        test = f'''
[[ui_display]]
name = "{name}_toggle"
description = "{desc}"
key = "{key}"
player_movement_blocked = {str(blocks_movement).lower()}
'''
        tests.append(test)

    return tests


def generate_machine_tests(machines):
    """機械動作テストを生成"""
    tests = []

    machine_behaviors = {
        'Miner': ('MinerBlock', 'on_ore', 'produces_ore', 2.0),
        'Furnace': ('FurnaceBlock', 'any', 'smelts_items', 5.0),
        'Crusher': ('CrusherBlock', 'any', 'crushes_items', 3.0),
        'Conveyor': ('ConveyorBlock', 'any', 'transports_items', 0.5),
    }

    for machine in machines:
        if machine in machine_behaviors:
            block, placement, behavior, interval = machine_behaviors[machine]
            test = f'''
[[machine_behavior]]
name = "{machine.lower()}_operation"
description = "{machine}の動作確認"
machine = "{block}"
placement = "{placement}"
expected = "{behavior}"
interval_seconds = {interval}
'''
            tests.append(test)

    return tests


def generate_conveyor_tests():
    """コンベアシステムテストを生成"""
    return '''
# コンベアシステムテスト（自動生成）

[[conveyor_system]]
name = "conveyor_straight"
description = "直線コンベア"
shape = "Straight"
screenshot_check = true

[[conveyor_system]]
name = "conveyor_corner_left"
description = "左コーナーコンベア"
shape = "CornerLeft"
precondition = "side_input_from_left"

[[conveyor_system]]
name = "conveyor_corner_right"
description = "右コーナーコンベア"
shape = "CornerRight"
precondition = "side_input_from_right"

[[conveyor_system]]
name = "conveyor_t_junction"
description = "T字路コンベア"
shape = "TJunction"
precondition = "both_side_inputs"

[[conveyor_system]]
name = "conveyor_splitter"
description = "スプリッターコンベア"
shape = "Splitter"
precondition = "multiple_outputs"
'''


def update_patterns_file():
    """テストパターンファイルを更新"""

    print("=== Test Pattern Auto-Updater ===")
    print(f"Project root: {PROJECT_ROOT}")
    print()

    # 各種情報を抽出
    print("Extracting block types...")
    block_types = extract_block_types()
    print(f"  Found {len(block_types)} block types")

    print("Extracting initial equipment...")
    initial_equipment = extract_initial_equipment()
    print(f"  Found {len(initial_equipment)} initial items")

    print("Extracting machines...")
    machines = extract_machines()
    print(f"  Found {len(machines)} machines")

    print("Extracting UI keys...")
    ui_keys = extract_ui_keys()
    print(f"  Found {len(ui_keys)} UI bindings")

    print()

    # テストパターンを生成
    print("Generating test patterns...")

    header = f'''# E2E Test Patterns
# Auto-generated by tools/update_test_patterns.py
# Last updated: {datetime.now().isoformat()}
#
# このファイルを手動で編集しても構いませんが、
# update_test_patterns.py を実行すると上書きされます。
# カスタムテストは test_patterns_custom.toml に追加してください。

[metadata]
version = "1.0"
game_version = "0.1.0"
auto_generated = true
'''

    block_placement_tests = generate_block_placement_tests(block_types, initial_equipment)
    ui_tests = generate_ui_tests(ui_keys)
    machine_tests = generate_machine_tests(machines)
    conveyor_tests = generate_conveyor_tests()

    # ファイルに書き込み
    content = header
    content += "\n# =====================================================\n"
    content += "# ブロック設置テスト（自動生成）\n"
    content += "# =====================================================\n"
    content += "".join(block_placement_tests)

    content += "\n# =====================================================\n"
    content += "# UI表示テスト（自動生成）\n"
    content += "# =====================================================\n"
    content += "".join(ui_tests)

    content += "\n# =====================================================\n"
    content += "# 機械動作テスト（自動生成）\n"
    content += "# =====================================================\n"
    content += "".join(machine_tests)

    content += "\n# =====================================================\n"
    content += conveyor_tests

    # ホットバーテスト
    content += '''
# =====================================================
# ホットバー選択テスト（自動生成）
# =====================================================
'''
    for i in range(1, 10):
        content += f'''
[[hotbar_selection]]
name = "hotbar_slot_{i}"
description = "ホットバースロット{i}を選択"
key = "{i}"
'''

    # プレイヤー操作テスト
    content += '''
# =====================================================
# プレイヤー操作テスト（自動生成）
# =====================================================

[[player_control]]
name = "wasd_movement"
description = "WASD移動"
keys = ["W", "A", "S", "D"]
blocked_during = ["inventory_open", "command_input"]

[[player_control]]
name = "space_jump"
description = "スペースでジャンプ"
key = "Space"

[[player_control]]
name = "shift_descend"
description = "Shiftで降下"
key = "Shift"

[[player_control]]
name = "mouse_look"
description = "マウスで視点操作"
requires = "pointer_lock"
'''

    # ファイル書き込み
    PATTERNS_FILE.parent.mkdir(parents=True, exist_ok=True)
    PATTERNS_FILE.write_text(content)

    print(f"Written to: {PATTERNS_FILE}")
    print()
    print("=== Summary ===")
    print(f"Block types: {len(block_types)}")
    print(f"Machines: {machines}")
    print(f"UI keys: {ui_keys}")
    print()
    print("Done!")


if __name__ == "__main__":
    update_patterns_file()
