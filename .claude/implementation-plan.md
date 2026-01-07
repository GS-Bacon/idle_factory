# 実装計画

> 詳細な将来設計は `.claude/architecture-future.md` 参照
> ロードマップは `.specify/roadmap.md` 参照
> **移行状況確認**: `./scripts/migration-status.sh`

## 現状サマリー

| 項目 | 値 |
|------|-----|
| コード行数 | **~25,000行** |
| テスト | **485件** 通過 |
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

### D.2: 動的ID移行

**ステータス**: ✅ 基盤完成 / 🔄 移行 10% (970箇所残り)

#### 完了条件
- [ ] `grep -r 'BlockType' src` が 0件 (現在: 970)
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

#### Phase 3: 段階的移行 (後回し推奨)

**注**: 970箇所の移行は大規模作業。新機能実装時に段階的に移行推奨。

| ファイル | 箇所数 | 優先度 |
|----------|--------|--------|
| save/format.rs | 212 | ✅ V2形式で対応済み |
| block_type.rs | 102 | 最後（enum定義自体） |
| game_spec/registry.rs | 67 | ✅ ItemId API追加済み |
| game_spec/mod.rs | 57 | 中 |
| core/id.rs | 50 | 変換ヘルパー（残す） |
| game_spec/recipes.rs | 46 | 中 |
| player/*.rs | ~70 | 中 |
| その他 | ~360 | 低 |

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
