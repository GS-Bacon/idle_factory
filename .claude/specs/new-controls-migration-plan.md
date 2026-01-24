# 新操作仕様 移行計画 v2

## 変更履歴
- v2: セルフレビュー反映、Bevy 0.18アップグレード計画追加、PlatformInventory廃止

---

## Phase 0: Bevy 0.15 → 0.18 アップグレード（先行実施）

### アップグレード理由

現在 Bevy 0.15 を使用。0.18 で利用可能な新機能により実装を簡略化できる。

### Bevy 0.16 の新機能（活用可能）

| 機能 | 活用方法 |
|------|----------|
| **ECS Relationships** | 親子関係の改善、ChildOf/Children |
| **Entity Disabling** | UI表示/非表示をVisibilityから移行可能 |
| **Improved Spawn API** | `children!`マクロでUI生成簡略化 |
| **Required Components** | MachineBundle等のバンドル簡略化 |

### Bevy 0.17 の新機能（活用可能）

| 機能 | 活用方法 |
|------|----------|
| **CursorOptions 分離** | Window から分離（廃止するので影響小） |
| **Headless UI Widgets** | パレットUIのベース |
| **Event → Message 分離** | イベントシステム改善 |

### Bevy 0.18 の新機能（活用可能）

| 機能 | 活用方法 |
|------|----------|
| **First-Party Camera Controllers** | 自前flyカメラを置き換え可能か要検証 |
| **Standard UI Widgets** | ボタン、スライダー等 |
| **Popover Component** | ツールチップ、メニュー |
| **Font Variations** | フォント機能改善 |

### 破壊的変更への対応

| 変更 | 対応 |
|------|------|
| `Window.cursor_options` → `CursorOptions` Component | カーソル処理を更新 |
| `Pointer<Pressed>` → `Pointer<Press>` | イベント名変更 |
| `Event` → `Message` (buffered events) | 必要に応じて移行 |

### アップグレード手順

```bash
# 1. Cargo.toml 更新
bevy = "0.18"

# 2. コンパイル・修正サイクル
cargo build 2>&1 | head -100  # エラー確認
# 破壊的変更を順次修正

# 3. テスト
cargo test && cargo clippy
```

### First-Party Camera Controller 検証

```rust
// Bevy 0.18 の組み込みカメラコントローラーを検証
// 中ボタン回転がサポートされているか確認
// サポートされていない場合は自前実装を維持
```

**判断基準**:
- 中ボタンドラッグで回転できるか？
- Alt+左ドラッグ対応か？
- カスタマイズ可能か？

→ 要件を満たさない場合は自前実装を維持

---

## 現状アーキテクチャとの整合性分析

### 廃止対象と関連コード（更新）

| 機能 | 関連ファイル | 影響範囲 |
|------|-------------|----------|
| **PlayerInventory** | `src/player/inventory.rs` | 手持ちアイテム管理 |
| **PlatformInventory** | `src/player/platform_inventory.rs` | **廃止**（到達即ポイント化） |
| **インベントリUI** | `src/systems/inventory_ui/*.rs` (7ファイル) | スロット表示・操作 |
| **ホットバー** | `src/systems/hotbar.rs`, `src/setup/ui/inventory_ui.rs` | 1-9キー選択 |
| **カーソルロック** | `src/systems/cursor.rs`, `src/systems/player.rs` | FPS操作 |
| **サバイバル移動** | `src/systems/player.rs` (survival_movement) | 衝突・重力 |
| **破壊時間** | `src/systems/block_operations/breaking.rs` | 進捗バー |
| **手動納品** | `src/systems/quest.rs` (quest_deliver_button) | 納品ボタン |

### 維持対象（更新）

| 機能 | 関連ファイル | 変更点 |
|------|-------------|--------|
| **LocalPlayer** | `src/player/inventory.rs` | **維持**（カメラ・移動で使用） |
| **コンベアシステム** | `src/logistics/conveyor.rs` | 到達時に即ポイント化 |
| **機械システム** | `src/machines/generic/*.rs` | クリックでUI表示（維持） |
| **クエストシステム** | `src/systems/quest.rs` | 自動完了に変更 |
| **イベントシステム** | `src/events/*.rs` | そのまま維持 |

### 追加で変更が必要なファイル（セルフレビューで発見）

| ファイル | 変更内容 |
|----------|----------|
| `src/save/format/*.rs` | セーブフォーマット変更（Inventory削除） |
| `src/setup/initial_items.rs` | 初期ポイント設定に変更 |
| `src/systems/tutorial.rs` | チュートリアル全面更新 |
| `src/components/ui.rs` | `UIContext::Inventory` 削除 |
| `tests/e2e_test.rs` | テスト更新 |

### 新規追加

| 機能 | 新規ファイル | 概要 |
|------|-------------|------|
| **BuildingPoints** | `src/core/points.rs` | ポイントリソース |
| **PlacementMode** | `src/systems/placement_mode.rs` | カテゴリ・選択状態 |
| **パレットUI** | `src/setup/ui/palette_ui.rs` | 画面下部UI |
| **マウス設定** | `src/settings.rs` に追加 | 左右入れ替え |

---

## 依存関係グラフ（更新）

```
現状の依存:
┌─────────────────┐
│ PlayerInventory │◄─┬── block_place (選択アイテム取得)
│   Component     │  ├── block_break (アイテム追加)
└────────┬────────┘  ├── hotbar (選択切替)
         │           └── inventory_ui (表示)
         ▼
┌─────────────────┐     ┌───────────────────┐
│   ホットバーUI   │     │ PlatformInventory │◄── conveyor_transfer
└─────────────────┘     └─────────┬─────────┘
                                  │
                                  ▼
                        ┌─────────────────┐
                        │  quest_deliver  │◄── 手動ボタン
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
└─────────────────┘  ├── conveyor_transfer (到達時付与)
                     └── quest_complete (完了報酬)
```

---

## 詳細移行フェーズ（更新）

### Phase 0: Bevy 0.18 アップグレード

**目的**: 最新Bevyの機能を活用、技術的負債解消

#### 0.1 Cargo.toml 更新

```toml
bevy = { version = "0.18", default-features = false, features = [...] }
```

#### 0.2 破壊的変更対応

| 変更 | 対応ファイル |
|------|-------------|
| CursorOptions 分離 | `src/systems/player.rs`, `src/systems/cursor.rs` |
| Pointer イベント名 | `src/systems/inventory_ui/*.rs` |
| Required Components | `src/machines/generic/mod.rs` |

#### 0.3 新機能活用（オプション）

- `children!` マクロでUI生成簡略化
- Entity Disabling でUI表示制御
- First-Party Camera Controller 検証

### Phase 1: 操作系変更（基盤）

**目的**: カーソルロック廃止、新カメラ操作

#### 1.1 カーソル常時表示化

**変更箇所**:
- `src/systems/player.rs`: `toggle_cursor_lock` → 削除
- `src/systems/player.rs`: `initialize_cursor` → カーソル解放に固定
- `src/systems/cursor.rs`: `lock_cursor`/`release_cursor` → 常時解放

**Bevy 0.17+ 対応**:
```rust
// 旧 (0.15): window.cursor_options.grab_mode = ...
// 新 (0.17+): CursorOptions component を使用
fn release_cursor(cursor_options: &mut CursorOptions) {
    cursor_options.grab_mode = CursorGrabMode::None;
    cursor_options.visible = true;
}
```

#### 1.2 中ボタンドラッグで視点回転

**変更箇所**:
- `src/systems/player.rs`: `player_look` を改修
- `src/input/mod.rs`: 必要に応じて更新

**新ロジック**:
```rust
// 中ボタン押下中、またはAlt+左ボタン押下中のみ回転
let rotating = mouse_button.pressed(MouseButton::Middle)
    || (keyboard.pressed(KeyCode::AltLeft) && mouse_button.pressed(MouseButton::Left));

if rotating {
    let delta = accumulated_mouse_motion.delta;
    camera.yaw -= delta.x * sensitivity;
    camera.pitch -= delta.y * sensitivity;
}
```

#### 1.3 サバイバルモード削除

**変更箇所**:
- `src/systems/player.rs`: `player_move` → 飛行移動のみ
- `src/systems/player.rs`: `survival_movement` → 削除
- `src/lib.rs`: `GRAVITY`, `JUMP_VELOCITY`, `TERMINAL_VELOCITY` → 削除
- `src/components/player.rs`: `PlayerPhysics` → 簡略化
- `CreativeMode` リソース → 削除

**衝突なし実装**:
```rust
pub fn player_move(
    time: Res<Time>,
    input: Res<InputManager>,
    mut player_query: Query<&mut Transform, With<Player>>,
    camera_query: Query<&PlayerCamera>,
    // PlayerPhysics不要
) {
    // 常に飛行移動、衝突判定なし
    let mut direction = Vec3::ZERO;
    if input.pressed(GameAction::MoveForward) { direction += forward; }
    if input.pressed(GameAction::MoveBackward) { direction -= forward; }
    if input.pressed(GameAction::MoveLeft) { direction -= right; }
    if input.pressed(GameAction::MoveRight) { direction += right; }
    if input.pressed(GameAction::Jump) { direction.y += 1.0; }
    if input.pressed(GameAction::Descend) { direction.y -= 1.0; }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
        player_transform.translation += direction * PLAYER_SPEED * dt;
    }
}
```

---

### Phase 2: ブロック操作変更

**目的**: 即時撤去、左右クリック役割変更

#### 2.1 即時撤去

**変更箇所**:
- `src/systems/block_operations/breaking.rs`: 即時撤去に変更
- `src/systems/inventory_ui/breaking_bar.rs` → 削除

#### 2.2 左=撤去、右=設置

**変更箇所**:
- `src/systems/block_operations/breaking.rs`: 左クリック
- `src/systems/block_operations/placement.rs`: 右クリック
- `src/settings.rs`: `swap_mouse_buttons: bool` 追加

#### 2.3 撤去時の動作変更

**現状**: 撤去 → PlayerInventory に追加
**新仕様**: 撤去 → ポイント返還（アイテムは消滅）

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
```

#### 3.2 パレットUI

**新規**: `src/setup/ui/palette_ui.rs`

Bevy 0.18 の Standard Widgets を活用:
```rust
// Bevy 0.18 の children! マクロで簡略化
commands.spawn((
    Node { ... },
    PalettePanel,
    children![
        // カテゴリタブ
        // アイコンスロット（1-9）
        // ポイント残高
    ],
));
```

---

### Phase 4: ポイントシステム

**目的**: 設置コスト・撤去返還・アイテム到達時付与

#### 4.1 BuildingPoints リソース

**新規**: `src/core/points.rs`

```rust
#[derive(Resource)]
pub struct BuildingPoints {
    pub current: u32,
}

impl Default for BuildingPoints {
    fn default() -> Self {
        Self { current: 100 } // 初期ポイント
    }
}
```

#### 4.2 初期ポイント設定

**変更**: `src/setup/initial_items.rs`

```rust
// 旧: PlayerInventory に初期アイテム
// 新: BuildingPoints に初期ポイント
pub fn setup_initial_points(mut commands: Commands) {
    commands.insert_resource(BuildingPoints { current: 100 });
}
```

---

### Phase 5: 自動納品（PlatformInventory廃止）

**目的**: 到達即ポイント化、蓄積なし

#### 5.1 conveyor_transfer 改修

**変更箇所**: `src/logistics/conveyor.rs`

```rust
TransferTarget::Delivery => {
    // 旧: platform_inventory.add_item(...)
    // 新: 即座にポイント化
    let point_value = get_item_point_value(item.item_id);
    points.add(point_value);

    // クエスト進捗更新
    quest_progress.record_delivery(item.item_id, 1);

    // ビジュアル削除
    if let Some(visual) = item.visual_entity {
        commands.entity(visual).despawn();
    }
    source_conv.items.remove(action.item_index);
}
```

#### 5.2 クエスト進捗管理（新規）

**新規**: `src/systems/quest_progress.rs`

```rust
#[derive(Resource, Default)]
pub struct QuestProgress {
    pub delivered: HashMap<ItemId, u32>,
}

impl QuestProgress {
    pub fn record_delivery(&mut self, item_id: ItemId, count: u32) {
        *self.delivered.entry(item_id).or_default() += count;
    }

    pub fn check_completion(&self, requirements: &[(ItemId, u32)]) -> bool {
        requirements.iter().all(|(id, req)| {
            self.delivered.get(id).copied().unwrap_or(0) >= *req
        })
    }
}
```

#### 5.3 PlatformInventory 削除

**削除対象**:
- `src/player/platform_inventory.rs`
- `LocalPlatformInventory` type alias
- 関連するセーブ/ロードコード

---

### Phase 6: 既存機能削除

**目的**: 不要になったコードを削除

#### 6.1 削除対象ファイル

```
src/player/inventory.rs              # PlayerInventory（LocalPlayerは別ファイルに移動）
src/player/platform_inventory.rs     # PlatformInventory
src/systems/inventory_ui/            # 全7ファイル
src/systems/hotbar.rs                # ホットバー
src/setup/ui/inventory_ui.rs         # インベントリUIセットアップ
src/game_spec/breaking_spec.rs       # 破壊時間定義
```

#### 6.2 LocalPlayer の移動

`src/player/inventory.rs` から `src/player/mod.rs` または `src/components/player.rs` に移動:

```rust
/// Resource holding the local player's entity
#[derive(Resource)]
pub struct LocalPlayer(pub Entity);
```

#### 6.3 UIContext 更新

**変更**: `src/components/ui.rs`

```rust
pub enum UIContext {
    Gameplay,
    PauseMenu,
    Settings,
    Quest,
    // Inventory, ← 削除
    MachineUI(Entity),
}
```

#### 6.4 チュートリアル更新

**変更**: `src/systems/tutorial.rs`

- インベントリ操作のチュートリアル削除
- パレット操作のチュートリアル追加
- 設置/撤去操作の説明更新

#### 6.5 セーブフォーマット更新

**変更**: `src/save/format/*.rs`

```rust
// 旧
pub struct SaveData {
    pub player_inventory: Vec<(ItemId, u32)>,
    pub platform_inventory: Vec<(ItemId, u32)>,
    // ...
}

// 新
pub struct SaveData {
    pub building_points: u32,
    pub quest_progress: HashMap<ItemId, u32>,
    // player_inventory, platform_inventory 削除
}
```

#### 6.6 テスト更新

**変更**: `tests/e2e_test.rs`, `tests/scenarios/*.toml`

- インベントリ関連テストを削除
- ポイントシステムのテスト追加
- パレット操作のテスト追加

---

## 移行チェックリスト

### Phase 0 完了条件
- [ ] `bevy = "0.18"` でコンパイル成功
- [ ] CursorOptions 対応完了
- [ ] 全テストパス
- [ ] First-Party Camera Controller 検証完了

### Phase 1 完了条件
- [ ] カーソルが常時表示される
- [ ] 中ボタンドラッグで視点回転する
- [ ] Alt+左ドラッグでも視点回転する
- [ ] 衝突判定なしで移動できる
- [ ] 重力なしで上下移動できる

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

### Phase 4 完了条件
- [ ] 設置時にポイントが消費される
- [ ] 撤去時にポイントが返還される
- [ ] コンベアが無料で設置できる
- [ ] ポイント残高がUIに表示される
- [ ] 初期ポイントが付与される

### Phase 5 完了条件
- [ ] コンベアでプラットフォーム到達時に即ポイント化
- [ ] PlatformInventory 参照が0件
- [ ] クエスト進捗が正しく記録される
- [ ] クエスト完了時に報酬ポイントが付与される

### Phase 6 完了条件
- [ ] PlayerInventory関連コードが0行
- [ ] PlatformInventory関連コードが0行
- [ ] ホットバー関連コードが0行
- [ ] インベントリUI関連コードが0行
- [ ] LocalPlayer が正しく維持されている
- [ ] チュートリアルが更新されている
- [ ] セーブ/ロードが動作する
- [ ] `cargo build` 成功
- [ ] `cargo test` 全パス
- [ ] E2Eテストパス

---

## リスクと対策（更新）

| リスク | 影響 | 対策 |
|--------|------|------|
| Bevy 0.18 移行失敗 | 開発停止 | 0.16→0.17→0.18 段階的移行 |
| セーブデータ非互換 | 既存セーブが読めない | CLAUDE.mdに「後方互換不要」明記済み |
| LocalPlayer 削除ミス | カメラ・移動が動かない | Phase 6 で明示的に維持確認 |
| クエスト進行不能 | ゲーム詰み | 初期ポイント十分付与、コンベア無料 |
| 操作混乱 | UX悪化 | チュートリアル更新、設定で入れ替え可能 |

---

## 推奨実装順序（更新）

```
Phase 0 (Bevy 0.18) → Phase 1 (操作系) → Phase 2 (ブロック操作)
                                              ↓
Phase 4 (ポイント) ← Phase 3 (パレットUI)
         ↓
Phase 5 (自動納品・PlatformInventory廃止)
         ↓
Phase 6 (削除)
```

**理由**:
1. Phase 0 を先に行うことで、新機能（children!マクロ等）を活用できる
2. Phase 4 は Phase 3 と並行可能（ポイント表示のため）
3. Phase 5 は PlatformInventory 廃止を含むため、Phase 4 の後
4. Phase 6（削除）は最後に行い、他の機能が動作確認できてから削除

---

## 参考リンク

- [Bevy 0.16 Release Notes](https://bevy.org/news/bevy-0-16/)
- [Bevy 0.17 Release Notes](https://bevy.org/news/bevy-0-17/)
- [Bevy 0.18 Release Notes](https://bevy.org/news/bevy-0-18/)
- [Bevy 0.15 to 0.16 Migration](https://bevy.org/learn/migration-guides/0-15-to-0-16/)
- [Bevy 0.16 to 0.17 Migration](https://bevy.org/learn/migration-guides/0-16-to-0-17/)
