# 新操作仕様 移行計画

## 現状アーキテクチャとの整合性分析

### 廃止対象と関連コード

| 機能 | 関連ファイル | 影響範囲 |
|------|-------------|----------|
| **PlayerInventory** | `src/player/inventory.rs` | 手持ちアイテム管理 |
| **インベントリUI** | `src/systems/inventory_ui/*.rs` (7ファイル) | スロット表示・操作 |
| **ホットバー** | `src/systems/hotbar.rs`, `src/setup/ui/inventory_ui.rs` | 1-9キー選択 |
| **カーソルロック** | `src/systems/cursor.rs`, `src/systems/player.rs` | FPS操作 |
| **サバイバル移動** | `src/systems/player.rs` (survival_movement) | 衝突・重力 |
| **破壊時間** | `src/systems/block_operations/breaking.rs`, `src/game_spec/breaking_spec.rs` | 進捗バー |
| **手動納品** | `src/systems/quest.rs` (quest_deliver_button) | 納品ボタン |

### 維持対象

| 機能 | 関連ファイル | 変更点 |
|------|-------------|--------|
| **PlatformInventory** | `src/player/platform_inventory.rs` | そのまま維持 |
| **コンベアシステム** | `src/logistics/conveyor.rs` | そのまま維持 |
| **機械システム** | `src/machines/generic/*.rs` | クリックでUI表示（維持） |
| **クエストシステム** | `src/systems/quest.rs` | 自動納品に変更 |
| **イベントシステム** | `src/events/*.rs` | そのまま維持 |

### 新規追加

| 機能 | 新規ファイル | 概要 |
|------|-------------|------|
| **BuildingPoints** | `src/core/points.rs` | ポイントリソース |
| **設置モード** | `src/systems/placement_mode.rs` | カテゴリ・選択状態 |
| **パレットUI** | `src/setup/ui/palette_ui.rs` | 画面下部UI |
| **マウス設定** | `src/settings.rs` に追加 | 左右入れ替え |

---

## 依存関係グラフ

```
現状の依存:
┌─────────────────┐
│ PlayerInventory │◄─┬── block_place (選択アイテム取得)
│   Component     │  ├── block_break (アイテム追加)
└────────┬────────┘  ├── hotbar (選択切替)
         │           └── inventory_ui (表示)
         ▼
┌─────────────────┐
│   ホットバーUI   │◄── 1-9キー入力
└─────────────────┘

新仕様の依存:
┌─────────────────┐
│ PlacementMode   │◄─┬── block_place (選択アイテム取得)
│   Resource      │  ├── palette_ui (表示・選択)
└────────┬────────┘  └── カテゴリキー入力
         │
         ▼
┌─────────────────┐
│ BuildingPoints  │◄─┬── block_place (消費)
│   Resource      │  ├── block_break (返還)
└─────────────────┘  └── quest (報酬)
```

---

## 詳細移行フェーズ

### Phase 1: 操作系変更（基盤）

**目的**: カーソルロック廃止、新カメラ操作

#### 1.1 カーソル常時表示化

**変更箇所**:
- `src/systems/player.rs`: `toggle_cursor_lock` → 削除
- `src/systems/player.rs`: `initialize_cursor` → カーソル解放に固定
- `src/systems/cursor.rs`: `lock_cursor`/`release_cursor` → 常時解放

**テスト**:
```toml
[[steps]]
action = "assert"
params = { condition = "cursor_locked == false" }
```

#### 1.2 中ボタンドラッグで視点回転

**変更箇所**:
- `src/systems/player.rs`: `player_look` を改修
  - マウス移動での回転を削除
  - 中ボタン押下中のみ回転適用
- `src/input/mod.rs`: `GameAction::RotateView` 追加

**新ロジック**:
```rust
// 中ボタン押下中、またはAlt+左ボタン押下中のみ回転
let rotating = mouse_button.pressed(MouseButton::Middle)
    || (keyboard.pressed(KeyCode::AltLeft) && mouse_button.pressed(MouseButton::Left));

if rotating {
    camera.yaw -= delta.x * sensitivity;
    camera.pitch -= delta.y * sensitivity;
}
```

#### 1.3 サバイバルモード削除

**変更箇所**:
- `src/systems/player.rs`: `player_move` → クリエイティブ移動のみ
- `src/systems/player.rs`: `survival_movement` → 削除
- `src/lib.rs`: `GRAVITY`, `JUMP_VELOCITY`, `TERMINAL_VELOCITY` → 削除
- `src/components/player.rs`: `PlayerPhysics` → 簡略化（on_ground等不要）
- `CreativeMode` リソース → 削除（常にクリエイティブ相当）

**衝突なし実装**:
```rust
pub fn player_move(...) {
    // 常に飛行移動、衝突判定なし
    let mut direction = Vec3::ZERO;
    if input.pressed(GameAction::MoveForward) { direction += forward; }
    // ...省略...
    player_transform.translation += direction * PLAYER_SPEED * dt;
}
```

#### 1.4 スクロールズーム無効化

**確認**: 現状スクロールズームは実装されていない → 対応不要

---

### Phase 2: ブロック操作変更

**目的**: 即時撤去、左右クリック役割変更

#### 2.1 即時撤去

**変更箇所**:
- `src/systems/block_operations/breaking.rs`:
  - `BreakingProgress` → 削除
  - 左クリックで即座に撤去実行
- `src/systems/inventory_ui/breaking_bar.rs` → 削除
- `src/game_spec/breaking_spec.rs` → 削除

**新ロジック**:
```rust
pub fn block_break(...) {
    // 左クリックで即座に撤去
    if mouse_button.just_pressed(MouseButton::Left) {
        // find_break_target() でターゲット取得
        // execute_machine_break() or execute_block_break() を即実行
        // ポイント返還処理追加
    }
}
```

#### 2.2 左=撤去、右=設置

**変更箇所**:
- `src/systems/block_operations/breaking.rs`: 左クリックで撤去
- `src/systems/block_operations/placement.rs`: 右クリックで設置
- `src/settings.rs`: `SwapMouseButtons: bool` 追加

**設定による入れ替え**:
```rust
let (break_button, place_button) = if settings.swap_mouse_buttons {
    (MouseButton::Right, MouseButton::Left)
} else {
    (MouseButton::Left, MouseButton::Right)
};
```

#### 2.3 撤去時の動作変更

**現状**: 撤去 → PlayerInventory に追加
**新仕様**: 撤去 → ポイント返還（アイテムは消滅）

**変更箇所**:
- `execute_machine_break()`: `inventory.add_item_by_id()` → `points.add(cost)`
- `execute_block_break()`: 同上

---

### Phase 3: 設置モードUI（パレット）

**目的**: PlayerInventory依存を断ち切る

#### 3.1 PlacementMode リソース

**新規**: `src/systems/placement_mode.rs`

```rust
#[derive(Resource, Default)]
pub struct PlacementMode {
    pub category: Category,
    pub selected_index: usize,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    #[default]
    Logistics,  // K
    Production, // P
    Power,      // L
    Building,   // B
}

impl PlacementMode {
    pub fn selected_item(&self) -> Option<ItemId> {
        get_category_items(self.category).get(self.selected_index).copied()
    }
}

fn get_category_items(category: Category) -> &'static [ItemId] {
    match category {
        Category::Logistics => &[items::conveyor_block(), items::splitter_block(), ...],
        Category::Production => &[items::miner_block(), items::furnace_block(), ...],
        // ...
    }
}
```

#### 3.2 カテゴリキー実装

**変更箇所**: `src/input/mod.rs`

```rust
pub enum GameAction {
    // ...既存...
    // カテゴリ切替
    CategoryLogistics,  // K
    CategoryProduction, // P
    CategoryPower,      // L
    CategoryBuilding,   // B
}
```

**システム**: `src/systems/placement_mode.rs`

```rust
pub fn handle_category_input(
    input: Res<InputManager>,
    mut mode: ResMut<PlacementMode>,
) {
    if input.just_pressed(GameAction::CategoryLogistics) {
        mode.category = Category::Logistics;
        mode.selected_index = 0;
    }
    // ...

    // 数字キーで選択
    for i in 0..9 {
        if input.just_pressed(GameAction::Hotbar1 + i) {
            mode.selected_index = i;
        }
    }
}
```

#### 3.3 パレットUI

**新規**: `src/setup/ui/palette_ui.rs`

```rust
pub fn setup_palette_ui(mut commands: Commands, ...) {
    // 画面下部にパレットパネル
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Percent(50.0),
            // ...
        },
        PalettePanel,
    )).with_children(|parent| {
        // カテゴリ表示
        // アイコンスロット（1-9）
        // ポイント残高
    });
}
```

#### 3.4 block_place 改修

**変更箇所**: `src/systems/block_operations/placement.rs`

```rust
// 旧: let Some(selected_item_id) = inventory.selected_item_id()
// 新:
let Some(selected_item_id) = placement_mode.selected_item() else {
    return;
};

// ポイント消費
let cost = get_placement_cost(selected_item_id);
if !points.try_consume(cost) {
    return; // ポイント不足
}
```

---

### Phase 4: ポイントシステム

**目的**: 設置コスト・撤去返還・クエスト報酬

#### 4.1 BuildingPoints リソース

**新規**: `src/core/points.rs`

```rust
#[derive(Resource, Default)]
pub struct BuildingPoints {
    pub current: u32,
}

impl BuildingPoints {
    pub fn add(&mut self, amount: u32) {
        self.current = self.current.saturating_add(amount);
    }

    pub fn try_consume(&mut self, amount: u32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }
}
```

#### 4.2 コスト定義

**新規**: `src/game_spec/costs.rs`

```rust
pub fn get_placement_cost(item_id: ItemId) -> u32 {
    if item_id == items::conveyor_block() { return 0; } // 無料
    if item_id == items::splitter_block() { return 2; }
    if item_id == items::miner_block() { return 5; }
    if item_id == items::furnace_block() { return 10; }
    if item_id == items::crusher_block() { return 8; }
    // 電力系（将来）
    // if item_id == items::power_wire() { return 4; }
    1 // デフォルト
}

pub fn get_refund_rate() -> f32 {
    1.0 // 100%返還（難易度設定で変更可能に）
}
```

#### 4.3 ポイントUI

**変更箇所**: `src/setup/ui/palette_ui.rs`

```rust
// パレット右端にポイント表示
parent.spawn((
    Text::new("1250"),
    PointsDisplay,
));

// 更新システム
pub fn update_points_display(
    points: Res<BuildingPoints>,
    mut query: Query<&mut Text, With<PointsDisplay>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        **text = format!("{}", points.current);
    }
}
```

#### 4.4 クエスト報酬でポイント付与

**変更箇所**: `src/systems/quest.rs`

```rust
// quest_claim_rewards を改修
pub fn quest_claim_rewards(..., mut points: ResMut<BuildingPoints>) {
    // ...

    // アイテム報酬 → ポイントに変換
    for (item_id, amount) in &quest.rewards {
        let point_value = get_item_point_value(*item_id) * amount;
        points.add(point_value);
    }
}
```

---

### Phase 5: 既存機能削除

**目的**: 不要になったコードを削除

#### 5.1 削除対象ファイル

```
src/player/inventory.rs              # PlayerInventory
src/systems/inventory_ui/            # 全7ファイル
src/systems/hotbar.rs                # ホットバー
src/setup/ui/inventory_ui.rs         # インベントリUIセットアップ
src/systems/inventory_ui/breaking_bar.rs # 破壊進捗バー
src/game_spec/breaking_spec.rs       # 破壊時間定義
```

#### 5.2 削除対象コンポーネント/リソース

```rust
// 削除
- PlayerInventory (Component)
- BreakingProgress (Resource)
- CreativeMode (Resource)  # 常にクリエイティブ相当
- CursorLockState (Resource)  # カーソル常時解放
- InventoryOpen (Resource)
- HeldItem (Component)
- InventorySlot (Component)
```

#### 5.3 削除対象システム

```rust
// 削除
- toggle_cursor_lock
- survival_movement
- update_breaking_progress_ui
- inventory_slot_click
- inventory_update_slots
- update_inventory_visibility
- hotbar_input
```

#### 5.4 GameAction整理

```rust
pub enum GameAction {
    // 維持
    MoveForward, MoveBackward, MoveLeft, MoveRight,
    Jump,  // 上昇に使用
    Descend,
    LookUp, LookDown, LookLeft, LookRight,  // 矢印キー
    TogglePause,
    ToggleQuest,
    Confirm, Cancel,
    PrimaryAction, SecondaryAction,
    RotateBlock,
    ToggleDebug,

    // 削除
    // ToggleInventory,  # インベントリなし
    // Hotbar1-9,        # パレットの数字キーで再利用

    // 追加
    CategoryLogistics,
    CategoryProduction,
    CategoryPower,
    CategoryBuilding,
}
```

---

### Phase 6: 自動納品

**目的**: 手動納品ボタン廃止、コンベア到達時に自動納品

#### 6.1 conveyor_transfer 改修

**変更箇所**: `src/logistics/conveyor.rs`

```rust
TransferTarget::Delivery => {
    // PlatformInventory に追加
    platform_inventory.add_item(item.item_id, 1);

    // 自動でクエスト進捗チェック・完了処理
    auto_deliver_for_quest(&mut current_quest, &quest_cache, item.item_id, 1, &mut points);

    // ...
}
```

#### 6.2 自動納品処理

**新規**: `src/systems/auto_delivery.rs`

```rust
pub fn auto_deliver_for_quest(
    current_quest: &mut CurrentQuest,
    quest_cache: &QuestCache,
    item_id: ItemId,
    count: u32,
    points: &mut BuildingPoints,
) {
    if current_quest.completed {
        return;
    }

    let Some(quest) = quest_cache.main_quests.get(current_quest.index) else {
        return;
    };

    // クエスト要件を満たしているかチェック（PlatformInventoryベース）
    // → 満たしていれば自動的にcompleted = true, ポイント付与
}
```

#### 6.3 クエストUI簡略化

**変更箇所**: `src/systems/quest.rs`

```rust
// 削除
- quest_deliver_button
- QuestDeliverButton Component

// update_quest_ui を簡略化
// 「納品」ボタンなし、進捗バーのみ
```

---

## 移行チェックリスト

### Phase 1 完了条件
- [ ] カーソルが常時表示される
- [ ] 中ボタンドラッグで視点回転する
- [ ] Alt+左ドラッグでも視点回転する
- [ ] 衝突判定なしで移動できる
- [ ] 重力なしで上下移動できる
- [ ] `cargo test` 全パス
- [ ] `cargo clippy` 警告0

### Phase 2 完了条件
- [ ] 左クリックで即座に撤去できる
- [ ] 右クリックで即座に設置できる
- [ ] 設定で左右入れ替えできる
- [ ] 破壊進捗バーが表示されない

### Phase 3 完了条件
- [ ] Kキーで物流カテゴリが選択される
- [ ] 数字キーでアイテムが選択される
- [ ] パレットUIが画面下部に表示される
- [ ] パレットクリックで選択できる
- [ ] PlayerInventory参照が0件

### Phase 4 完了条件
- [ ] 設置時にポイントが消費される
- [ ] 撤去時にポイントが返還される
- [ ] コンベアが無料で設置できる
- [ ] ポイント残高がUIに表示される
- [ ] クエスト完了でポイントが付与される

### Phase 5 完了条件
- [ ] PlayerInventory関連コードが0行
- [ ] ホットバー関連コードが0行
- [ ] インベントリUI関連コードが0行
- [ ] `cargo build` 成功
- [ ] `cargo test` 全パス

### Phase 6 完了条件
- [ ] 納品ボタンがない
- [ ] コンベアでプラットフォームに到達したら自動納品
- [ ] クエスト完了時にポイントが自動付与
- [ ] ゲームが正常に進行する（E2Eテスト）

---

## リスクと対策

| リスク | 影響 | 対策 |
|--------|------|------|
| セーブデータ非互換 | 既存セーブが読めない | CLAUDE.mdに「後方互換不要」明記済み |
| PlayerInventory参照漏れ | コンパイルエラー | Phase 3→5の順でコンパイル確認 |
| クエスト進行不能 | ゲーム詰み | 初期ポイント十分付与、コンベア無料 |
| 操作混乱 | UX悪化 | チュートリアル更新、設定で入れ替え可能 |

---

## 見積もり

| Phase | 変更規模 | 新規/削除 |
|-------|----------|-----------|
| 1 | 中 | player.rs改修、cursor.rs簡略化 |
| 2 | 中 | breaking.rs改修、placement.rs改修 |
| 3 | 大 | 新規UI、新リソース、システム追加 |
| 4 | 中 | 新リソース、システム追加 |
| 5 | 中 | 大量削除（コンパイル確認が重要） |
| 6 | 小 | conveyor.rs改修、quest.rs簡略化 |

**推奨順序**: 1 → 2 → 3 → 4 → 6 → 5

理由: Phase 5（削除）は最後に行い、他の機能が動作確認できてから削除する
