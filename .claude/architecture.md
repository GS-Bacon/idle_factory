# アーキテクチャ設計

> **対象**: AI・人間（両方）
> **役割**: 機能追加時の設計ガイド、拡張ポイントの参照
> **タスク詳細**: `.claude/implementation-plan.md`
> **進捗確認**: `.specify/roadmap.md`

## ドキュメント役割分担

| ファイル | 対象 | 内容 |
|----------|------|------|
| `.specify/roadmap.md` | 人間 | マイルストーン概要、完了条件 |
| `.claude/implementation-plan.md` | AI | タスク詳細、シナリオテスト例 |
| **このファイル** | 両方 | 機能設計骨格、拡張ポイント |

## 目次

| セクション | 内容 | 読むタイミング |
|------------|------|----------------|
| [確定判断](#確定した設計判断) | 変更不可の根本決定 | 必ず |
| [設計原則](#設計原則) | 動的ID、マルチ、Mod、イベント | 必ず |
| [機能リスト](#機能リスト) | 全17機能の一覧 | 計画時 |
| [現在のアーキテクチャ](#現在のアーキテクチャ) | ディレクトリ構成 | 新規追加時 |
| [各機能骨格](#各機能の設計骨格) | 機能別の詳細設計 | 該当機能実装時 |
| [機能依存関係](#機能依存関係) | 実装順序の依存 | 計画時 |
| [共通拡張](#共通拡張ポイント) | IoPort、Network等 | 複数機能に跨る時 |
| [AI向けガイド](#ai向けガイド) | 追加方法、保護機構 | 実装時 |

---

## 設計チェックリスト

新機能追加時の必須チェック:

| # | チェック | 質問 |
|---|----------|------|
| 1 | 境界条件 | base↔Mod、信頼↔非信頼の境界は？ |
| 2 | 失敗ケース | 未知ID、欠落データでどうなる？ |
| 3 | パニック監査 | unwrap/expect は本当に必要？ |
| 4 | フォールバック | 失敗時のデフォルト動作は？ |

**アンチパターン**: 「基盤作ったから完了」→移行完了条件を明記せよ

---

## 機能リスト

| # | 機能 | 概要 | 既存影響度 |
|---|------|------|-----------|
| **基盤システム** | | | |
| 1 | 電力システム | 発電機→電線→機械の電力供給 | 中 |
| 2 | 液体・気体 | パイプ、タンク、流体処理機械 | 中 |
| 3 | 信号制御 | レッドストーン的な論理回路 | 中 |
| 4 | クラフトシステム | プレイヤー手動クラフト | 中 |
| **物流・輸送** | | | |
| 5 | 線路機能 | 列車による長距離輸送 | 中 |
| 6 | ストレージ | 大容量倉庫、フィルター、自動整理 | 中 |
| **自動化・AI** | | | |
| 7 | ロボット | データ駆動プログラム（飛行含む） | 低〜中 |
| 8 | Mob追加 | 敵/友好NPC | 中 |
| **UI・UX** | | | |
| 9 | マップ機能 | ワールド俯瞰表示 | 低 |
| 10 | ブループリント | 配置パターンの保存・再利用 | 低 |
| 11 | 統計・分析 | 生産量グラフ、ボトルネック表示 | 低 |
| 12 | サウンド | BGM、効果音、環境音 | 低 |
| 13 | 実績システム | Steam実績連携 | 低 |
| **カスタマイズ** | | | |
| 14 | プレイヤースキン | マイクラ風の見た目変更 | 低 |
| 15 | モデルインポート | Blockbench直接インポート | 中 |
| **拡張性** | | | |
| 16 | Modding対応 | フルアクセス（Factorioレベル） | 高 |
| 17 | マルチプレイ基盤 | 複数プレイヤーの同期 | 高 |

**設計方針**:
- 研究ツリーはクエストと統合（レシピは基本アンロック済み、任意ロック可能）
- 資源枯渇なし（Modで対応可能に）
- 天候・昼夜は将来拡張可能な設計に（今は実装しない）

---

## 確定した設計判断

> 変更する場合は十分な理由が必要

| 判断 | 決定 | 理由 |
|------|------|------|
| **ID方式** | 動的ID + Phantom Type | 型安全 + Mod対応 |
| **Mod構成** | ハイブリッド（WASM + WebSocket + TOML） | ロジック拡張 + 言語自由 + データ定義 |
| **base** | 本体に内蔵（Engine + base 一体） | シンプル、開発速度優先 |
| **マルチ** | 確定実装、今すぐComponent化 | 後からは困難 |
| **イベント** | 全フック（設計付き） | マルチ・Mod・デバッグ全てに必要 |

### Modレイヤー

| レイヤー | 実行方式 | 言語 | 用途 |
|---------|---------|------|------|
| **Data Mod** | TOML読み込み | - | アイテム/機械/レシピ |
| **Script Mod** | WebSocket | Python, Lua, JS | イベントフック |
| **Core Mod** | WASM（Wasmtime） | Rust, C++ | ロジック追加・置換 |

---

## 設計原則

### 1. 動的ID

`Id<Category>` パターン: Phantom Type で型安全、Registry 経由でのみ生成

```rust
pub type ItemId = Id<ItemCategory>;      // アイテム
pub type MachineId = Id<MachineCategory>; // 機械
pub type RecipeId = Id<RecipeCategory>;   // レシピ
```

**保証**: カテゴリ混同→コンパイルエラー / 存在保証→Registry経由 / Mod対応→実行時追加可能

**セーブ**: 文字列ID（`"base:iron_ore"`）で保存、ロード時に再マッピング。不明IDは警告＋フォールバック。

---

### 2. マルチプレイ基盤

| 原則 | 内容 |
|------|------|
| プレイヤーデータはComponent | `Inventory` Component |
| Entity参照はNetworkId経由 | `EntityMap` で相互変換 |
| 状態変更はイベント経由 | 全フック設計 |

**現在のパターン**: `LocalPlayer(Entity)` + Query でアクセス

---

### 3. Modding アーキテクチャ

```
Engine + base (Rust) ─┬─ Core Mod (WASM)     ← ロジック追加
                      ├─ Script Mod (WebSocket) ← イベントフック
                      └─ Data Mod (TOML)     ← アイテム/機械定義
```

**設計判断**: base は本体内蔵、Core Mod で新ロジック → Data Mod で拡張可能

---

### 4. イベントシステム

**全状態変更をイベント経由に。** マルチ同期・Modフック・デバッグに必要。

**イベントカタログ**:
| カテゴリ | イベント例 |
|----------|-----------|
| ブロック | `BlockPlaced`, `BlockBroken` |
| 機械 | `MachineSpawned`, `MachineCompleted` |
| プレイヤー | `InventoryChanged`, `PlayerSpawned` |
| 物流 | `ConveyorTransfer`, `ItemDelivered` |
| クエスト | `QuestCompleted` |

**循環防止**: `GuardedEventWriter` で深さチェック（max_depth: 16）

**高頻度除外**: `ConveyorTransfer`, `PlayerMoved` は外部Mod通知OFF

**外部通知**: `ModEventBridge` でWebSocket経由

---

### 5. 固定Tick

20 tick/秒（50ms/tick）。FixedUpdate で機械・コンベア処理、Update で描画。

**理由**: マルチ同期、大規模工場の負荷軽減、決定論的再現

---

### 6. Script Mod 制限

| 制限 | 理由 |
|------|------|
| 非同期のみ | フリーズ防止 |
| 100ms タイムアウト | ハング防止 |
| 100 req/s レート制限 | DoS防止 |

**Core vs Script**: 毎Tick処理→Core、イベント反応→Script

---

### 7. Mod API（port 9877）

**エンドポイント**: Subscribe/Unsubscribe、GetRegistry、GetEntity、RegisterItem 等

**バージョニング**: セマンティックバージョニング、ハンドシェイクで互換性チェック

---

## 現在のアーキテクチャ

```
components/     ← ECSコンポーネント（データ）
    machines.rs     Machine, Conveyor, MachineSlot

game_spec/      ← 仕様定義（Single Source of Truth）
    machines.rs     MachineSpec, IoPort
    recipes.rs      RecipeSpec

machines/       ← 機械処理ロジック
    generic.rs      generic_machine_tick()

logistics/      ← 物流ロジック
    conveyor.rs     conveyor_transfer()

world/          ← チャンク・ワールド管理
    mod.rs          ChunkData, WorldData
```

---

## 各機能の設計骨格

### 1. 電力システム

**データ構造**
```rust
// components/power.rs
pub struct PowerConsumer {
    pub required_watts: u32,
    pub is_powered: bool,
}

pub struct PowerProducer {
    pub output_watts: u32,
    pub fuel_slot: Option<MachineSlot>,
}

// グリッドは自動計算（接続されたノードをグラフ探索）
```

**拡張ポイント**
- `MachineSpec` に `power_consumption: Option<u32>` 追加
- `generic_machine_tick()` で `is_powered` チェック追加
- 新モジュール: `logistics/power_grid.rs`

**既存変更**
- `IoPort` に `PortType::Power` 追加

---

### 2. 液体・気体

**データ構造**
```rust
// components/fluids.rs
pub struct FluidSlot {
    pub fluid_type: Option<FluidType>,
    pub amount_mb: u32,      // ミリバケット
    pub capacity_mb: u32,
}

pub struct Pipe {
    pub position: IVec3,
    pub fluid: Option<FluidType>,
    pub pressure: f32,
}

// game_spec/fluids.rs
pub struct FluidSpec {
    pub id: &'static str,
    pub viscosity: f32,      // 流速に影響
    pub temperature: f32,    // 熱処理用
}
```

**拡張ポイント**
- 新モジュール: `logistics/pipe.rs`（conveyor.rsと並列）
- `IoPort` に `PortType::Fluid` 追加
- `MachineSpec.slots` に `SlotType::Fluid` 追加

---

### 3. 信号制御（レッドストーン的）

**データ構造**
```rust
// components/signals.rs
pub struct SignalEmitter {
    pub signal_strength: u8,  // 0-15
    pub condition: SignalCondition,
}

pub struct SignalReceiver {
    pub threshold: u8,
    pub is_active: bool,
}

pub enum SignalCondition {
    Always,
    InventoryFull,
    InventoryEmpty,
    HasItem(ItemId),  // 動的ID
    PowerLow,
    Timer { interval_secs: f32 },
}

// 信号線
pub struct SignalWire {
    pub position: IVec3,
    pub strength: u8,
}
```

**拡張ポイント**
- 新モジュール: `logistics/signals.rs`
- 機械に `signal_input: bool` で動作ON/OFF
- 論理ゲート: AND, OR, NOT, XOR ブロック

**信号の伝播**
```
センサー（在庫検知等）
    ↓ signal_strength: 15
信号線（減衰: 1/ブロック）
    ↓ strength: 12
論理ゲート（AND等）
    ↓
機械（signal_input=true で動作）
```

---

### 4. クラフトシステム

**データ構造**
```rust
// components/crafting.rs
pub struct CraftingGrid {
    pub slots: [Option<(ItemId, u32)>; 9],  // 3x3
    pub result: Option<(ItemId, u32)>,
}

// game_spec/recipes.rs に追加
pub struct CraftingRecipe {
    pub pattern: Vec<String>,  // ["III", " S ", " S "]
    pub ingredients: HashMap<char, ItemId>,
    pub result: (ItemId, u32),
    pub unlocked: bool,  // デフォルトtrue、任意ロック可能
}
```

**拡張ポイント**
- `ui/crafting.rs` でクラフトUI
- クエスト報酬で `unlocked` を変更可能
- Modでレシピ追加/ロック変更可能

---

### 5. 線路機能

**データ構造**
```rust
// components/trains.rs
pub struct Rail {
    pub position: IVec3,
    pub rail_type: RailType,  // Straight, Curve, Switch
    pub signal_block_id: u32, // 閉塞区間
}

pub struct Train {
    pub cars: Vec<TrainCar>,
    pub speed: f32,
    pub route: Vec<IVec3>,
}

pub struct TrainStation {
    pub position: IVec3,
    pub schedule: Vec<ScheduleEntry>,
}
```

**拡張ポイント**
- 新モジュール: `logistics/rail.rs`
- 信号制御との連携（閉塞信号）
- 列車はEntityの集合（機関車 + 貨車）

---

### 6. ストレージ

**データ構造**
```rust
// components/storage.rs
pub struct StorageBlock {
    pub capacity: u32,
    pub slots: Vec<MachineSlot>,
    pub filter: Option<Vec<ItemId>>,  // 許可アイテムリスト（動的ID）
    pub priority: i32,  // 入出力優先度
}

pub struct StorageNetwork {
    pub storages: Vec<Entity>,
    pub total_capacity: u32,
    pub auto_sort: bool,
}
```

**拡張ポイント**
- 倉庫ブロック（小/中/大）
- フィルター設定UI
- コンベアとの接続
- 自動整理機能

**依存関係**: ストレージはコンベアシステム（`logistics/conveyor.rs`）に依存。コンベアのIoPort接続ロジックを再利用。

---

### 7. ロボット（データ駆動）

**設計方針**: プリセット動作をデータ駆動で定義。Mod APIで動的に追加・変更可能。

**データ構造**
```rust
// components/robots.rs
pub struct Robot {
    pub position: Vec3,  // 浮動小数点（飛行対応）
    pub inventory: Vec<MachineSlot>,
    pub program_id: RobotProgramId,  // データ駆動
    pub program_params: RobotProgramParams,
    pub state: RobotState,
    pub can_fly: bool,
    pub fuel: f32,
}

// game_spec/robots.rs
pub struct RobotProgramSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub required_params: &'static [ParamDef],
}

/// パラメータ定義（UI自動生成用）
pub struct ParamDef {
    pub name: &'static str,
    pub param_type: ParamType,  // Position, Entity, ItemFilter, etc.
}

/// 実行時パラメータ
pub struct RobotProgramParams {
    pub values: HashMap<String, ParamValue>,
}

pub enum ParamValue {
    Position(IVec3),
    Entity(Entity),
    ItemFilter(Vec<ItemId>),
    Number(f32),
}

pub enum RobotState {
    Idle,
    Moving { target: Vec3 },
    Mining { target: IVec3, progress: f32 },
    Crafting,
    Flying { target: Vec3, altitude: f32 },
}
```

**組み込みプログラム**
```rust
// game_spec/robots.rs
pub const ROBOT_PROGRAMS: &[RobotProgramSpec] = &[
    RobotProgramSpec {
        id: "mine_and_deliver",
        name: "採掘→納品",
        required_params: &[
            ParamDef { name: "target", param_type: ParamType::Position },
            ParamDef { name: "delivery", param_type: ParamType::Entity },
        ],
    },
    RobotProgramSpec {
        id: "patrol_route",
        name: "巡回",
        required_params: &[
            ParamDef { name: "waypoints", param_type: ParamType::PositionList },
        ],
    },
    RobotProgramSpec {
        id: "filter_transfer",
        name: "フィルター転送",
        required_params: &[
            ParamDef { name: "from", param_type: ParamType::Entity },
            ParamDef { name: "to", param_type: ParamType::Entity },
            ParamDef { name: "filter", param_type: ParamType::ItemFilter },
        ],
    },
];
```

**Mod APIでの拡張**
```json
// 新プログラム登録
{
    "action": "register_robot_program",
    "program": {
        "id": "mymod:smart_miner",
        "name": "スマート採掘",
        "required_params": [
            { "name": "area", "type": "PositionRange" },
            { "name": "priority", "type": "ItemFilter" }
        ]
    }
}

// ロボットにプログラム設定
{
    "action": "set_robot_program",
    "robot_id": "uuid",
    "program_id": "mymod:smart_miner",
    "params": {
        "area": { "min": [0, 0, 0], "max": [10, 10, 10] },
        "priority": ["base:diamond_ore", "base:iron_ore"]
    }
}
```

**拡張ポイント**
- 新モジュール: `systems/robot.rs`
- `game_spec/robots.rs` でプログラム定義
- UIはパラメータ定義から自動生成
- Mod APIで新プログラム追加可能

---

### 8. Mob追加

**データ構造**
```rust
// components/mobs.rs
pub struct Mob {
    pub mob_type: MobType,
    pub health: f32,
    pub ai_state: AIState,
}

pub enum AIState {
    Idle,
    Wandering,
    Chasing(Entity),
    Fleeing,
    Working,  // 友好NPCの作業
}

// game_spec/mobs.rs
pub struct MobSpec {
    pub id: String,  // 動的ID（例: "base:zombie"）
    pub max_health: f32,
    pub speed: f32,
    pub hostile: bool,
    pub drops: Vec<(ItemId, u32, f32)>, // item, count, probability（動的ID）
}
```

**拡張ポイント**
- 新モジュール: `mobs/`
- 衝突システム拡張（player/ に戦闘追加）
- スポーン条件（バイオーム、時間帯）

---

### 9. マップ機能

**データ構造**
```rust
// components/map.rs
pub struct MapMarker {
    pub position: IVec3,
    pub marker_type: MarkerType,
    pub label: Option<String>,
}

pub enum MarkerType {
    Machine(ItemId),
    Player,
    Custom,
}

// リソース
pub struct MapData {
    pub explored_chunks: HashSet<IVec2>,
    pub markers: Vec<MapMarker>,
}
```

**拡張ポイント**
- 新モジュール: `ui/map.rs`
- チャンクの「探索済み」フラグ
- ミニマップ + フルスクリーンマップ

---

### 10. ブループリント

**データ構造**
```rust
// save/blueprint.rs
pub struct Blueprint {
    pub name: String,
    pub size: IVec3,
    pub blocks: Vec<BlueprintBlock>,
}

pub struct BlueprintBlock {
    pub offset: IVec3,
    pub block_type: ItemId,
    pub rotation: u8,
    pub machine_config: Option<MachineConfig>,
}
```

**拡張ポイント**
- 保存: `save/blueprint.rs`
- UI: 選択範囲 → 保存、プレビュー表示
- 配置: ゴースト表示 → 一括配置

---

### 11. 統計・分析

**データ構造**
```rust
// components/statistics.rs
pub struct ProductionStats {
    pub item_produced: HashMap<ItemId, TimeSeries>,  // 動的ID対応
    pub item_consumed: HashMap<ItemId, TimeSeries>,  // 動的ID対応
    pub power_usage: TimeSeries,
}

pub struct TimeSeries {
    pub samples: VecDeque<(f64, f32)>,  // (timestamp, value)
    pub resolution: f32,  // サンプリング間隔（秒）
}

pub struct BottleneckAnalysis {
    pub slow_machines: Vec<(Entity, f32)>,  // 稼働率が低い機械
    pub full_outputs: Vec<Entity>,  // 出力詰まり
    pub empty_inputs: Vec<Entity>,  // 入力待ち
}
```

**拡張ポイント**
- 新モジュール: `ui/statistics.rs`
- グラフ描画（bevy_egui等）
- ボトルネック自動検出

---

### 12. サウンド

**データ構造**
```rust
// components/audio.rs
pub struct SoundEmitter {
    pub sound_id: &'static str,
    pub volume: f32,
    pub loop_: bool,
}

// game_spec/sounds.rs
pub struct SoundSpec {
    pub id: &'static str,
    pub path: &'static str,
    pub category: SoundCategory,  // BGM, SFX, Ambient
    pub default_volume: f32,
}
```

**拡張ポイント**
- `bevy_audio` または `bevy_kira_audio`
- カテゴリ別音量設定
- 3D空間サウンド（機械の動作音）

---

### 13. 実績システム

**データ構造**
```rust
// components/achievements.rs
pub struct Achievement {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub condition: AchievementCondition,
    pub unlocked: bool,
}

pub enum AchievementCondition {
    ProduceItem { item: ItemId, count: u32 },
    PlaceMachines { count: u32 },
    CompleteQuest { quest_id: &'static str },
    PlayTime { minutes: u32 },
    // ...
}
```

**拡張ポイント**
- Steamworks SDK連携
- ローカル実績（オフライン用）
- 通知UI

---

### 14. プレイヤースキン

**データ構造**
```rust
// components/player_skin.rs
pub struct PlayerSkin {
    pub texture: Handle<Image>,
    pub model: PlayerModel,  // Steve/Alex 体型
}

pub enum PlayerModel {
    Classic,  // 4px腕
    Slim,     // 3px腕
}

pub struct SkinLoader {
    pub cache: HashMap<String, Handle<Image>>,
}
```

**拡張ポイント**
- マイクラ形式のスキンPNG読み込み（64x64）
- スキン選択UI
- オンラインスキンサーバー対応（将来）

---

### 15. モデルインポート（Blockbench対応）

**参考ツール**:
- [Blockbench Import Library](https://github.com/tomalbrc/blockbench-import-library) - Fabric用ライブラリ
- [GeckoLib](https://github.com/bernie-g/geckolib) - アニメーションエンジン
- [Chameleon](https://github.com/mchorse/chameleon) - Blockbenchモデル読み込み

**データ構造**
```rust
// assets/model_loader.rs
pub struct BlockbenchModel {
    pub meshes: Vec<Mesh>,
    pub textures: Vec<Handle<Image>>,
    pub animations: HashMap<String, Animation>,
    pub bones: Vec<Bone>,
}

pub struct Animation {
    pub name: String,
    pub duration: f32,
    pub keyframes: Vec<Keyframe>,
    pub loop_mode: LoopMode,
}

// .bbmodel (JSON) パーサー
pub fn load_bbmodel(path: &str) -> Result<BlockbenchModel, Error>;
```

**対応フォーマット**
- `.bbmodel` (Blockbench Generic)
- `.ajmodel` (Animated Java)
- glTF/GLB（既存のBevy対応）

**拡張ポイント**
- 新モジュール: `assets/blockbench.rs`
- アニメーション再生システム
- Modからのモデル読み込み
- ホットリロード対応

---

### 16. Modding対応（フルアクセス）

**目標: Factorioレベルの深いMod対応**

**設計方針（2026-01-08 確定）**:
- **ハイブリッド方式**: WASM（ロジック）+ WebSocket（スクリプト）+ TOML（データ）
- base は本体に内蔵（Engine + base 一体）
- Core Mod で新ロジック定義 → Data Mod で別ユーザーが拡張可能

**Modレイヤー構成**
| レイヤー | 実行方式 | 言語 | 用途 |
|---------|---------|------|------|
| **Core Mod** | WASM（同一プロセス） | Rust, C++ | ロジック追加・置換 |
| **Script Mod** | WebSocket（別プロセス） | Python, Lua, JS | イベントフック |
| **Data Mod** | TOML読み込み | - | アイテム/機械/レシピ |

**Modで変更可能にする範囲**
| レイヤー | 内容 | 実現方法 |
|----------|------|----------|
| コンテンツ | ブロック、アイテム、機械、レシピ | Data Mod（TOML） |
| ロジック | 機械動作、物流ルール | Core Mod（WASM） |
| イベント | フック、条件分岐 | Script Mod（WebSocket）/ Core Mod |
| UI | カスタムUI、HUD | Core Mod |
| レンダリング | シェーダー、エフェクト | カスタムシェーダー |
| モデル | 機械・Mobの3Dモデル | Blockbench直接インポート |
| サウンド | BGM、効果音 | OGG/WAVファイル |
| スキン | プレイヤー見た目 | PNG画像 |

**アーキテクチャ**

```rust
// 動的ID（Mod追加アイテム用）
pub struct ItemId(u32);

pub struct ModRegistry {
    pub items: HashMap<ItemId, ItemDescriptor>,
    pub machines: HashMap<MachineId, MachineDescriptor>,
    pub recipes: HashMap<RecipeId, RecipeDescriptor>,
    pub models: HashMap<String, BlockbenchModel>,
    pub sounds: HashMap<String, SoundSpec>,
}

// 外部Mod API
pub struct ModApiServer {
    // WebSocket or JSON-RPC
    pub connections: Vec<ModConnection>,
}
```

**Mod API（外部プロセスから呼び出し）**
```json
// WebSocket経由でイベント購読
{ "action": "subscribe", "events": ["block_placed", "machine_processed"] }

// イベント通知（ゲーム→Mod）
{ "event": "block_placed", "pos": [10, 5, 20], "block": "iron_ore" }

// アクション実行（Mod→ゲーム）
{ "action": "register_item", "id": "mymod:super_ingot", "descriptor": {...} }
{ "action": "spawn_entity", "type": "machine", "pos": [10, 5, 20] }
```

**Data Mod（TOML形式）**
```toml
# mods/my_mod/items.toml
[[items]]
id = "mymod:super_ingot"
name = "超合金インゴット"
color = [0.8, 0.8, 0.2]
stack_size = 64

[[recipes]]
machine = "furnace"
inputs = [{ id = "mymod:rare_ore", count = 2 }]
outputs = [{ id = "mymod:super_ingot", count = 1 }]
```

**必要な基盤**
1. イベントシステム: Bevy Observer + 外部通知
2. 動的レジストリ: enum廃止、ID化
3. Data Mod ローダー: TOML読み込み
4. WebSocket API: Script Mod 用（port 9877）
5. WASM ローダー: Wasmtime 統合
6. アセットローダー: Mod用アセットパス対応
7. Blockbenchローダー: .bbmodel パーサー

---

### 17. マルチプレイ基盤

**必要な変更（大規模）**
```rust
// components/network.rs
pub struct NetworkId(u64);
pub struct Authority {
    pub owner: PlayerId,
}

// 同期対象
- プレイヤー位置
- ブロック変更
- 機械状態
- インベントリ
- チャンク読み込み
```

**拡張ポイント**
- 新モジュール: `networking/`
- 全システムに同期ポイント追加
- サーバー権威モデル

---

## 機能依存関係

```
Mod基盤 ─────────────────────────────────────────────
  ├─ イベントシステム
  ├─ 動的ID (ItemId/MachineId)
  ├─ Data Mod (TOML)
  └─ Script Mod (WebSocket)
         │
         ↓
基盤システム（順に実装）
  電力 → 液体・気体 → 信号制御 → 線路 → Mob
         │
         ↓
独立機能（並行可）
  マップ / ブループリント / 統計 / サウンド / 実績
  スキン / ストレージ / クラフト / ロボット
         │
         ↓
マルチプレイ（最後）
```

---

## 共通拡張ポイント

### 1. IoPortの拡張
```rust
pub struct IoPort {
    pub side: PortSide,
    pub port_type: PortType,  // 追加: Item, Fluid, Power, Signal
    pub slot_id: u8,
}

pub enum PortType {
    Item,
    Fluid,
    Power,
    Signal,
}
```

### 2. ネットワーク（グラフ）基盤
```rust
// 電力グリッド、信号網、パイプ網で共通
pub trait NetworkNode {
    fn position(&self) -> IVec3;
    fn connections(&self) -> Vec<IVec3>;
}

pub struct Network<T: NetworkNode> {
    pub nodes: HashMap<IVec3, Entity>,
    pub graph: Graph<Entity>,  // petgraphなど
}
```

### 3. Descriptor統一
```rust
// 全てのゲームオブジェクトを同じパターンで定義
pub trait GameDescriptor {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
}
```

### 4. 天候・昼夜の拡張ポイント（将来用）
```rust
// 今は実装しないが、拡張可能な設計にしておく
pub struct TimeOfDay {
    pub hour: f32,  // 0.0-24.0
}

pub struct Weather {
    pub weather_type: WeatherType,
    pub intensity: f32,
}

pub enum WeatherType {
    Clear,
    Rain,
    Storm,
    // Modで追加可能
}
```

---

## AI向けガイド

### アイテム追加

`mods/base/items.toml` + 3Dモデル → 自動登録

### 機械追加

`mods/base/machines.toml` + `recipes.toml` → 自動UI生成

### 保護機構

| 機構 | 内容 |
|------|------|
| MachineBundle | 必須コンポーネントをBundleで強制 |
| WorldCommands | 直接操作禁止、安全なAPIのみ |
| newtype | ItemId/MachineId で型混同防止 |
| フォールバック | 不明IDは警告＋デフォルト値 |

---

*最終更新: 2026-01-11*
