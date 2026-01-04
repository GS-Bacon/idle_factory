# アーキテクチャ再設計計画 v1.0

**作成日**: 2026-01-04
**レビュー**: Claude + Gemini合見積

---

## 目的

1. **コードダイエット**: 重複削減、責務明確化（-1,300行目標）
2. **将来拡張性**: 電力・流体・Modding・マルチプレイ対応
3. **ECS最適化**: Bevyのパフォーマンスを最大化

---

## 設計方針（Geminiレビュー反映）

### 採用する方針

| 方針 | 理由 |
|------|------|
| **ECSコンポジション** | `Machine`トレイトの「神メソッド」化を回避 |
| **データ駆動UI** | `MachineDescriptor`からUI自動生成 |
| **Component Hooks** | ネットワークノードの登録/削除を自動化 |
| **Required Components** | spawn時の安全性向上 |
| **コンベア特別扱い** | 物流インフラとして機械と分離 |

### 却下した方針

| 方針 | 却下理由 |
|------|----------|
| `tick`メソッド必須 | ECSの並列性を損なう |
| `Box<dyn Machine>` | Dynamic Dispatchのオーバーヘッド |

---

## 新アーキテクチャ

### ディレクトリ構造

```
src/
├── main.rs              # エントリ（100行以下）
├── lib.rs
│
├── core/                # 純粋ロジック（Bevy非依存）
│   ├── mod.rs
│   ├── inventory.rs     # インベントリ計算
│   ├── recipe.rs        # レシピ評価
│   └── network.rs       # グラフ構造（電力・流体共通）
│
├── machines/            # 機械統合
│   ├── mod.rs
│   ├── components.rs    # 機能コンポーネント（下記参照）
│   ├── descriptors.rs   # MachineDescriptor定義
│   ├── systems.rs       # 機械共通システム
│   ├── miner.rs         # 採掘機固有ロジック
│   ├── furnace.rs       # 精錬炉固有ロジック
│   └── crusher.rs       # 粉砕機固有ロジック
│
├── logistics/           # ★物流インフラ（機械と分離）
│   ├── mod.rs
│   ├── conveyor.rs      # コンベア
│   ├── inserter.rs      # 将来用
│   └── pipe.rs          # 将来用
│
├── networks/            # ネットワークシステム
│   ├── mod.rs
│   ├── graph.rs         # 汎用グラフ構造
│   ├── power.rs         # 電力ネットワーク（将来）
│   └── fluid.rs         # 流体ネットワーク（将来）
│
├── ui/                  # UI統合
│   ├── mod.rs
│   ├── widgets.rs       # 共通ウィジェット
│   ├── machine.rs       # 機械UI（自動生成）
│   ├── inventory.rs     # インベントリUI
│   ├── storage.rs       # 倉庫UI
│   ├── quest.rs         # クエストUI
│   └── hotbar.rs        # ホットバー
│
├── save/                # セーブ統合
│   ├── mod.rs
│   ├── format.rs        # データ構造
│   └── systems.rs       # 保存/読込システム
│
├── block/               # ブロック関連統合
│   ├── mod.rs
│   ├── types.rs         # BlockType
│   ├── breaking.rs      # 破壊処理
│   └── placement.rs     # 設置処理
│
├── world/               # ワールド生成（現状維持）
├── player/              # プレイヤー（現状維持）
├── commands/            # コマンドシステム
├── events/              # イベント
├── plugins/             # プラグイン
├── debug/               # デバッグ（cfg分離）
└── updater/             # 自動更新（feature gate）
```

---

## ECSコンポジション設計

### 機能コンポーネント（machines/components.rs）

```rust
/// 入力ポートを持つ機械
#[derive(Component)]
#[require(MachineInventory)]  // Bevy 0.15 Required Components
pub struct ItemAcceptor {
    pub ports: Vec<InputPort>,
    pub filter: Option<ItemFilter>,
}

/// 出力ポートを持つ機械
#[derive(Component)]
pub struct ItemEjector {
    pub ports: Vec<OutputPort>,
}

/// レシピ処理を行う機械
#[derive(Component)]
pub struct Crafter {
    pub recipe_id: Option<RecipeId>,
    pub progress: f32,
    pub speed_multiplier: f32,
}

/// 電力を消費する機械（将来用、空実装で準備）
#[derive(Component)]
pub struct PowerConsumer {
    pub required_power: f32,
    pub current_power: f32,
}

/// 機械の共通インベントリ
#[derive(Component, Default)]
pub struct MachineInventory {
    pub input_slots: Vec<ItemStack>,
    pub output_slots: Vec<ItemStack>,
    pub fuel_slot: Option<ItemStack>,
}
```

### 機械定義例

```rust
// 精錬炉の生成（Required Componentsで安全に）
fn spawn_furnace(commands: &mut Commands, position: IVec3) -> Entity {
    commands.spawn((
        Furnace,
        ItemAcceptor { ports: vec![InputPort::Back], filter: None },
        ItemEjector { ports: vec![OutputPort::Front] },
        Crafter { recipe_id: None, progress: 0.0, speed_multiplier: 1.0 },
        MachineInventory::default(),
        MachineDescriptor::FURNACE,
        // PowerConsumer は将来追加
    )).id()
}
```

### システム設計

```rust
// 汎用的な入力受付システム（全機械共通）
fn accept_items_system(
    mut query: Query<(&ItemAcceptor, &mut MachineInventory)>,
    mut transfer_events: EventReader<ItemTransferEvent>,
) {
    for event in transfer_events.read() {
        // フィルター確認 → インベントリ追加
    }
}

// 汎用的なクラフトシステム（全機械共通）
fn crafting_system(
    time: Res<Time>,
    mut query: Query<(&mut Crafter, &mut MachineInventory)>,
    recipes: Res<RecipeRegistry>,
) {
    for (mut crafter, mut inv) in query.iter_mut() {
        // レシピ進行 → 完成時にアイテム生成
    }
}

// 機械固有ロジック（必要な場合のみ）
fn miner_system(
    mut query: Query<(&Miner, &mut MachineInventory, &GlobalTransform)>,
    world_data: Res<WorldData>,
) {
    // 採掘固有の処理
}
```

---

## MachineDescriptor（UI自動生成用）

```rust
pub struct MachineDescriptor {
    pub id: &'static str,
    pub display_name: &'static str,
    pub category: MachineCategory,
    pub input_slots: u8,
    pub output_slots: u8,
    pub has_fuel_slot: bool,
    pub has_recipe_select: bool,
    pub power_consumption: Option<f32>,
}

impl MachineDescriptor {
    pub const MINER: Self = Self {
        id: "miner",
        display_name: "採掘機",
        category: MachineCategory::Production,
        input_slots: 0,
        output_slots: 1,
        has_fuel_slot: false,
        has_recipe_select: false,
        power_consumption: None,
    };

    pub const FURNACE: Self = Self {
        id: "furnace",
        display_name: "精錬炉",
        category: MachineCategory::Processing,
        input_slots: 1,
        output_slots: 1,
        has_fuel_slot: true,
        has_recipe_select: false,
        power_consumption: None,
    };
}
```

---

## 物流インフラ（logistics/）

コンベアは他の機械と異なり、**連続的移動・ジッパー合流・曲がり角処理**など特殊な要件があるため、`machines/`とは分離。

```rust
// logistics/conveyor.rs
#[derive(Component)]
pub struct Conveyor {
    pub speed: f32,
    pub shape: ConveyorShape,
    pub items: Vec<ConveyorItem>,  // 連続的な位置を持つ
}

// コンベア専用システム（高速化のため最適化）
fn conveyor_tick_system(
    time: Res<Time>,
    mut conveyors: Query<(&mut Conveyor, &GlobalTransform)>,
) {
    // アイテム移動、合流処理
}
```

---

## UI共通化（ui/widgets.rs）

```rust
/// 共通スロットウィジェット
pub fn spawn_slot(
    commands: &mut Commands,
    config: SlotConfig,
) -> Entity {
    commands.spawn((
        Slot { index: config.index },
        Node {
            width: Val::Px(48.0),
            height: Val::Px(48.0),
            ..default()
        },
        BackgroundColor(ui_colors::SLOT_BG),
        BorderColor(ui_colors::SLOT_BORDER),
        BorderRadius::all(Val::Px(6.0)),
    ))
    .observe(on_slot_click)  // Entity Observers
    .id()
}

/// 機械UIを自動生成
pub fn spawn_machine_ui(
    commands: &mut Commands,
    descriptor: &MachineDescriptor,
    machine_entity: Entity,
) {
    // descriptor.input_slots分のスロット生成
    // descriptor.has_fuel_slotならFuelSlot追加
    // descriptor.has_recipe_selectならレシピセレクタ追加
    // descriptor.output_slots分のスロット生成
}
```

---

## ネットワーク基盤（networks/graph.rs）

```rust
/// 汎用ネットワークグラフ（電力・流体共通）
pub struct NetworkGraph<V: Copy + Default> {
    nodes: HashMap<Entity, NetworkNode<V>>,
    edges: Vec<(Entity, Entity)>,
}

pub struct NetworkNode<V> {
    pub capacity: V,
    pub current: V,
}

impl<V: Copy + Default + std::ops::Add<Output = V>> NetworkGraph<V> {
    /// ノード追加（Component Hookから呼ばれる）
    pub fn add_node(&mut self, entity: Entity, capacity: V) { ... }

    /// ノード削除（Component Hookから呼ばれる）
    pub fn remove_node(&mut self, entity: Entity) { ... }

    /// 接続追加
    pub fn connect(&mut self, from: Entity, to: Entity) { ... }

    /// ネットワーク全体を解決（毎フレーム）
    pub fn solve(&mut self) { ... }
}
```

---

## 移行計画

### Phase A: 準備（2-3時間、破壊的変更なし）

| タスク | 詳細 | リスク |
|--------|------|--------|
| core/作成 | 純粋ロジック抽出（inventory, recipe） | 低 |
| 機能コンポーネント定義 | ItemAcceptor, ItemEjector, Crafter | 低 |
| MachineDescriptor定義 | 既存機械のメタデータ | 低 |
| ui/widgets.rs作成 | spawn_slot, spawn_button | 低 |

### Phase B-1: コンベア分離（1-2時間）

| タスク | 詳細 | 削減行数 |
|--------|------|----------|
| logistics/conveyor.rs作成 | systems/conveyor.rsから移動 | 0（移動のみ） |
| コンベア専用最適化 | 必要に応じて | - |

### Phase B-2: 機械統合（3-4時間）

| タスク | 詳細 | 削減行数 |
|--------|------|----------|
| machines/作成 | miner, furnace, crusher統合 | -200行 |
| 共通システム抽出 | accept_items, crafting | -150行 |
| components/machines.rs削除 | machines/components.rsに統合 | -100行 |

### Phase B-3: UI統合（3-4時間）

| タスク | 詳細 | 削減行数 |
|--------|------|----------|
| ui/machine.rs | 機械UI自動生成 | -300行 |
| ui/inventory.rs | inventory_ui.rsから移動 | -100行 |
| 3箇所のUI統合 | setup/ui/, systems/inventory_ui, ui/ | -200行 |

### Phase B-4: セーブ統合（1-2時間）

| タスク | 詳細 | 削減行数 |
|--------|------|----------|
| save/作成 | save.rs + save_systems.rs統合 | -100行 |
| 変換マクロ化 | BlockTypeSave等 | -50行 |

### Phase C: 最適化（1時間）

| タスク | 詳細 | 効果 |
|--------|------|------|
| main.rs分割 | GamePlugin化 | 可読性向上 |
| debug/ cfg分離 | `#[cfg(debug_assertions)]` | リリース軽量化 |
| updater/ feature gate | `feature = "updater"` | 必要時のみ |

---

## 期待効果

### 行数削減

| フェーズ | Before | After | 削減 |
|----------|--------|-------|------|
| Phase A | 20,803行 | 20,900行 | +100行（準備） |
| Phase B-1 | - | - | 0（移動のみ） |
| Phase B-2 | - | - | -450行 |
| Phase B-3 | - | - | -600行 |
| Phase B-4 | - | - | -150行 |
| Phase C | - | - | -100行（cfg分離） |
| **合計** | 20,803行 | ~19,500行 | **-1,300行 (-6%)** |

### 将来拡張コスト

| 機能 | 現在 | 再設計後 |
|------|------|----------|
| 新機械追加 | 5ファイル改修 | 1ファイル + Descriptor追加 |
| 電力システム | 全機械改修 | PowerConsumerコンポーネント追加 |
| Modding | 大規模改修 | Registry登録のみ |
| 機械UI | 個別実装 | 自動生成 |

---

## リスクと対策

| リスク | 対策 |
|--------|------|
| 大規模リファクタのバグ | Phase分割、各フェーズでテスト全通過確認 |
| コンポーネント設計ミス | 最初は既存機械のみ、新機械追加時に検証 |
| パフォーマンス劣化 | ベンチマーク計測、問題あれば最適化 |
| セーブデータ非互換 | 既存プレイヤーなし（CLAUDE.md記載）、移行不要 |
| 移行中の機能開発 | Phase Aは新規追加のみで並行可能 |

---

## 次のアクション

1. **承認後、Phase Aを開始**
   - core/モジュール作成
   - 機能コンポーネント定義
   - MachineDescriptor定義
   - ui/widgets.rs作成

2. **Phase A完了後、Phase B-1（コンベア分離）**
   - 最もリスクが低く、構造が見えやすい

3. **段階的にPhase B-2〜C**
   - 各フェーズでテスト全通過を確認

---

## 合見積サマリー

| 観点 | Claude評価 | Gemini評価 |
|------|------------|------------|
| 全体方針 | ドメイン駆動 + ECS | ECSコンポジション推奨 |
| Machineトレイト | tick必須 | tick不要、入出力に特化 |
| コンベア | 機械として統合 | **物流インフラとして分離** |
| ネットワーク基盤 | 汎用設計 | Component Hooks活用 |
| UI | 共通化 | Entity Observers活用 |
| 移行 | 3Phase | **Phase Bを垂直分割** |

**最終採用**: Geminiの指摘を反映し、ECSのパフォーマンスとBevy 0.15の機能を最大活用する設計に修正。
