# 実装計画

> 詳細な将来設計は `.claude/architecture-future.md` 参照
> ロードマップは `.specify/roadmap.md` 参照
> **移行状況確認**: `./scripts/migration-status.sh`

## 現状サマリー

| 項目 | 値 |
|------|-----|
| コード行数 | **~24,000行** |
| テスト | **493件** 通過 |
| Clippy警告 | **0件** |
| Phase | **D.0-D.14 基盤実装済み** |

---

## 未完了の移行作業（優先度順）

> **注意**: 以下は「基盤は作ったが移行していない」タスク

| # | タスク | 基盤 | 移行 | 残作業 |
|---|--------|------|------|--------|
| D.2 | **動的ID** | ✅ | ❌ 0% | BlockType → ItemId (940箇所) |
| D.4 | **本体Mod化** | ✅ | ✅ 100% | 起動時ロード完了 |
| D.1 | **イベント** | ✅ | ✅ 100% | 7箇所でEventReader使用 |
| - | **セーブ形式** | ✅ | ✅ 100% | V2形式で保存、両形式読込対応 |
| - | **レガシー削除** | ✅ | ✅ | 完了 |
| D.6-14 | **各機能プラグイン** | ✅ | ✅ | 完了 |

---

### D.2: 動的ID移行 - 🟡 段階的実施

**ステータス**: ✅ 基盤完成 / 🔄 移行 10% (1,121箇所残り)

#### 完了条件
- [ ] `grep -r 'BlockType' src` が 0件 (現在: 1,121)
- [ ] 全アイテムが `ItemId` で参照される
- [x] セーブデータが文字列ID形式 (V2形式)

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

#### Phase 3: 段階的移行計画（2026-01-07策定）

**方針**: 全箇所一括移行は非現実的。P.0-P.3完了後、新機能実装時に段階的移行。

| 優先度 | ファイル群 | 箇所数 | 方針 | 状態 |
|--------|----------|--------|------|------|
| 🔴 最優先 | P.0-P.3対象 | ~50 | パニック防止で移行 | ❌ 未着手 |
| 🟡 高 | player/*.rs | 92 | インベントリItemId化 | ❌ 未着手 |
| 🟡 中 | game_spec/*.rs | 174 | 仕様定義のItemId化 | ❌ 未着手 |
| 🟢 低 | その他 | ~360 | 新機能時に順次 | ❌ 未着手 |
| 🔵 最後 | block_type.rs | 102 | enum定義削除 | ❌ 最後 |

#### ファイル別詳細（Top 15）

| ファイル | 箇所数 | 優先度 | 備考 |
|----------|--------|--------|------|
| save/format.rs | 212 | ✅ 対応済 | V2形式で文字列ID |
| block_type.rs | 102 | 最後 | enum定義自体 |
| game_spec/registry.rs | 67 | ✅ 対応済 | ItemId API追加済み |
| game_spec/mod.rs | 57 | 中 | Descriptor定義 |
| game_spec/recipes.rs | 50 | 中 | レシピ定義 |
| core/id.rs | 50 | 残す | 変換ヘルパー |
| player/global_inventory.rs | 47 | 高 | 全体インベントリ |
| player/inventory.rs | 45 | 高 | ローカルインベントリ |
| craft/mod.rs | 45 | P.2 | クラフトシステム |
| components/mod.rs | 40 | 中 | コンポーネント |
| statistics/mod.rs | 35 | 中 | 統計 |
| world/mod.rs | 32 | 中 | ワールド |
| main.rs | 31 | 中 | 初期化 |
| components/machines.rs | 28 | P.1 | **最重要** |
| logistics/conveyor.rs | 19 | P.1 | 物流コア |

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

**ステータス**: ✅ 基盤実装済み / ✅ 購読 100%

#### 完了条件
- [x] 主要イベントが実際に送信される
- [x] EventReaderで処理される
- [ ] Mod APIがイベントを購読可能（将来）

#### Phase 1: 基盤 ✅
- [x] `EventSystemConfig`, `EventDepth` (`src/events/mod.rs`)
- [x] `GuardedEventWriter` 循環防止
- [x] イベント型8個定義 (`src/events/game_events.rs`)
  - BlockPlaced, BlockBroken
  - MachineSpawned, MachineStarted, MachineCompleted
  - InventoryChanged, ConveyorTransfer, ItemDelivered
- [x] その他イベント18個（Mod, Command, Craft等）
- [x] `GameEventsExtPlugin` でadd_event登録

#### Phase 2: イベント送信 ✅ (8/8 - InventoryChangedのみ未実装)
| イベント | 送信箇所 | 状態 |
|----------|----------|------|
| BlockPlaced | placement.rs | ✅ |
| BlockBroken | breaking.rs:374 | ✅ |
| MachineSpawned | placement.rs | ✅ |
| MachineStarted | generic.rs | ✅ |
| MachineCompleted | generic.rs | ✅ |
| InventoryChanged | - | ❌ 未実装（複雑） |
| ConveyorTransfer | conveyor.rs | ✅ |
| ItemDelivered | conveyor.rs | ✅ |

#### Phase 3: イベント購読 ✅
- [x] 統計システムがイベントを購読 (`statistics/mod.rs`)
  - `MachineCompleted` → 生産統計
  - `MachineStarted` → 消費統計
  - `ItemDelivered` → 納品統計
- [x] 実績システムがイベントを購読 (`achievements/mod.rs`)
  - `MachineSpawned` → 機械設置カウント
  - `BlockPlaced` → ブロック設置カウント
  - `MachineCompleted` → 生産カウント
  - `ItemDelivered` → 納品カウント
- [ ] Mod APIがイベントを外部通知（将来）

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
| **P.0** | core/id.rs | `from_block_type_static`をResult返却に | ❌ 未着手 |
| **P.1** | ConveyorItem, MachineSlot | BlockType廃止→**ItemId直接保持** | ❌ 未着手 |
| **P.2** | Quest, Craft | フォールバック + warn!() | ❌ 未着手 |
| **P.3** | セーブ/ロード | 不明アイテムフィルタリング | ❌ 未着手 |
| **P.4** | Blockbench | フォールバックメッシュ | ❌ 未着手 |
| **P.5** | ValidItemId | 型安全強化 | ❌ 未着手 |

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

*最終更新: 2026-01-07*
