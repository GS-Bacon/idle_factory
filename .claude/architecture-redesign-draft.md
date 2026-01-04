# アーキテクチャ再設計案 (Draft)

## 目的

1. **コードダイエット**: 重複削減、責務明確化
2. **将来拡張性**: 電力・流体・Modding・マルチプレイ対応

---

## 現状分析

### コード分布 (総計: 20,803行)

| モジュール | 行数 | 責務 |
|------------|------|------|
| systems/ | 8,785行 (42%) | ゲームロジック全般（肥大化） |
| setup/ | 1,421行 | 初期化・UI生成 |
| components/ | 1,409行 | ECSコンポーネント |
| game_spec/ | 1,040行 | 静的データ定義 |
| world/ | 1,024行 | ワールド生成 |
| updater/ | 715行 | 自動更新 |
| player/ | 604行 | プレイヤー・インベントリ |
| ui/ | 604行 | UI関連 |
| save.rs | 875行 | セーブデータ構造 |
| main.rs | 833行 | エントリポイント |
| meshes.rs | 592行 | メッシュ生成 |

### 問題点

1. **UI分散**: 3箇所に分散（2,642行）
2. **機械分散**: 7ファイルに分散（2,797行）
3. **セーブ分散**: 2ファイル（1,458行）
4. **責務不明確**: systems/が全体の42%

---

## 提案: ドメイン駆動 + トレイト統一

### 新ディレクトリ構造

```
src/
├── main.rs              # エントリ（100行以下に削減）
├── lib.rs
│
├── core/                # ★純粋ロジック（Bevy非依存、テスト容易）
│   ├── mod.rs
│   ├── inventory.rs     # インベントリ計算
│   ├── recipe.rs        # レシピ評価
│   ├── network.rs       # 電力・流体・信号の共通基盤
│   └── save_format.rs   # セーブデータ構造（バージョン付き）
│
├── machines/            # ★機械を統合（2,797行→1モジュール）
│   ├── mod.rs
│   ├── traits.rs        # Machine, PowerConsumer, FluidHandler
│   ├── registry.rs      # 機械レジストリ（Mod対応準備）
│   ├── conveyor.rs      # コンベアシステム+コンポーネント
│   ├── miner.rs         # 採掘機
│   ├── furnace.rs       # 精錬炉
│   ├── crusher.rs       # 粉砕機
│   └── components.rs    # 共通コンポーネント
│
├── ui/                  # ★UI統合（2,642行→1モジュール）
│   ├── mod.rs
│   ├── widgets.rs       # 共通ウィジェット（スロット、ボタン等）
│   ├── inventory.rs     # インベントリUI
│   ├── machine.rs       # 機械UI（MachineDescriptorから自動生成）
│   ├── storage.rs       # 倉庫UI
│   ├── quest.rs         # クエストUI
│   └── hotbar.rs        # ホットバー
│
├── world/               # ワールド生成（現状維持）
│   ├── mod.rs
│   ├── chunk.rs
│   └── biome.rs
│
├── player/              # プレイヤー（現状維持）
│   ├── mod.rs
│   ├── inventory.rs
│   └── global_inventory.rs
│
├── save/                # ★セーブ統合（1,458行→1モジュール）
│   ├── mod.rs
│   ├── format.rs        # データ構造
│   ├── systems.rs       # Bevyシステム
│   └── migration.rs     # バージョン移行（将来用）
│
├── commands/            # コマンドシステム（command/から移動）
│   ├── mod.rs
│   ├── executor.rs
│   └── handlers.rs
│
├── block/               # ブロック関連統合
│   ├── mod.rs
│   ├── types.rs         # BlockType（block_type.rsから）
│   ├── breaking.rs      # 破壊処理
│   └── placement.rs     # 設置処理
│
├── events/              # イベント（現状維持）
├── plugins/             # プラグイン（現状維持）
├── debug/               # デバッグ（cfg(debug)で分離）
└── updater/             # 自動更新（feature gateで分離）
```

---

## Machineトレイト設計

### トレイト定義

```rust
/// すべての機械が実装する基本トレイト
pub trait Machine: Send + Sync {
    /// 毎フレームの処理
    fn tick(&mut self, ctx: &mut MachineContext) -> MachineResult;

    /// 入出力ポート定義
    fn ports(&self) -> &[Port];

    /// UI/セーブ用のメタデータ
    fn descriptor(&self) -> &MachineDescriptor;

    /// アイテム受け入れ判定
    fn can_accept(&self, port: PortId, item: &ItemStack) -> bool;

    /// アイテム投入
    fn insert(&mut self, port: PortId, item: ItemStack) -> Result<(), ItemStack>;

    /// アイテム取り出し
    fn extract(&mut self, port: PortId) -> Option<ItemStack>;
}

/// 電力消費機械（将来用）
pub trait PowerConsumer {
    fn power_required(&self) -> f32;
    fn on_power_change(&mut self, available: f32);
}

/// 流体処理機械（将来用）
pub trait FluidHandler {
    fn fluid_ports(&self) -> &[FluidPort];
    fn fluid_tick(&mut self, ctx: &mut FluidContext);
}
```

### MachineDescriptor

```rust
/// 機械のメタデータ（UI自動生成、セーブ、Mod対応に使用）
pub struct MachineDescriptor {
    pub id: &'static str,        // "miner", "furnace" など
    pub display_name: &'static str,
    pub category: MachineCategory,
    pub size: IVec3,             // 占有ブロック数
    pub ports: Vec<PortDescriptor>,
    pub recipes: Option<&'static [RecipeId]>,
    pub power_consumption: Option<f32>,  // 将来用
}
```

---

## ネットワーク基盤（電力・流体・信号共通）

```rust
/// ネットワークノードの共通インターフェース
pub trait NetworkNode {
    type Value: Copy + Default;  // f32 for power, FluidStack for fluid

    fn can_provide(&self) -> Self::Value;
    fn can_accept(&self) -> Self::Value;
    fn apply_flow(&mut self, flow: Self::Value);
}

/// 汎用ネットワーク
pub struct Network<N: NetworkNode> {
    nodes: HashMap<Entity, N>,
    edges: Vec<(Entity, Entity)>,
    cached_flow: Option<Vec<(Entity, Entity, N::Value)>>,
}

impl<N: NetworkNode> Network<N> {
    /// ネットワーク全体の流れを計算
    pub fn solve(&mut self) { ... }

    /// 各ノードに流量を適用
    pub fn apply(&mut self) { ... }
}
```

---

## UI共通化

### 共通ウィジェット

```rust
// 現在: 各UIで似たコードを繰り返し
// commands.spawn((Node { ... }, BackgroundColor(...), ...))

// 提案: ウィジェットビルダー
pub fn spawn_slot(commands: &mut Commands, config: SlotConfig) -> Entity {
    commands.spawn((
        Slot,
        Node {
            width: Val::Px(config.size),
            height: Val::Px(config.size),
            ..default()
        },
        BackgroundColor(ui_colors::SLOT_BG),
        BorderColor(ui_colors::SLOT_BORDER),
        BorderRadius::all(Val::Px(6.0)),
        config.marker,
    )).id()
}

pub fn spawn_button(commands: &mut Commands, text: &str, on_click: impl Component) -> Entity {
    // 共通ボタン実装
}
```

### 機械UI自動生成

```rust
/// MachineDescriptorからUIを自動生成
pub fn spawn_machine_ui(
    commands: &mut Commands,
    descriptor: &MachineDescriptor,
    machine_entity: Entity,
) {
    // 入力ポート数に応じてスロット生成
    // レシピがあれば進捗バー追加
    // 出力ポートに応じてスロット生成
}
```

---

## 移行計画

### Phase A: 準備（破壊的変更なし）

| タスク | 詳細 | 影響範囲 |
|--------|------|----------|
| core/モジュール作成 | 純粋ロジックを抽出 | 新規追加のみ |
| Machineトレイト定義 | traits.rsを作成 | 新規追加のみ |
| ウィジェット関数作成 | ui/widgets.rs | 新規追加のみ |

### Phase B: 統合（リファクタリング）

| タスク | 詳細 | 削減行数 |
|--------|------|----------|
| 機械モジュール統合 | systems/conveyor等 → machines/ | -200行 |
| UI統合 | 3箇所 → ui/ | -400行 |
| セーブ統合 | 2ファイル → save/ | -100行 |

### Phase C: 最適化

| タスク | 詳細 | 効果 |
|--------|------|------|
| main.rs分割 | Plugin化 | 可読性向上 |
| debug/ feature gate | cfg(debug) | リリースビルド軽量化 |
| updater/ feature gate | feature = "updater" | 必要時のみ含む |

---

## 期待効果

### 短期

| 項目 | Before | After |
|------|--------|-------|
| 総行数 | 20,803行 | ~19,500行 (-6%) |
| 最大モジュール | systems/ 8,785行 | machines/ ~2,500行 |
| UI重複 | 3箇所 | 1箇所 |

### 長期（将来機能追加時）

| 機能 | 現在の実装コスト | 再設計後 |
|------|------------------|----------|
| 新機械追加 | 5ファイル改修 | 1ファイル追加 |
| 電力システム | 全機械を改修 | PowerConsumer実装のみ |
| Modding | 大規模改修 | Registry登録のみ |
| 機械UI | 個別実装 | 自動生成 |

---

## リスク

| リスク | 対策 |
|--------|------|
| 大規模リファクタのバグ | Phase分割、各Phaseでテスト |
| トレイト設計ミス | 最初は既存機械のみ、新機械で検証 |
| 移行中の機能開発 | Phase Aは新規追加のみで並行可能 |

---

## 質問事項（Geminiレビュー用）

1. Machineトレイトの設計は適切か？過不足は？
2. ネットワーク基盤の汎用性は十分か？
3. 移行の優先順位は妥当か？
4. 見落としているリスクはないか？
5. Bevy 0.15のベストプラクティスに沿っているか？
