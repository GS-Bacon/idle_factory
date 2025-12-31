# コーディングルール

## 命名規則

| 対象 | 規則 | 例 |
|------|------|-----|
| 構造体 | PascalCase | `PlayerCamera`, `ChunkData` |
| 関数 | snake_case | `setup_player`, `update_hotbar_ui` |
| 定数 | SCREAMING_SNAKE | `BLOCK_SIZE`, `HOTBAR_SLOTS` |
| モジュール | snake_case | `block_type`, `inventory_ui` |
| コンポーネント | PascalCase、名詞 | `Player`, `Furnace`, `HotbarSlot` |
| システム | snake_case、動詞 | `player_move`, `furnace_smelting` |

## Bevyパターン

### コンポーネント定義

```rust
#[derive(Component, Default)]
pub struct Furnace {
    pub input: Option<BlockType>,
    pub fuel: u32,
    pub output: Option<BlockType>,
    pub progress: f32,
}
```

### システム定義

```rust
pub fn furnace_smelting(
    time: Res<Time>,
    mut query: Query<&mut Furnace>,
) {
    for mut furnace in query.iter_mut() {
        // 処理
    }
}
```

### リソース

```rust
#[derive(Resource, Default)]
pub struct Inventory {
    pub slots: [Option<(BlockType, u32)>; NUM_SLOTS],
    pub selected_slot: usize,
}
```

## エラーハンドリング

```rust
// 良い例: early return
let Some(chunk) = world.chunks.get(&coord) else {
    return;
};

// 避ける: 深いネスト
if let Some(chunk) = world.chunks.get(&coord) {
    if let Some(block) = chunk.get_block(pos) {
        // ...
    }
}
```

## ログ出力

```rust
use bevy::log::{info, warn, error};

// イベントログ（BLOCK, MACHINE, QUEST）
info!("BLOCK place: {:?} at {:?}", block_type, pos);
info!("MACHINE miner placed at {:?}", pos);
info!("QUEST delivered: {:?} x{}", item, count);

// デバッグ用（本番では削除）
#[cfg(debug_assertions)]
bevy::log::debug!("Debug info: {:?}", value);
```

## UI実装

```rust
// マーカーコンポーネントで識別
#[derive(Component)]
pub struct HotbarSlotUI(pub usize);

#[derive(Component)]
pub struct InventoryPanel;

// UI状態変更時は必ず呼ぶ
set_ui_open_state(&mut commands, &windows, true);  // UI開く
set_ui_open_state(&mut commands, &windows, false); // UI閉じる
```

## 入力処理

```rust
// InputStateで状態チェック
let state = InputState::current(
    &inventory_open,
    &furnace,
    &crusher,
    &miner,
    &command,
    &cursor,
);

if !state.allows_movement() {
    return;
}
```

## テスト

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_機能名_テスト内容() {
        // Arrange
        let mut inventory = Inventory::default();

        // Act
        inventory.add_item(BlockType::Stone, 10);

        // Assert
        assert_eq!(inventory.get_item_count(BlockType::Stone), 10);
    }
}
```

## 禁止事項

| 禁止 | 理由 |
|------|------|
| `unwrap()` | パニックの原因。`if let` か `?` を使う |
| `clone()` 乱用 | パフォーマンス低下。参照で済むなら参照 |
| マジックナンバー | 定数化する |
| 1000行超ファイル | 即座に分割 |
| 深いネスト (3段以上) | early return で解消 |

## コミットルール

- 日本語で書く
- 「何をしたか」を簡潔に
- 技術詳細より目的を重視

```
良い例:
feat: コンベア分岐機能を追加
fix: インベントリ表示中の移動を無効化
refactor: machines.rsを機械ごとに分割

悪い例:
update code
fix bug
WIP
```
