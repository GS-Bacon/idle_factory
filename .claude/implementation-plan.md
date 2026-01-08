# 実装計画

> 詳細な将来設計は `.claude/architecture-future.md` 参照
> ロードマップは `.specify/roadmap.md` 参照
> **移行状況確認**: `./scripts/migration-status.sh`

## 現状サマリー (2026-01-08更新)

| 項目 | 値 |
|------|-----|
| コード行数 | **~25,000行** |
| テスト | **643件** 通過 |
| Clippy警告 | **0件** |
| Phase | **M.1-M.9 完了、BlockType段階的移行中** |

### 完了済みタスク (最新確認)

| タスク | 状態 | 備考 |
|--------|------|------|
| T.1 固定Tick | ✅ | `Time::<Fixed>::from_hz(20.0)` + FixedUpdate |
| M.2 PlayerInventory Component化 | ✅ | Query<&PlayerInventory>パターン |
| M.3 GlobalInventory統合 | ✅ | PlatformInventory Component |
| M.4 MachineBundle移行 | ✅ | 全spawn箇所でMachineBundle使用 |
| M.7 NetworkId/EntityMap | ✅ | components/network.rs実装済み |
| M.8 イベント送信網羅 | ✅ | 8イベント全送信確認 |
| M.9 GuardedEventWriter使用 | ✅ | 7ファイルで使用中 |
| P.0-P.3 パニック防止 | ✅ | フォールバック実装済み |
| P.4 Blockbenchフォールバック | ✅ | BlockbenchLoadErrorでエラー返却 |
| D.3 WebSocket完成 | ✅ | port 9877で起動、18メソッド実装 |

### 残タスク

| タスク | 状態 | 見積もり |
|--------|------|----------|
| M.1 BlockType廃止 | 🔄 段階的移行中 | 8-10時間 |
| P.5 ValidItemId導入 | ✅ 完了 | - |

---

## 次のタスク: M.1 BlockType完全廃止

> **目標**: BlockType/BlockTypeSave を完全廃止し、動的ID（ItemId/BlockId）に統一
> **見積もり**: 8-10時間
> **確認コマンド**: `./scripts/migration-status.sh`

### 廃止対象

| 対象 | 現状 | 廃止後 |
|------|------|--------|
| BlockType enum | 994箇所 | **削除** |
| BlockTypeSave enum | 188箇所 | **削除** |
| ブロック定義 | Rustハードコード | TOML外部化 |
| セーブV1形式 | 読込対応あり | **切り捨て** |

### 前提条件 ✅

- [x] P.0-P.3 パニック防止完了
- [x] ConveyorItem/MachineSlot が ItemId 直接保持
- [x] テスト643件パス
- [x] Clippy警告0件

### 作業順序

| Phase | 内容 | 時間 |
|-------|------|------|
| **0** | ブロック外部化基盤（BlockId, TOML, Registry） | 90分 |
| **1** | game_spec/*.rs 移行 (174箇所) | 60分 |
| **2** | player/*.rs + core/inventory.rs 移行 (114箇所) | 45分 |
| **3** | craft + components + statistics 移行 (120箇所) | 45分 |
| **4** | セーブ完全移行（BlockTypeSave削除, V1切り捨て, 212箇所） | **2.5時間** |
| **5** | main.rs + 各systems 移行 (78箇所) | 45分 |
| **6** | block_type.rs 削除 + 残存参照修正 | 30分 |
| **7** | 検証 | 30分 |

### チェックリスト

- [ ] Phase 0 完了 → `cargo test` + 起動確認
- [ ] Phase 1 完了 → `cargo test`
- [ ] Phase 2 完了 → `cargo test`
- [ ] Phase 3 完了 → `cargo test`
- [ ] Phase 4 完了 → `cargo test`
- [ ] Phase 5 完了 → `cargo test`
- [ ] Phase 6 完了 → `cargo test`
- [ ] 最終確認:
  - [ ] `grep -r 'BlockType' src | wc -l` = 0
  - [ ] `grep -r 'BlockTypeSave' src | wc -l` = 0
  - [ ] テスト全パス
  - [ ] Clippy警告0件
  - [ ] ゲーム起動確認

### M.1 詳細: Phase 0 ブロック外部化基盤

#### 新規ファイル: `mods/base/blocks.toml`
```toml
[[blocks]]
id = "base:stone"
color = [0.5, 0.5, 0.5, 1.0]
hardness = 1.0
model = "cube"
collision = "solid"
drops = "base:stone"

[[blocks]]
id = "base:iron_ore"
color = [0.6, 0.5, 0.4, 1.0]
hardness = 2.0
model = "cube"
collision = "solid"
drops = "base:iron_ore"

# ... 他15ブロック（全17種）
```

#### 新規ファイル: `src/core/block_id.rs`
```rust
pub type BlockId = Id<Block>;

pub struct BlockDescriptor {
    pub id: BlockId,
    pub color: Color,
    pub hardness: f32,
    pub model: BlockModel,
    pub collision: CollisionType,
    pub drops: Option<ItemId>,
}

pub enum BlockModel {
    Cube,
    Custom(String),  // GLBパス
}

pub enum CollisionType {
    Solid,
    None,
    Partial(Vec3),
}
```

#### ChunkData移行
```rust
// Before
pub blocks: [[[BlockType; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]

// After
pub blocks: [[[BlockId; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]
// BlockId は u16 なのでメモリ効率は同等
```

### M.1 詳細: 移行パターン

```rust
// パターン1: match式 → if/Registry
// Before
match block_type {
    BlockType::Stone => ...,
    BlockType::IronOre => ...,
}
// After
if block_id == blocks::stone() { ... }
let desc = registry.block_by_id(block_id);

// パターン2: 型注釈
// Before
fn process(item: BlockType) { ... }
// After
fn process(item: ItemId) { ... }

// パターン3: 構造体フィールド
// Before
pub item_type: BlockType,
// After
pub item_id: ItemId,
```

### M.1 詳細: リスク対策

| リスク | 対策 |
|--------|------|
| テスト破壊 | Phase毎に`cargo test`、失敗したら即修正 |
| パニック | try_into() + unwrap_or()でフォールバック |
| 描画崩壊 | Phase 0でChunkData移行後すぐに起動確認 |
| セーブ破損 | V1切り捨て明示、新規セーブでテスト |

### M.1 詳細: 効果

| 項目 | Before | After |
|------|--------|-------|
| 新ブロック追加 | Rust再コンパイル | TOML追記のみ |
| Modブロック | 不可能 | 可能 |
| 型の混乱 | BlockType/ItemId混在 | 全てId<T>で統一 |
| enum肥大化リスク | あり | なし |

---

## 他の移行タスク（M.1完了後）

| # | タスク | 現状 | 見積もり |
|---|--------|------|----------|
| M.2 | **PlayerInventory Component化** | ❌ Resource | 2-3時間 |
| M.3 | **GlobalInventory 廃止/統合** | ❌ Resource | 2-3時間 |
| M.4 | **Machine::new → MachineBundle** | ❌ 0% | 1-2時間 |
| M.7 | **NetworkId / EntityMap** | ❌ 0% | 2-3時間 |
| M.8 | **イベント送信網羅** | 🔄 30% | 1-2時間 |
| M.9 | **GuardedEventWriter使用** | ❌ 定義のみ | 1時間 |

### 完了済み

| # | タスク | 状態 |
|---|--------|------|
| D.4 | **本体Mod化** | ✅ 起動時ロード完了 |
| D.1 | **イベント** | ✅ 7箇所でEventReader使用 |
| - | **InteractingMachine統合** | ✅ 旧Interacting* 削除済み |
| - | **レガシー機械削除** | ✅ 完了 |
| D.6-14 | **各機能プラグイン** | ✅ 完了 |

---

### M.2: PlayerInventory Component化

**ステータス**: ❌ 未移行

**現状**: `PlayerInventory` が `Resource` として存在（17ファイルで使用）

**目標**: `Inventory` を `Component` にし、`LocalPlayer` Entity に付与

**完了条件**:
- [ ] `grep -r 'Res<PlayerInventory>' src` が 0件
- [ ] `grep -r 'ResMut<PlayerInventory>' src` が 0件
- [ ] `LocalPlayer` Entity が `Inventory` Component を持つ
- [ ] マルチプレイ対応可能な設計

**影響ファイル**:
- src/player/inventory.rs - 定義変更
- src/player/mod.rs - 公開API変更
- src/systems/inventory_ui.rs - Query化
- src/systems/hotbar.rs - Query化
- src/machines/generic.rs - Query化
- src/save/systems.rs - Entity経由で保存
- その他11ファイル

**移行パターン**:
```rust
// Before
fn system(inventory: ResMut<PlayerInventory>) { ... }

// After
fn system(
    local_player: Res<LocalPlayer>,
    mut inventories: Query<&mut Inventory>,
) {
    if let Ok(mut inv) = inventories.get_mut(local_player.0) { ... }
}
```

---

### M.3: GlobalInventory 廃止/統合

**ステータス**: ❌ 未移行

**現状**: `GlobalInventory` が `Resource` として存在（16ファイルで使用）

**目標**: プラットフォーム（倉庫）の `Inventory` Component に統合

**完了条件**:
- [ ] `grep -r 'GlobalInventory' src` が 0件
- [ ] プラットフォームEntity が `Inventory` Component を持つ
- [ ] コンベア納品がプラットフォームのInventoryに追加

**依存**: M.2（Inventory Component化）が先

---

### M.4: Machine::new → MachineBundle 移行

**ステータス**: ❌ 未使用（定義のみ）

**現状**: `MachineBundle` が定義されているが、23箇所で `Machine::new` を直接使用

**目標**: 全ての機械spawn を `MachineBundle` 経由に統一

**完了条件**:
- [ ] `grep -r 'Machine::new' src` が 0件（テスト除く）
- [ ] 全機械が `MachineBundle::spawn()` で生成

**影響ファイル**:
- src/systems/block_operations/placement.rs (6箇所)
- src/systems/command/handlers.rs (6箇所)
- src/save/systems.rs (3箇所)
- src/machines/generic.rs (3箇所・テスト)
- src/main.rs (5箇所・テスト)

---

### M.5: セーブ形式 enum→文字列ID 完全移行

**ステータス**: 🔄 2% (V2保存対応済み、読込時変換残り)

**現状**:
- 保存: V2形式（文字列ID）✅
- 読込: V1/V2両対応、内部で`BlockTypeSave` enum経由

**目標**: 内部でも文字列IDを直接使用

**完了条件**:
- [ ] `BlockTypeSave` enum 削除
- [ ] 全セーブデータが `ItemId` (文字列) で処理

**依存**: M.1（BlockType→ItemId）がほぼ完了してから

---

### M.6: 本体コンテンツMod化

**ステータス**: 🔄 30% (TOML定義済み、GameRegistry未統合)

**現状**:
- `mods/base/items.toml` - 16アイテム定義済み ✅
- `mods/base/machines.toml` - 12機械定義済み ✅
- `mods/base/recipes.toml` - 37レシピ定義済み ✅
- `LoadedModData` - 起動時ロード ✅
- **問題**: `GameRegistry` は `ITEM_DESCRIPTORS` (Rust定数) を使用、TOML未使用

**目標**: GameRegistry が TOML から読み込んだデータを使用

**完了条件**:
- [ ] `ITEM_DESCRIPTORS` 定数を削除 or 空に
- [ ] `GameRegistry::new()` が `LoadedModData` から構築
- [ ] 新アイテム追加が TOML のみで完結

**影響ファイル**:
- src/game_spec/registry.rs - `ITEM_DESCRIPTORS` 削除、TOML統合
- src/modding/mod.rs - GameRegistry構築ロジック追加

**依存**: M.1（BlockType→ItemId）と並行可能

---

### M.7: NetworkId / EntityMap

**ステータス**: ❌ 未実装

**現状**: `LocalPlayer(Entity)` で直接Entity参照。マルチプレイ時にサーバー/クライアント間でEntity値が異なる問題。

**目標**: 全てのネットワーク越しEntity参照を `NetworkId` 経由に

**完了条件**:
- [ ] `NetworkId` Component 定義
- [ ] `EntityMap` Resource 定義
- [ ] 機械/プレイヤーの spawn 時に NetworkId 付与
- [ ] ネットワークメッセージで NetworkId 使用

**影響ファイル**:
- src/components/mod.rs - NetworkId 定義
- src/resources/mod.rs - EntityMap 定義
- src/systems/block_operations/placement.rs - 機械spawn時
- src/save/systems.rs - セーブ/ロード時

**依存**: M.2（PlayerInventory Component化）と並行可能

---

### M.8: イベント送信網羅

**ステータス**: 🔄 30% (一部イベントのみ送信)

**現状**:
- ✅ 送信済み: `BlockPlaced`, `MachineStarted`, `MachineCompleted`, `ConveyorTransfer`, `ItemDelivered`
- ❌ 未送信: `BlockBroken`, `MachineSpawned`, `InventoryChanged`, `PlayerSpawned`, 他多数

**目標**: architecture-future.md のイベントカタログを全て実装

**完了条件**:
- [ ] `BlockBroken` - ブロック破壊時に送信
- [ ] `MachineSpawned` - 機械設置時に送信
- [ ] `MachineRemoved` - 機械撤去時に送信
- [ ] `InventoryChanged` - インベントリ変更時に送信
- [ ] `PlayerSpawned` - プレイヤー参加時に送信

**影響ファイル**:
- src/systems/block_operations/breaking.rs - BlockBroken
- src/systems/block_operations/placement.rs - MachineSpawned
- src/player/inventory.rs - InventoryChanged

---

### M.9: GuardedEventWriter 使用

**ステータス**: ❌ 定義のみ（使用0箇所）

**現状**: `GuardedEventWriter` が定義されているが、全てのイベント送信が通常の `EventWriter` を使用

**目標**: イベント循環防止のため、全イベント送信を `GuardedEventWriter` 経由に

**完了条件**:
- [ ] `EventWriter<T>` → `GuardedEventWriter<T>` に置換
- [ ] 循環検出テスト追加

**影響ファイル**:
- src/machines/generic.rs
- src/logistics/conveyor.rs
- src/systems/block_operations/placement.rs
- src/achievements/mod.rs
- 他イベント送信箇所

---

### D.2: 動的ID移行 - 🟡 段階的実施

**ステータス**: ✅ 基盤完成 / 🔄 移行 40% (1,007箇所残り)

#### 完了条件
- [ ] `grep -r 'BlockType' src` が 0件 (現在: 1,007)
- [ ] 全アイテムが `ItemId` で参照される
- [x] セーブデータが文字列ID形式 (V2形式)
- [x] **ConveyorItem** が `item_id: ItemId` を直接保持
- [x] **MachineSlot** が `item_id: Option<ItemId>` を直接保持

#### Phase 1: 基盤 ✅
- [x] `Id<T>` Phantom Type 定義 (`src/core/id.rs`)
- [x] `StringInterner` 実装
- [x] `ItemId`, `MachineId`, `RecipeId`, `FluidId` 型エイリアス
- [x] `BlockType` ↔ `ItemId` 変換ヘルパー (`From`/`TryFrom` trait)
- [x] `items` モジュール (16アイテムの定数関数)
- [x] `Serialize`/`Deserialize`/`Default` 実装
- [x] テスト15個追加

#### Phase 2: GameRegistry拡張 ✅
- [x] `item_by_id()` / `machine_by_id()` API追加
- [x] `all_item_ids()` / `all_machine_ids()` API追加
- [x] `to_item_id()` / `to_block_type()` 変換API
- [x] 旧API (`item()`, `machine()`) を `#[deprecated]` マーク
- [x] テスト7個追加

#### Phase 3: 段階的移行計画（2026-01-08更新）

**方針**: 全箇所一括移行は非現実的。コア構造体の移行完了後、新機能実装時に段階的移行。

| 優先度 | ファイル群 | 箇所数 | 方針 | 状態 |
|--------|----------|--------|------|------|
| 🔴 最優先 | P.0-P.3対象 | ~50 | パニック防止で移行 | ✅ 完了 |
| ✅ 完了 | components/machines.rs | - | ConveyorItem/MachineSlot | ✅ 完了 |
| ✅ 完了 | logistics/conveyor.rs | - | 物流コア | ✅ 完了 |
| 🟡 高 | player/*.rs | 92 | インベントリItemId化 | ❌ 未着手 |
| 🟡 中 | game_spec/*.rs | 174 | 仕様定義のItemId化 | ❌ 未着手 |
| 🟢 低 | その他 | ~360 | 新機能時に順次 | ❌ 未着手 |
| 🔵 残す | block_type.rs | 102 | 描画層で必要 | 維持 |

#### ファイル別詳細（Top 15）

| ファイル | 箇所数 | 優先度 | 備考 |
|----------|--------|--------|------|
| save/format.rs | 212 | ✅ 対応済 | V2形式で文字列ID |
| block_type.rs | 102 | 維持 | 描画層で必要（削除不可） |
| game_spec/registry.rs | 67 | ✅ 対応済 | ItemId API追加済み |
| game_spec/mod.rs | 57 | 中 | Descriptor定義 |
| game_spec/recipes.rs | 50 | 中 | レシピ定義 |
| core/id.rs | 50 | 残す | 変換ヘルパー |
| player/global_inventory.rs | 47 | 高 | 全体インベントリ |
| player/inventory.rs | 45 | 高 | ローカルインベントリ |
| craft/mod.rs | 45 | 中 | クラフトシステム |
| components/mod.rs | 40 | 中 | コンポーネント |
| statistics/mod.rs | 35 | 中 | 統計 |
| world/mod.rs | 32 | 維持 | 描画層（ChunkData） |
| main.rs | 31 | 中 | 初期化 |
| components/machines.rs | - | ✅ 完了 | **ItemId直接保持** |
| logistics/conveyor.rs | - | ✅ 完了 | **ItemId直接保持** |

#### 移行パターン

```rust
// Before: BlockType直接使用
fn process_item(item: BlockType) { ... }

// After: ItemId使用
fn process_item(item: ItemId) {
    // 描画時のみBlockType変換（フォールバック付き）
    let render_type = item.try_into().unwrap_or(BlockType::Stone);
}
```

#### 使用方法

```rust
// 新しいコード（推奨）
use idle_factory::core::items;
let stone = items::stone();  // ItemId

// 変換が必要な場合
let block_type: BlockType = stone.try_into().unwrap();
let item_id: ItemId = block_type.into();

// GameRegistry経由
let desc = registry.item_by_id(items::iron_ore());
let machine = registry.machine_by_id(items::furnace_block());
```

#### 見積もり

| 作業 | 時間 |
|------|------|
| P.0-P.3（パニック防止） | 6-8時間 |
| player/*.rs移行 | 2-3時間 |
| game_spec/*.rs移行 | 3-4時間 |
| その他段階的移行 | 新機能時に順次 |
| block_type.rs削除 | 最後（全移行後） |

---

### D.4: 本体アイテムMod化

**ステータス**: ✅ 基盤実装済み / 🔄 移行 75%

#### 完了条件
- [x] `mods/base/items.toml` に全アイテム定義
- [x] `mods/base/machines.toml` に全機械定義
- [ ] `ITEM_DESCRIPTORS` 定数が空 or 削除
- [x] 起動時にbase Modを最初にロード

#### Phase 1: 基盤 ✅
- [x] `ItemDefinition` struct (`src/modding/data.rs`)
- [x] `MachineDefinition` struct
- [x] `RecipeDefinition` struct
- [x] `DataPack` コンテナ
- [x] TOML/JSONパーサー
- [x] テスト4個

#### Phase 2: TOML作成 ✅
- [x] `mods/base/` ディレクトリ作成
- [x] `mods/base/mod.toml` (Mod情報)
- [x] `mods/base/items.toml` (16アイテム)
- [x] `mods/base/machines.toml` (4機械)
- [x] `mods/base/recipes.toml` (11レシピ)

#### Phase 3: 起動時ロード ✅
- [x] 起動時にbase Modを自動ロード (`ModdingPlugin`)
- [x] `LoadedModData` リソースでデータ保持
- [ ] GameRegistryへの統合（将来）
- [ ] `ITEM_DESCRIPTORS` 定数を削除（D.2移行後）

---

### D.1: イベントシステム

**ステータス**: ✅ 基盤実装済み / ⚠️ リファクタリング予定

> **詳細設計**: `.claude/design-event-system.md` 参照

#### 現状の問題点

| 問題 | 影響 | 深刻度 |
|------|------|--------|
| イベント定義が2箇所（mod.rs, game_events.rs） | AI が片方だけ読んで実装する | 高 |
| Mod API EventType が手動同期 | 新イベント追加時に漏れる | 高 |
| GuardedEventWriter が未使用 | 循環防止が機能していない | 中 |
| MachineCompleted がMod APIで未ブリッジ | Modが購読できない | 中 |

#### 新設計（実装予定）

```
src/events/
├── mod.rs              # 再エクスポート + EventsPlugin
├── core.rs             # 【新規】全コアイベント定義（Single Source of Truth）
├── types.rs            # EventSource, CoreEventKind 等の共通型
├── mod_event.rs        # 【新規】Mod独自イベント（動的）
├── subscriptions.rs    # 購読管理
├── bridge.rs           # 【新規】Mod API自動ブリッジ
└── guarded_writer.rs   # GuardedEventWriter（既存）
```

#### コアイベント一覧（19種）

| カテゴリ | イベント | 用途 |
|----------|----------|------|
| **ブロック** | BlockPlacing, BlockPlaced, BlockBreaking, BlockBroken | ブロック操作 |
| **機械** | MachineSpawned, MachineStarted, MachineCompleted, MachineFuelConsumed, MachineRemoved | 機械状態 |
| **プレイヤー** | PlayerSpawned, PlayerMoved, InventoryChanged | プレイヤー操作 |
| **物流** | ConveyorTransfer, ItemPickedUp, ItemDropped, ItemDelivered | アイテム移動 |
| **クエスト** | QuestStarted, QuestProgressed, QuestCompleted | クエスト進行 |

#### AI安全性の担保

1. **Single Source of Truth**: `core.rs` に全コアイベント定義
2. **自動ブリッジ**: `bridge_core_event!` マクロで Mod API 自動対応
3. **網羅性テスト**: `core_event_kind_count_matches` でイベント追加漏れ検出
4. **コメントによるガイド**: AI向けの手順を明記

#### 新イベント追加時の作業

| 変更前（4ファイル） | 変更後（1ファイル + マクロ2箇所） |
|---------------------|----------------------------------|
| events/mod.rs | core.rs に構造体追加 |
| events/game_events.rs | CoreEventKind enum に追加 |
| modding/handlers/events.rs | register_core_events! に追加 |
| modding/event_bridge.rs | register_all_bridges() に追加 |

#### Modイベント

```rust
// Mod独自のイベント（動的、型なし）
pub struct ModEvent {
    pub event_id: String,  // "my_mod:custom_explosion"
    pub data: serde_json::Value,
    pub source: EventSource,
}
```

**Modは**:
- コアイベントを購読可能
- 独自イベントを発火可能
- 他Modのイベントも購読可能

#### 移行計画

| Phase | 内容 | 状態 |
|-------|------|------|
| Phase 1 | core.rs 作成、旧イベント統合 | ❌ 未着手 |
| Phase 2 | 既存コードを新イベント名に更新 | ❌ 未着手 |
| Phase 3 | 互換レイヤー削除、テスト追加 | ❌ 未着手 |

#### 既存の送信/購読状況

**イベント送信**:
| イベント | 送信箇所 | 状態 |
|----------|----------|------|
| BlockPlaced | placement.rs | ✅ |
| BlockBroken | breaking.rs | ✅ |
| MachineSpawned | placement.rs | ✅ |
| MachineStarted | generic.rs | ✅ |
| MachineCompleted | generic.rs | ✅ |
| ConveyorTransfer | conveyor.rs | ✅ |
| ItemDelivered | conveyor.rs | ✅ |
| InventoryChanged | - | ❌ 未実装 |

**イベント購読**:
- [x] 統計システム (`statistics/mod.rs`)
- [x] 実績システム (`achievements/mod.rs`)
- [ ] Mod API外部通知（設計済み、未実装）

---

### セーブ形式移行

**ステータス**: ✅ 完了

#### 完了条件
- [x] V2形式で保存（文字列ID）
- [x] V1/V2両形式の読み込み対応
- [x] アイテムIDが文字列形式 (`"base:iron_ore"`)
- [ ] `BlockTypeSave` enum 削除（内部変換用に残存、将来削除）

#### Phase 1: 基盤 ✅
- [x] セーブ/ロードシステム (`src/save/`)
- [x] 各種SaveData構造体
- [x] `SAVE_VERSION_V2` 定数
- [x] `BlockTypeSave.to_string_id()` / `from_string_id()`
- [x] `ItemStackV2` 構造体（文字列ID形式）
- [x] `BlockType::to_save_string_id()` / `from_save_string_id()`
- [x] V1 ↔ V2 変換トレイト

#### Phase 2: V2構造体 ✅
- [x] `SaveDataV2` - メインセーブ構造体（全て文字列ID）
- [x] `InventorySaveDataV2`, `GlobalInventorySaveDataV2`
- [x] `WorldSaveDataV2`, `QuestSaveDataV2`
- [x] `MachineSaveDataV2` (Miner/Conveyor/Furnace/Crusher)
- [x] V1 → V2, V2 → V1 変換実装
- [x] `save_game()` がV2形式で出力
- [x] `load_game()` が両形式対応
- [x] テスト4個追加

---

### D.3: Mod API Server - 🟡 WebSocket実装待ち

**ステータス**: ✅ 基盤実装済み / ❌ WebSocket未起動

#### 現状

| 要素 | 状態 | 備考 |
|------|------|------|
| API定義 (18メソッド) | ✅ | `src/modding/api.rs` |
| TOML読み込み | ✅ | `src/modding/data.rs` |
| ModManager | ✅ | `src/modding/mod.rs` |
| WebSocketサーバー | ❌ | **未実装** |
| イベント購読 | ❌ | GuardedEventWriter実装済み、未使用 |

#### 完了条件
- [ ] WebSocketサーバーが起動する
- [ ] 外部プロセスからJSON-RPC接続可能
- [ ] イベント購読・通知が動作する
- [ ] E2Eテスト（Python/JSクライアント）

#### Phase 1: 依存ライブラリ追加

```toml
# Cargo.toml
[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "net", "sync"] }
tokio-tungstenite = "0.21"
futures-util = "0.3"
```

#### Phase 2: WebSocketサーバー実装

**新規ファイル**: `src/modding/server.rs`

```rust
pub struct ModApiServer {
    config: ApiServerConfig,
    connections: HashMap<u64, WebSocketStream>,
    next_conn_id: u64,
}

pub struct ModEventBridge {
    subscribers: HashMap<&'static str, Vec<u64>>, // event_type -> conn_ids
}

// Bevy Plugin
pub struct ModApiServerPlugin;

impl Plugin for ModApiServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ModApiServer::default())
           .insert_resource(ModEventBridge::default())
           .add_systems(Update, mod_api_server_tick);
    }
}
```

#### Phase 3: APIハンドラ実装

**登録済みメソッド（18個）の実装**:

| カテゴリ | メソッド | 実装優先度 |
|---------|---------|-----------|
| ゲーム | `game.version`, `game.state` | 高 |
| Mod | `mod.list`, `mod.info`, `mod.enable`, `mod.disable` | 高 |
| アイテム | `item.list`, `item.add` | 中 |
| 機械 | `machine.list`, `machine.add` | 中 |
| レシピ | `recipe.list`, `recipe.add` | 中 |
| イベント | `event.subscribe`, `event.unsubscribe` | 高 |

#### Phase 4: イベント購読

```rust
// GuardedEventWriterとの統合
fn notify_mod_subscribers(
    mut events: EventReader<BlockPlaced>,
    bridge: Res<ModEventBridge>,
    server: Res<ModApiServer>,
) {
    for event in events.read() {
        bridge.notify("block_placed", &event, &server);
    }
}
```

#### Phase 5: テスト

```python
# E2Eテスト（Python）
import websocket
import json

ws = websocket.create_connection("ws://127.0.0.1:9877")
ws.send(json.dumps({"id": 1, "method": "game.version", "params": {}}))
result = json.loads(ws.recv())
assert result["success"] == True
```

#### 見積もり

| 作業 | 時間 |
|------|------|
| 依存ライブラリ追加 | 30分 |
| WebSocketサーバー基盤 | 2-3時間 |
| APIハンドラ18個 | 3-4時間 |
| イベント購読統合 | 1-2時間 |
| E2Eテスト | 1-2時間 |
| **合計** | **8-12時間** |

#### 影響ファイル

| ファイル | 変更内容 |
|----------|----------|
| `Cargo.toml` | tokio, tokio-tungstenite追加 |
| `src/modding/server.rs` | **新規**: WebSocketサーバー |
| `src/modding/mod.rs` | ModApiServerPlugin追加 |
| `src/modding/api.rs` | ハンドラ実装 |
| `src/plugins/game.rs` | Plugin登録 |

---

### レガシーコード削除

**ステータス**: ✅ 完了

#### 完了条件
- [x] `src/components/machines.rs` から旧struct削除
- [x] `src/save/format.rs` から旧SaveData削除
- [x] `src/debug/state_dump.rs` から旧Dump削除

#### 削除済み
- `pub struct Miner` / `Furnace` / `Crusher`
- `MinerSaveData` / `FurnaceSaveData` / `CrusherSaveData`
- `MinerDump` / `FurnaceDump` / `CrusherDump`
- `InteractingFurnace` / `InteractingCrusher` / `InteractingMiner`

---

### D.6-D.14: 各機能プラグイン登録

**ステータス**: ✅ 完了

#### 完了条件
- [x] 各プラグインがGamePluginで登録される
- [x] ゲーム起動時に機能が有効化

#### 各機能の状態
| # | 機能 | ファイル | Plugin定義 | 登録 |
|---|------|----------|-----------|------|
| D.6 | マップ | src/map/mod.rs | MapPlugin | ✅ |
| D.7 | ブループリント | src/blueprint/mod.rs | BlueprintPlugin | ✅ |
| D.8 | クラフト | src/craft/mod.rs | CraftPlugin | ✅ |
| D.9 | ストレージ | src/storage/mod.rs | StoragePlugin | ✅ |
| D.10 | 統計 | src/statistics/mod.rs | StatisticsPlugin | ✅ |
| D.11 | サウンド | src/audio/mod.rs | AudioPlugin | ✅ |
| D.12 | 実績 | src/achievements/mod.rs | AchievementsPlugin | ✅ |
| D.13 | スキン | src/skin/mod.rs | SkinPlugin | ✅ |
| D.14 | ロボット | src/robot/mod.rs | RobotPlugin | ✅ |
| - | Modding | src/modding/mod.rs | ModdingPlugin | ✅ |

登録箇所: `src/plugins/game.rs:64-73`

---

## Phase D: 基盤強化（実装状況）

| # | タスク | 基盤 | 移行 | 備考 |
|---|--------|------|------|------|
| D.0 | **マルチ準備** | ✅ | ✅ | 完了 |
| D.1 | **イベント** | ✅ | ✅ | 送信7/8、購読7箇所 |
| D.2 | **動的ID** | ✅ | ❌ | 926箇所未移行 |
| D.3 | **Mod API** | ✅ | 🔄 | WebSocket未起動 |
| D.4 | **データMod** | ✅ | ✅ | 起動時ロード実装済み |
| D.5 | **Blockbench** | ✅ | ✅ | 完了 |
| D.6 | **マップ** | ✅ | ✅ | Plugin登録済み |
| D.7 | **ブループリント** | ✅ | ✅ | Plugin登録済み |
| D.8 | **クラフト** | ✅ | ✅ | Plugin登録済み |
| D.9 | **ストレージ** | ✅ | ✅ | Plugin登録済み |
| D.10 | **統計** | ✅ | ✅ | Plugin登録済み |
| D.11 | **サウンド** | ✅ | ✅ | Plugin登録済み |
| D.12 | **実績** | ✅ | ✅ | Plugin登録済み |
| D.13 | **スキン** | ✅ | ✅ | Plugin登録済み |
| D.14 | **ロボット** | ✅ | ✅ | Plugin登録済み |

---

## 安全性レベルと作業計画

### レベル定義

| レベル | 定義 | 必要作業 |
|--------|------|----------|
| **L1** | Mod対応として最低限 | P.0-P.3 |
| **L2** | 外部入力全般に堅牢 | L1 + P.4 + GuardedEventWriter使用 |
| **L3** | 将来拡張も安全 | L2 + P.5 + EntityMap + StringInterner |

### 作業順序と見積もり

```
今すぐ（L1達成）
├── P.0: core/id.rs Result返却        [0.5日]
├── P.1: ConveyorItem, MachineSlot    [0.5日]
├── P.2: Quest, Craft                 [0.5日]
└── P.3: セーブ/ロード フォールバック  [0.5日]
    ↓
D.15着手前（L2達成）
├── P.4: Blockbench エラーハンドリング [0.5日]
└── GuardedEventWriter使用開始        [0.5日]
    ↓
D.15-D.19と並行（L3達成）
├── P.5: ValidItemId導入              [1日]
└── EntityMap実装                     [1日]
    ↓
D.20（マルチ）前
└── StringInternerスレッドセーフ化    [0.5日]

合計: 約5-6日
```

---

## 並列実行計画

### Wave 1（同時開始・4並列）

| サブエージェント | タスク | 時間 | 依存 |
|-----------------|--------|------|------|
| 1 | P.0: core/id.rs | 30分 | なし |
| 2 | P.2a+P.2b: Quest/Craft | 40分 | なし |
| 3 | P.3: Save/Load | 30分 | なし |
| 4 | D.3-A: Cargo.toml + Protocol | 45分 | なし |

### Wave 2（Wave 1 完了後・2並列）

| サブエージェント | タスク | 時間 | 依存 |
|-----------------|--------|------|------|
| 1 | P.1: ConveyorItem/MachineSlot | 2-3h | P.0 |
| 2 | D.3-B: WebSocketサーバー | 2h | D.3-A |

### Wave 3（Wave 2 完了後・6並列）

| サブエージェント | タスク | ファイル |
|-----------------|--------|----------|
| 1 | game.version/state | handlers/game.rs |
| 2 | mod.list/info/enable/disable | handlers/mod_handlers.rs |
| 3 | item.list/add | handlers/items.rs |
| 4 | machine.list/add | handlers/machines.rs |
| 5 | recipe.list/add | handlers/recipes.rs |
| 6 | event.subscribe/unsubscribe | handlers/events.rs |

### Wave 4（Wave 3 完了後・2並列）

| サブエージェント | タスク | 時間 |
|-----------------|--------|------|
| 1 | D.3-D: イベント購読統合 | 1.5h |
| 2 | 統合テスト | 1h |

### 見積もり比較

| 方式 | 時間 |
|------|------|
| 直列実行 | 約12時間 |
| 並列実行（4サブエージェント） | 約4-5時間 |

---

## パニック防止（P.0-P.3）- 🔴 最優先

> 詳細は `.claude/architecture-future.md` の「パニック防止戦略」セクション参照

**目標**: Modアイテム追加・削除でゲームがクラッシュしない設計

### 見積もり比較（2026-01-07 Gemini/Claude合見積）

| 見積元 | 選択肢A (ItemId直接) | 選択肢B (フォールバック) |
|--------|---------------------|------------------------|
| Gemini 2.5 Pro | 8-15日 | 3-5日 |
| Claude | 6-8時間 | - |

**採用方針**: ハイブリッド方式
- P.1 (ConveyorItem/MachineSlot): **ItemId直接保持**（物流の核心、根本解決）
- P.2/P.3: フォールバック方式（初期化時のみ）

### 実装順序と見積もり

```
P.0 → P.1 → P.3 → P.2
 ↓      ↓      ↓     ↓
30分  2-3h   1h   30分
```

| Phase | 対象 | 内容 | 状態 |
|-------|------|------|------|
| **P.0** | core/id.rs | `from_block_type_static`をResult返却に | ✅ 完了 |
| **P.1** | ConveyorItem, MachineSlot | BlockType廃止→**ItemId直接保持** | ✅ 完了 |
| **P.2** | Quest, Craft | フォールバック + warn!() | ✅ 完了 |
| **P.3** | セーブ/ロード | 不明アイテムフィルタリング | ✅ 完了 |
| **P.4** | Blockbench | フォールバックメッシュ | ✅ 完了 |
| **P.5** | ValidItemId | 型安全強化 | ✅ 完了 |

---

### P.0: core/id.rs (30分)

**変更内容**:
```rust
impl ItemId {
    /// 安全なAPI（新規）
    pub fn try_from_block_type_static(block_type: BlockType) -> Option<Self> {
        let name = format!("{}", block_type);
        items::by_name(&name)
    }

    /// 既存API（フォールバック付きに変更）
    pub fn from_block_type_static(block_type: BlockType) -> Self {
        Self::try_from_block_type_static(block_type)
            .unwrap_or_else(|| {
                warn!("BlockType::{:?} not found, using fallback", block_type);
                items::stone()
            })
    }
}
```

**影響ファイル**: `src/core/id.rs`

---

### P.1: ConveyorItem/MachineSlot (2-3時間)

**ConveyorItem 変更**:
```rust
pub struct ConveyorItem {
    item_id: ItemId,  // BlockType → ItemId
    pub progress: f32,
    pub visual_entity: Option<Entity>,
    pub lateral_offset: f32,
}

impl ConveyorItem {
    pub fn new(item_id: ItemId, progress: f32) -> Self {
        Self { item_id, progress, visual_entity: None, lateral_offset: 0.0 }
    }
    pub fn item_id(&self) -> ItemId { self.item_id }
    /// 描画用（Modアイテムは石にフォールバック）
    pub fn block_type_for_render(&self) -> BlockType {
        self.item_id.try_into().unwrap_or(BlockType::Stone)
    }
}
```

**MachineSlot 変更**:
```rust
pub struct MachineSlot {
    item_id: Option<ItemId>,  // BlockType → ItemId
    pub count: u32,
}
impl MachineSlot {
    pub fn add_id(&mut self, item: ItemId, amount: u32) -> u32 { ... }
    pub fn item_id(&self) -> Option<ItemId> { self.item_id }
    pub fn block_type_for_render(&self) -> Option<BlockType> {
        self.item_id.and_then(|id| id.try_into().ok())
    }
}
```

**影響ファイル**:
- `src/components/machines.rs` - 構造体定義
- `src/logistics/conveyor.rs` - 物流ロジック
- `src/machines/generic.rs` - 機械tick処理
- `src/save/systems.rs` - セーブ/ロード

---

### P.2: Quest/Craft (30分)

**変更方針**: パニック → warn!() + Option/スキップ

```rust
// QuestDef::new
pub fn new(...) -> Option<Self> {
    let required_bt = match required_item.try_into() {
        Ok(bt) => bt,
        Err(_) => { warn!("Unknown item"); return None; }
    };
    // rewards も filter_map で不明アイテム除外
    Some(Self { ... })
}

// CraftingRecipeBuilder::input_id
pub fn input_id(mut self, item: ItemId, count: u32) -> Self {
    if let Ok(bt) = item.try_into() {
        self.inputs.push(RecipeInput::new(bt, count, 0));
    } else {
        warn!("Unknown input item, skipping");
    }
    self
}
```

**影響ファイル**:
- `src/components/mod.rs`
- `src/craft/mod.rs`

---

### P.3: Save/Load (1時間)

**変更方針**: 不明アイテムはログ出力してスキップ（除外）

```rust
impl TryFrom<ItemStackV2> for ItemStack {
    fn try_from(v2: ItemStackV2) -> Result<Self, Self::Error> {
        BlockTypeSave::from_string_id(&v2.item_id)
            .map(|item_type| ItemStack { item_type, count: v2.count })
            .ok_or_else(|| {
                warn!("Unknown item '{}' removed from save", v2.item_id);
                format!("Unknown: {}", v2.item_id)
            })
    }
}
```

**影響ファイル**:
- `src/save/format.rs`
- `src/save/systems.rs`

---

### テスト計画（P.1完了後に追加）

```rust
#[test]
fn test_conveyor_item_with_mod_item_no_panic() {
    let mod_item = ItemId::from_string("test_mod:super_ingot", &mut interner);
    let item = ConveyorItem::new(mod_item, 0.0);
    assert_eq!(item.item_id(), mod_item);
    assert_eq!(item.block_type_for_render(), BlockType::Stone); // フォールバック
}

#[test]
fn test_load_unknown_item_filtered() {
    // 不明アイテムがインベントリから除外されることを確認
}
```

---

### 追加タスク（L2-L3用）

| タスク | 内容 | 状態 |
|--------|------|------|
| **GuardedEventWriter使用** | 全イベント送信箇所で使用開始 | ❌ 未着手 |
| **EntityMap実装** | NetworkId ↔ Entity マッピング | ❌ 未着手 |
| **StringInterner安全化** | Arc<RwLock>でスレッドセーフに | ❌ 未着手 |

### 致命的パニック箇所（優先度順）

| 優先度 | ファイル | 問題 |
|--------|----------|------|
| 🔴 P0 | core/id.rs:191 | `from_block_type_static`が無条件パニック |
| 🔴 P0 | machines.rs:93,119 | ConveyorItemでModアイテム即死 |
| 🔴 P0 | machines.rs:375,383 | MachineSlotでModアイテム即死 |
| 🔴 P1 | mod.rs:356,363 | QuestでModアイテム即死 |
| 🔴 P1 | craft/mod.rs:131,150 | レシピでModアイテム即死 |
| 🔴 P1 | save/format.rs:1660 | 不明ItemIDでパニック |
| 🟡 P2 | blockbench.rs:907,945等 | JSON/Base64パース失敗でパニック |

### 設計と実装の乖離（要修正）

| 設計書の記述 | 現状 | 差異 | 対応Phase |
|------------|------|------|-----------|
| パニック防止戦略 | 未着手 | ❌ 完全未実装 | P.0-P.4 |
| GuardedEventWriter | 実装済み | ⚠️ 未使用 | L2 |
| EntityMap | 未実装 | ❌ LocalPlayerがEntity直接参照 | L3 |
| ValidItemId | 未実装 | ❌ ItemIdは未検証 | P.5 |
| StringInterner | thread_local | ⚠️ マルチ非対応 | L3 |

### 不足テスト（P.1完了後に追加）

- `mod_item_on_conveyor_no_panic` - Modアイテムがコンベア通過
- `save_with_unknown_item_has_fallback` - Mod削除後セーブ読込
- `malformed_blockbench_returns_error` - 不正ファイルでエラー返却

---

## D.3: Mod API Server - WebSocket実装

**ステータス**: ✅ 基盤実装済み / ❌ WebSocket未起動

### 完了条件
- [ ] WebSocketサーバーが起動する
- [ ] 外部プロセスからJSON-RPC接続可能
- [ ] イベント購読・通知が動作する
- [ ] E2Eテスト（Python/JSクライアント）

### 現状

| 要素 | 状態 | ファイル |
|------|------|----------|
| API定義 (18メソッド) | ✅ | `src/modding/api.rs` |
| データ構造 | ✅ | `src/modding/data.rs` |
| ModManager | ✅ | `src/modding/mod.rs` |
| イベント基盤 | ✅ | `src/events/guarded_writer.rs` |
| WebSocketサーバー | ❌ | 未実装 |

### 実装計画

#### D.3-A: 依存追加 + Protocol (45分)

**Cargo.toml**:
```toml
tokio = { version = "1", features = ["rt-multi-thread", "net", "sync", "macros"] }
tokio-tungstenite = "0.21"
futures-util = "0.3"
```

**新規ファイル**:
- `src/modding/protocol.rs` - JsonRpcRequest/Response/Notification
- `src/modding/connection.rs` - ModConnection, ConnectionManager

#### D.3-B: WebSocketサーバー (2h)

**新規ファイル**: `src/modding/server.rs`

**実装方針**:
- crossbeam_channel でBevyメインスレッドと通信
- 別スレッドで tokio runtime 起動
- ServerMessage/ClientMessage enum

**変更ファイル**: `src/modding/mod.rs`
- ModApiServerPlugin 追加
- process_server_messages システム

#### D.3-C: ハンドラ18メソッド (1.5h)

**新規ディレクトリ**: `src/modding/handlers/`

| メソッド | ファイル |
|----------|----------|
| game.version, game.state | handlers/game.rs |
| mod.list/info/enable/disable | handlers/mod_handlers.rs |
| item.list, item.add | handlers/items.rs |
| machine.list, machine.add | handlers/machines.rs |
| recipe.list, recipe.add | handlers/recipes.rs |
| event.subscribe/unsubscribe | handlers/events.rs |

#### D.3-D: イベント購読統合 (1.5h)

**新規ファイル**: `src/modding/event_bridge.rs`

- EventReader で各イベントを読み取り
- 購読者にJSON-RPC通知を送信
- ConveyorTransfer等は除外リスト

### 見積もり

| 作業 | 時間 |
|------|------|
| D.3-A 依存+Protocol | 45分 |
| D.3-B WebSocketサーバー | 2時間 |
| D.3-C ハンドラ18個 | 1.5時間 |
| D.3-D イベント購読 | 1.5時間 |
| **合計** | **6時間** |

---

## 次のタスク（基盤強化続き）

> 参照: `/home/bacon/.claude/plans/expressive-brewing-metcalfe.md`

### 完了済み（2026-01-08）

| タスク | 状態 | 備考 |
|--------|------|------|
| P.0-P.3 パニック防止 | ✅ | **実装済み**（フォールバック付き） |
| D.3 WebSocket API | 🔄 95% | **後回し**（EventSubscriptions初期化のみ残り） |
| BlockType→ItemId移行 | 🔄 60% | **後回し**（完全排除不可能、レンダリング層で必要） |

**P.0-P.3 実装箇所**:
- P.0: `core/id.rs:197-205` - `from_block_type_static`にフォールバック（stone）
- P.1: `components/machines.rs` - ConveyorItem/MachineSlotがItemId直接保持
- P.2: Quest/Craftにpanicなし（確認済み）
- P.3: `save/format.rs:652-825` - 不明アイテムは警告+フィルタリング

### 次の優先タスク

| # | タスク | 内容 | 優先度 |
|---|--------|------|--------|
| T.1 | **固定 Tick 導入** | FixedUpdate(20tick/s) + 描画補間 | 🔴 高 |
| T.2 | **WASM ローダー基盤** | Wasmtime 統合（将来の Core Mod 用） | 🟡 中 |
| T.3 | **新ゲーム機能** | 電力、液体、など | 🟢 機能次第 |

---

### T.1: 固定 Tick システム導入

**目的**: マルチプレイ、大規模工場、Script Mod 対応の基盤

**変更内容**:
```rust
// main.rs または plugins/game.rs
app.insert_resource(Time::<Fixed>::from_hz(20.0)); // 20 tick/秒

// ゲームロジック → FixedUpdate
app.add_systems(FixedUpdate, (
    generic_machine_tick,
    conveyor_transfer,
    // ... 他のロジック
));

// 描画 → Update（補間付き）
app.add_systems(Update, (
    interpolate_conveyor_items,
    render_ui,
));
```

**影響ファイル**:
- `src/plugins/game.rs` - スケジュール変更
- `src/logistics/conveyor.rs` - 補間ロジック追加
- `src/machines/generic.rs` - FixedUpdate に移動

**見積もり**: 2-3時間

---

## Phase D.15-D.20: 高度機能（次フェーズ）

| # | タスク | 内容 | 状態 |
|---|--------|------|------|
| D.15 | **電力** | 電力網、発電機、消費機械 | ❌ 未着手 |
| D.16 | **液体・気体** | パイプ、タンク、ポンプ | ❌ 未着手 |
| D.17 | **信号制御** | ワイヤー、ゲート、センサー | ❌ 未着手 |
| D.18 | **線路** | レール、列車、駅 | ❌ 未着手 |
| D.19 | **Mob** | NPC、敵、AI | ❌ 未着手 |
| D.20 | **マルチプレイ** | P2P/サーバー、同期 | ❌ 未着手 |

---

## 完了済みPhase

### Phase C: データ駆動設計 ✅ (2026-01-07)

| 追加するもの | 以前 | 現在 |
|--------------|------|------|
| 新アイテム | 100行 | **8行** (ItemDescriptor) |
| 新機械 | 500行 | **20行** (MachineSpec) |
| 新レシピ | 5行 | 5行 |

### Phase B: アーキテクチャ再設計 ✅

- 物流分離: `logistics/conveyor.rs`
- 機械統合: `machines/generic.rs`
- UI統合: `UIState`, `UIAction`, `UIContext`

### Phase A: v0.2完成 ✅

- UIテーマ刷新
- バイオーム表示UI
- チュートリアル

---

## 新コンテンツ追加フロー

### 現在（Rustハードコード）
```
1. BlockType enumに追加
2. game_spec/registry.rs にItemDescriptor追加（8行）
3. game_spec/machines.rs にMachineSpec追加（20行）
4. game_spec/recipes.rs にレシピ追加（5行）
5. assets/models/ に3Dモデル配置
```

### 目標（TOML駆動）
```
1. mods/base/items.toml に追加（3行）
2. mods/base/machines.toml に追加（10行）
3. mods/base/recipes.toml に追加（3行）
4. assets/models/ に3Dモデル配置
5. 完了（Rustコード変更なし）
```

---

*最終更新: 2026-01-08*
