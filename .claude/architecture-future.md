# 骨格設計：将来機能の拡張ポイント

## 機能リスト（全17機能）

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
| 7 | ロボット（Lua） | プログラム可能な自動化（飛行含む） | 低〜中 |
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

## 現在のアーキテクチャ（参照用）

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
    HasItem(BlockType),
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
    pub slots: [Option<(BlockType, u32)>; 9],  // 3x3
    pub result: Option<(BlockType, u32)>,
}

// game_spec/recipes.rs に追加
pub struct CraftingRecipe {
    pub pattern: &'static [&'static str],  // "III", " S ", " S " など
    pub ingredients: &'static [(char, BlockType)],
    pub result: (BlockType, u32),
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
    pub filter: Option<Vec<BlockType>>,  // 許可アイテムリスト
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

---

### 7. ロボット（Lua）

**データ構造**
```rust
// components/robots.rs
pub struct Robot {
    pub position: Vec3,  // 浮動小数点（飛行対応）
    pub inventory: Vec<MachineSlot>,
    pub script: Handle<LuaScript>,
    pub state: RobotState,
    pub can_fly: bool,
    pub fuel: f32,
}

pub enum RobotState {
    Idle,
    Moving { target: Vec3 },
    Mining { target: IVec3, progress: f32 },
    Crafting,
    Flying { target: Vec3, altitude: f32 },
}
```

**Lua API**
```lua
robot.move(direction)      -- 移動
robot.fly(x, y, z)         -- 飛行（飛行可能な場合）
robot.mine()               -- 前方を採掘
robot.place(item)          -- 設置
robot.transfer(from, to)   -- アイテム移動
robot.detect()             -- 前方ブロック検知
robot.signal()             -- 信号送信
robot.craft(recipe)        -- クラフト実行
```

**拡張ポイント**
- 新モジュール: `scripting/` （mlua統合）
- `game_spec/robot_api.rs` でAPI定義

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
    pub id: &'static str,
    pub max_health: f32,
    pub speed: f32,
    pub hostile: bool,
    pub drops: &'static [(BlockType, u32, f32)], // item, count, probability
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
    Machine(BlockType),
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
    pub block_type: BlockType,
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
    pub item_produced: HashMap<BlockType, TimeSeries>,
    pub item_consumed: HashMap<BlockType, TimeSeries>,
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
    ProduceItem { item: BlockType, count: u32 },
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

**Modで変更可能にする範囲**
| レイヤー | 内容 | 実現方法 |
|----------|------|----------|
| コンテンツ | ブロック、アイテム、機械、レシピ | データファイル（JSON/Lua） |
| ロジック | 機械動作、物流ルール、レシピ計算 | Luaスクリプト |
| UI | カスタムUI、HUD、メニュー | Lua + テンプレート |
| システム | システム実行順序、イベントフック | Luaイベント |
| レンダリング | シェーダー、エフェクト | カスタムシェーダー |
| モデル | 機械・Mobの3Dモデル | Blockbench直接インポート |
| サウンド | BGM、効果音 | OGG/WAVファイル |
| スキン | プレイヤー見た目 | PNG画像 |

**アーキテクチャ変更**

```rust
// 現在: BlockType は enum（コンパイル時固定）
pub enum BlockType { Stone, IronOre, ... }

// 必要: 動的ID + Luaバインディング
pub struct BlockId(u32);

pub struct ModRegistry {
    pub blocks: HashMap<BlockId, BlockDescriptor>,
    pub items: HashMap<ItemId, ItemDescriptor>,
    pub machines: HashMap<MachineId, MachineDescriptor>,
    pub recipes: HashMap<RecipeId, RecipeDescriptor>,
    pub scripts: HashMap<String, LuaScript>,
    pub models: HashMap<String, BlockbenchModel>,
    pub sounds: HashMap<String, SoundSpec>,
}
```

**Luaイベントシステム**
```lua
-- mod.lua
mod.on_init(function()
    -- Mod初期化
end)

mod.on_tick(function(delta)
    -- 毎フレーム処理
end)

mod.on_machine_tick(function(machine)
    -- 機械処理をオーバーライド
    if machine.type == "my_custom_machine" then
        -- カスタムロジック
    end
end)

mod.on_item_transfer(function(from, to, item)
    -- 物流ルールをカスタマイズ
end)

mod.register_ui("my_panel", function()
    -- カスタムUIを描画
end)

-- Blockbenchモデル登録
mod.register_model("my_machine", "models/my_machine.bbmodel")
```

**必要な基盤**
1. **Lua統合**: mlua クレート
2. **イベントシステム**: フックポイントを全システムに追加
3. **動的レジストリ**: enum廃止、ID化
4. **アセットローダー**: Mod用アセットパス対応
5. **サンドボックス**: ファイルアクセス制限
6. **Blockbenchローダー**: .bbmodel パーサー

**実装順序**
1. Phase C: 内部でDescriptor化（現在進行中）
2. Luaランタイム統合（ロボット機能と共通）
3. イベントシステム設計
4. 動的レジストリへの移行
5. Blockbenchローダー実装
6. Mod API設計・公開

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

## 依存関係グラフ（Lua後付け設計）

```
                    ┌─────────────────────────────────┐
                    │      イベントシステム           │
                    │  （フックポイント + ゲームAPI） │
                    └─────────────────────────────────┘
                                   │
            ┌──────────────────────┼──────────────────────┐
            │                      │                      │
            ↓                      ↓                      ↓
    ┌───────────────┐    ┌─────────────────┐    ┌─────────────┐
    │データ駆動Mod  │    │  動的レジストリ  │    │  ゲームAPI  │
    │(JSON/TOML)    │    │  (enum廃止)     │    │  (状態参照) │
    └───────────────┘    └─────────────────┘    └─────────────┘
            │                      │                      │
            └──────────────────────┼──────────────────────┘
                                   │
                    ┌──────────────┴──────────────┐
                    │    データ駆動Modding完成    │
                    │  （コンテンツ追加が可能）   │
                    └─────────────────────────────┘
                                   │
    ┌──────────────────────────────┼──────────────────────────────┐
    │              │               │               │              │
    ↓              ↓               ↓               ↓              ↓
┌──────┐    ┌──────────┐    ┌──────────┐    ┌─────────┐    ┌──────┐
│ 電力 │    │液体・気体│    │ 信号制御 │    │  線路   │    │ Mob  │
└──────┘    └──────────┘    └──────────┘    └─────────┘    └──────┘


            独立機能（いつでも実装可能）
            ════════════════════════════

    ┌─────────┐  ┌─────────────┐  ┌──────────┐  ┌─────────┐
    │ マップ  │  │ブループリント│  │ 統計分析 │  │ サウンド│
    └─────────┘  └─────────────┘  └──────────┘  └─────────┘

    ┌─────────┐  ┌─────────────┐  ┌──────────┐  ┌───────────┐
    │ 実績    │  │プレイヤー   │  │ストレージ│  │Blockbench │
    │         │  │スキン       │  │          │  │インポート │
    └─────────┘  └─────────────┘  └──────────┘  └───────────┘

    ┌──────────┐
    │クラフト  │
    │システム  │
    └──────────┘


            スクリプト機能（後付け）
            ══════════════════════════

                    ┌─────────────────┐
                    │    Lua統合      │ ← イベントシステム + ゲームAPIに接続
                    └─────────────────┘
                             │
                             ↓
                    ┌─────────────────┐
                    │    ロボット     │
                    └─────────────────┘


            最後に実装
            ══════════

                    ┌─────────────────┐
                    │ マルチプレイ基盤│
                    └─────────────────┘
```

**設計方針**:
- イベントシステムとゲームAPIを先に整備
- Luaは後付けで、既存のフックポイントに接続するだけ
- データ駆動Modding（JSON/TOML）で大半のMod対応が可能
- ロジック変更が必要な場合のみLuaを使用

---

## 推奨実装順序（フルアクセスModding優先）

| Phase | 機能 | 理由 |
|-------|------|------|
| **基盤フェーズ** | | |
| D.1 | Phase C完了 | Descriptor化が全ての前提 |
| D.2 | イベントシステム | フックポイント（Lua後付け可能に） |
| D.3 | ゲームAPI整理 | 状態アクセスの統一インターフェース |
| D.4 | データ駆動Modding | JSON/TOMLでコンテンツ追加 |
| D.5 | 動的レジストリ | BlockType enum → 動的ID |
| D.6 | Blockbenchローダー | モデル追加の基盤 |
| **独立機能（並行可）** | | |
| D.7 | マップ機能 | 独立、QoL向上 |
| D.8 | ブループリント | 独立、大規模建築効率化 |
| D.9 | クラフトシステム | 手動クラフト |
| D.10 | ストレージ | 倉庫ブロック |
| D.11 | 統計・分析 | 生産可視化 |
| D.12 | サウンド | BGM、効果音 |
| D.13 | 実績システム | Steam連携 |
| D.14 | プレイヤースキン | 見た目変更 |
| **Mod対応しながら機能追加** | | |
| D.15 | 電力システム + Mod API | 最初のMod対応機能として |
| D.16 | 液体・気体 + Mod API | 電力と同パターン |
| D.17 | 信号制御 + Mod API | 電力・流体の条件分岐 |
| D.18 | 線路機能 + Mod API | 信号制御で閉塞管理 |
| D.19 | Mob + Mod API | 敵/NPCのMod追加可能に |
| **スクリプト機能（後付け）** | | |
| D.20 | Lua統合 | イベントシステム + ゲームAPIに接続 |
| D.21 | ロボット | Lua APIで動作 |
| **最後** | | |
| D.22 | マルチプレイ | 全機能の同期設計 |

**ポイント**:
- 各機能追加時にMod APIも同時設計
- 「後からMod対応」ではなく「Mod対応しながら機能追加」
- これでModが深くいじれる設計になる

---

## 共通拡張ポイント（先に整備すべき）

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

## 次のアクション

1. **Phase C完了**（現在進行中）
   - BlockDescriptor, ItemDescriptor の実装
   - レジストリシステム完成

2. **Phase D: 基盤強化**
   - UIState統合（壊れそうな箇所の修正）
   - IoPort拡張（PortType追加）
   - Network基盤の設計

3. **Lua統合 + Blockbenchローダー**
   - mlua クレート導入
   - .bbmodel パーサー実装

4. **独立機能から実装**
   - マップ機能
   - ブループリント
   - クラフトシステム

5. **基盤システム実装**
   - 電力 → 液体 → 信号 の順
