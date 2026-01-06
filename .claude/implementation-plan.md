# 統合実装計画 (2026-01-04 更新)

## 現状サマリー

| 項目 | 値 |
|------|-----|
| コード行数 | **22,500行** |
| テスト | **323件** 通過 (lib:108, bin:37, e2e:148, fuzz:11, proptest:8, ssim:3, integration:8) |
| unwrap() | **17箇所** (全てテストコード内) |
| Clippy警告 | **0件** |
| カバレッジ | **8.54%** (全体)、ロジック部分70%+ |

---

## 優先順位（2026-01-04 更新）

| 順位 | カテゴリ | 理由 |
|------|----------|------|
| **1** | v0.2完成 | ゲームとして遊べる状態に |
| **2** | アーキテクチャ再設計 | 将来機能の土台（-1,300行） |
| **3** | 機能拡張 | v0.3以降の新機能 |

---

## Phase A: v0.2完成（短期）✅ 完了

### A.1 UIテーマ刷新 ✅

- テーマ定数: `setup/ui/mod.rs`に定義済み（SLOT_SIZE, SLOT_RADIUS, 色定数等）
- スロットBorderRadius: 全UIに適用済み
- ホバー/選択スタイル: `systems/inventory_ui.rs`で実装済み
- 機械UI統一: `ui/machine_ui.rs`でFactoryテーマ適用済み

### A.2 バイオーム表示UI ✅

- BiomeHudText: `setup/ui/mod.rs`で実装済み
- update_biome_hud: `systems/debug_ui.rs`で実装済み

### A.3 チュートリアル ✅

- TutorialProgress: `components/mod.rs`で定義済み
- TutorialPanel, TutorialStepText: `setup/ui/mod.rs`で実装済み
- tutorial.rs: `systems/tutorial.rs`で8ステップ実装済み

---

## Phase B: アーキテクチャ再設計（中期）✅ 大部分完了

**参照**: [architecture-redesign.md](architecture-redesign.md)

### B.1 準備 ✅ 完了

- `core/`: inventory.rs, network.rs, recipe.rs 実装済み
- 機能コンポーネント: ItemAcceptor, ItemEjector, Crafter, MachineInventory, PowerConsumer 定義済み
- MachineDescriptor: MINER, FURNACE, CRUSHER 定義済み
- `ui/widgets.rs`: spawn_slot, spawn_button 実装済み

### B.2 物流インフラ分離 ✅ 完了

- `logistics/conveyor.rs`: 557行、コンベアシステム実装済み

### B.3 機械統合 ✅ 完了

- `machines/`: miner.rs, furnace.rs, crusher.rs で個別実装
- 共通コンポーネント: `components/machines.rs` で定義

### B.4 UI統合 ✅ 完了

- `ui/`: machine_ui.rs, storage_ui.rs, widgets.rs
- `setup/ui/`: inventory_ui.rs, mod.rs
- `systems/`: inventory_ui.rs, debug_ui.rs
- 3箇所に分散しているが、各々の責務が明確で統合の必要なし

### B.5 セーブ統合 ✅ 完了

- `save/`: format.rs, systems.rs 実装済み

### B.6 最適化 ✅ 完了

- main.rs: GamePlugin化済み（50行のみ）
- updater/: feature gate実装済み
- debug/: 既存のまま維持

**現状**: 22,567行（アーキテクチャ整備完了、行数削減は必須ではない）

---

## 現在のタスク（2026-01-06）

| # | タスク | 状態 | 備考 |
|---|--------|------|------|
| 1 | チュートリアル中にメインクエストを完全に非表示にする | ✅完了 | `tutorial.rs:185-191` |
| 2 | 製錬炉が鉱石の搬入を受け付けない問題を修正 | ✅完了 | テスト8件通過、`logistics/conveyor.rs:295-340` |
| 3 | チュートリアルクエストで個数関係のものにプログレスバーを表示 | ✅完了 | `tutorial.rs:216-248`、スクショ確認済 |
| 4 | 鉱石バイオーム表示をシンプルに、他UIと色を統一 | ✅完了 | 左上「[Cu] 銅鉱脈」表示確認済 |

### 根本問題タスク（2026-01-06 Claude+Gemini調査）

| # | タスク | 状態 | リスク | 発見者 |
|---|--------|------|--------|--------|
| 5 | 座標変換ユーティリティ統一 | ✅完了 | 高 | 両方 |
| 6 | 機械出力ロジック共通化 | ✅完了 | 高 | 両方 |
| 7 | 機械インタラクション共通化 | ✅完了 | 高 | Gemini |
| 8 | カーソル管理の集約 | ✅完了 | 中 | Gemini |
| 9 | 並列worktree重複コミット検出 | ✅完了 | 低 | - |

#### タスク5: 座標変換ユーティリティ統一

**問題**: `Transform.translation` と `component.position` の二重管理

```rust
// 現状（不一貫）
let pos = transform.translation.floor().as_ivec3();  // 破壊時
let furnace = Furnace { position: block_pos, ... };  // 作成時
if conveyor_pos == transform.translation.floor()... // 搬入判定
```

**過去のバグ**:
- コミット 1dd61f4: 0.5ブロックズレ修正
- コミット 30f31a0: 製錬炉搬入位置判定修正

**対策**:
- `src/utils.rs` に `grid_to_world(IVec3) -> Vec3` / `world_to_grid(Vec3) -> IVec3` 作成
- `component.position` を SSOT とし、Transform.translation は視覚化専用

#### タスク6: 機械出力ロジック共通化

**問題**: miner.rs, furnace.rs, crusher.rs に**ほぼ同一のコード**が3箇所

```rust
// 3ファイルで重複
let output_pos = machine.position + machine.facing.to_ivec3();
for mut conveyor in conveyor_query.iter_mut() {
    if conveyor.position == output_pos {
        // 同じ転送ロジック
    }
}
```

**出力先の不整合**（Gemini発見）:

| マシン | コンベア | 精錬炉 | 粉砕機 |
|--------|---------|--------|--------|
| Miner | ✅ | ✅ | ✅ |
| Crusher | ✅ | ✅ | ❌ |
| Furnace | ✅ | ❌ | ❌ |

**対策**: 共通関数 `fn try_transfer_to_output(...)` に統一

#### タスク7: 機械インタラクション共通化

**問題**: 各マシンの `_interact` 関数が同じ制御フローを個別に実装

```rust
// miner.rs, furnace.rs, crusher.rs で重複
if inventory_open.0 || command_state.open || cursor_state.paused { return; }
if interacting.0.is_some() && (e_pressed || esc_pressed) { ... }
if !mouse_button.just_pressed(MouseButton::Right) { return; }
// ... レイキャスト処理 ...
```

**対策**: 共通ヘルパー `fn handle_machine_interaction(...)` に抽出

#### タスク8: カーソル管理の集約

**問題**: 各システムが直接カーソル状態を操作

```rust
window.cursor_options.grab_mode = CursorGrabMode::Locked;
window.cursor_options.visible = false;
```

**リスク**: UIが増えると競合が発生しやすい

**対策**: `CursorManager` リソース導入、各システムはリクエストを送る形に

#### タスク9: 並列worktree重複コミット検出

**問題**: 同時刻に同名コミットが発生（2026-01-05に2件）

**対策**: `parallel-run.sh finish` 時に同名コミットをチェック

### UI状態管理再設計（2026-01-06 Claude+Gemini設計）

**問題**: UI状態が6つの独立リソースに分散、ESC処理7箇所・E処理4箇所に分散

**採用案**: 案4「Event駆動型UIスタック」

#### 現状の問題点

| 問題 | 詳細 |
|------|------|
| Single Source of Truth欠如 | `InventoryOpen`, `InteractingFurnace`, `InteractingCrusher`, `InteractingMiner`, `CommandInputState`, `CursorLockState` が独立 |
| 入力処理の競合 | ESC 7箇所、E 4箇所に分散。「インベントリ閉じた瞬間にポーズが開く」等の競合 |
| 排他制御の重複 | 各システムが手動で他UIの状態をチェック |

#### 新設計

```rust
// UIの状態（階層化）
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiState {
    HUD,                          // 通常プレイ
    Inventory,                    // インベントリ (E)
    GlobalInventory,              // 倉庫 (Tab)
    Command,                      // コマンド (T/Slash)
    Machine(Entity),              // 機械UI（種類はEntityのComponentで判定）
    Menu(MenuScreen),             // メニュー階層
    Modal { title: String, ... }, // 確認ダイアログ等
}

#[derive(Clone, PartialEq, Eq)]
pub enum MenuScreen {
    Main, Settings, Keybinds, Graphics, Audio,
}

// スタック管理（ESCで戻る対応）
#[derive(Resource, Default)]
pub struct UiManager {
    stack: Vec<UiState>,
}

// 操作はEvent経由（疎結合）
#[derive(Event)]
pub enum UiAction {
    Open(UiState),  // 新しいUIを開く
    Back,           // 戻る（ESC）
    CloseAll,       // 全部閉じる
}
```

#### 実装ステップ

| # | タスク | 状態 | 工数 |
|---|--------|------|------|
| UI-1 | `UiState`, `UiManager`, `UiAction` 定義 | 未着手 | 小 |
| UI-2 | `ui_navigation_system` 実装（Event処理） | 未着手 | 小 |
| UI-3 | `ui_input_system` 実装（ESC/E/Tab集約） | 未着手 | 中 |
| UI-4 | 既存システムから入力処理を削除、`UiManager`参照に変更 | 未着手 | 中 |
| UI-5 | 旧リソース（`InventoryOpen`等）を削除 | 未着手 | 小 |
| UI-6 | カーソル管理を`UiManager`に統合 | 未着手 | 小 |

#### 設計のポイント

- **機械UIは`Machine(Entity)`のみ**: 種類はEntityのComponentで判定（拡張性）
- **メニュー階層は`MenuScreen`**: 設定→キーバインド等の階層遷移に対応
- **スタック構造**: ESCで「前の画面に戻る」が自然に実装可能
- **Event駆動**: ボタンクリックもキーボードも`UiAction`を発行するだけ

#### 参考（Factorioパターン）

- ESCキー: 最前面ウィンドウを閉じる、なければメインメニュー
- 機械UI + インベントリが同時表示（同じモード内）
- メニューはゲームをポーズ、インベントリはポーズしない

---

### 設定画面（UI再設計後に実装）

**前提**: UI-1〜6 完了後（`UiState::Menu(MenuScreen::Settings)` で統合）

#### 設定データ構造

```rust
// settings.rs
#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct GameSettings {
    // グラフィック
    pub view_distance: i32,      // 1-5
    pub fov: f32,                // 60-120
    pub vsync: bool,

    // 操作
    pub mouse_sensitivity: f32,  // 0.1-2.0
    pub invert_y: bool,
    pub keybinds: KeyBindings,

    // オーディオ（将来用）
    pub master_volume: f32,      // 0.0-1.0
    pub bgm_volume: f32,
    pub sfx_volume: f32,

    // ゲーム
    pub language: Language,
    pub show_tutorial: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            view_distance: 3,
            fov: 90.0,
            vsync: true,
            mouse_sensitivity: 1.0,
            invert_y: false,
            keybinds: KeyBindings::default(),
            master_volume: 1.0,
            bgm_volume: 0.7,
            sfx_volume: 1.0,
            language: Language::Japanese,
            show_tutorial: true,
        }
    }
}
```

#### UIレイアウト

```
┌─────────────────────────────────┐
│  設定                      [×] │
├─────────────────────────────────┤
│ [グラフィック] [操作] [音声]    │
├─────────────────────────────────┤
│                                 │
│ 描画距離    [━━━━●━━] 3        │
│ 視野角      [━━━━━●━] 90°      │
│ VSync       [✓]                │
│                                 │
│ マウス感度  [━━●━━━━] 1.0      │
│ Y軸反転     [ ]                │
│                                 │
├─────────────────────────────────┤
│     [デフォルトに戻す] [適用]  │
└─────────────────────────────────┘
```

#### 永続化

```
saves/settings.json  （セーブフォルダと同じ場所）
```

#### 実装タスク

| # | タスク | 状態 | 工数 | 依存 |
|---|--------|------|------|------|
| SET-1 | `GameSettings` リソース定義 | 未着手 | 小 | - |
| SET-2 | `SettingsPlugin` 実装（読み込み・保存） | 未着手 | 小 | SET-1 |
| SET-3 | `config.toml` → `GameSettings` 初期値読み込み | 未着手 | 小 | SET-2 |
| SET-4 | スライダーWidget実装 | 未着手 | 中 | - |
| SET-5 | トグルWidget実装 | 未着手 | 小 | - |
| SET-6 | 設定画面UI（タブ付きパネル） | 未着手 | 中 | UI-1〜6, SET-4, SET-5 |
| SET-7 | 設定変更の即時反映 | 未着手 | 小 | SET-6 |
| SET-8 | キーバインド画面 | 未着手 | 大 | SET-6 |

#### 優先順位

| 順位 | 内容 |
|------|------|
| 1 | SET-1〜3（設定データ基盤） |
| 2 | SET-4〜5（UIウィジェット） |
| 3 | SET-6〜7（設定画面本体） |
| 4 | SET-8（キーバインド、後回しでOK） |

#### 現状の問題点

- `config.toml` が存在するが読み込まれていない
- `constants.rs` にハードコードされた値が使われている
- 設定変更にはコード修正が必要

#### 完成後

```
ESC → メインメニュー → 設定 → グラフィック/操作/音声タブ
                           ↓
                    スライダー/トグルで調整
                           ↓
                    [適用] で saves/settings.json に保存
                           ↓
                    次回起動時に自動読み込み
```

---

### ボクセル基盤改善タスク（2026-01-06 追加）

| # | タスク | 状態 | 効果 | 工数 |
|---|--------|------|------|------|
| 10 | Greedy meshing実装 | 未着手 | 頂点数50%減、GPU負荷大幅改善 | 中 |
| 11 | ChunkData HashMap削除 | 未着手 | メモリ50%減 | 小 |
| 12 | 差分メッシュ更新 | 未着手 | 隣接チャンク再生成のCPU負荷減 | 中 |
| 13 | LOD実装 | 未着手 | 遠距離描画軽量化 | 大 |
| 14 | パレット方式導入 | 未着手 | メモリ1/4〜1/8（将来対応） | 中 |

#### タスク10: Greedy meshing

**問題**: 現状は単純な面単位メッシュ生成。同一ブロックタイプの連続面を結合していない。

```
現状: ブロック1個 = 最大6面 × 4頂点 = 24頂点
改善後: 連続面を1ポリゴンに → 頂点数50-70%削減
```

**実装箇所**: `src/world/mod.rs` の `generate_mesh_with_neighbors()`

#### タスク11: ChunkData HashMap削除

**問題**: `blocks: Vec` と `blocks_map: HashMap` の二重管理でメモリ2倍消費

```rust
// 現状（冗長）
pub struct ChunkData {
    pub blocks: Vec<Option<BlockType>>,      // フラット配列
    pub blocks_map: HashMap<IVec3, BlockType>, // 互換層（削除対象）
}
```

**実装箇所**: `src/world/mod.rs`

#### タスク12: 差分メッシュ更新

**問題**: ブロック変更時に隣接4チャンクのメッシュを全再生成

**改善案**: 境界面のみ更新する差分アルゴリズム

**実装箇所**: `src/systems/chunk.rs:186-200`

#### タスク13: LOD実装

**問題**: 遠距離チャンクも近距離と同じ詳細度でレンダリング

**改善案**: 距離に応じた詳細度切り替え（例: 2チャンク以上で簡略化）

#### タスク14: パレット方式導入

**問題**: ブロック種類増加時のメモリ効率が悪い

**現状**:
```rust
blocks: Vec<Option<BlockType>>  // 各ブロックにEnumを直接格納
```

**改善案（Minecraft方式）**:
```rust
struct ChunkData {
    palette: Vec<BlockType>,  // このチャンク内で使われる種類のリスト
    blocks: Vec<u8>,          // パレットへのインデックス（小さい整数）
}
```

**効果**: チャンク内ブロック種類が少ない場合、メモリ1/4〜1/8に削減

**優先度**: 低（ブロック種類100+またはVIEW_DISTANCE大幅増加時に検討）

---

## 将来タスク（v0.3以降）

以下は現時点では着手しない。v0.2完成 + アーキテクチャ安定後に検討。

| 機能 | 詳細 |
|------|------|
| **データ駆動設計** | Descriptor化（下記参照） |
| 電力システム | 発電機・導管・消費 |
| 流体パイプ | ポンプ・パイプ・タンク |
| マルチプレイ | WebSocket同期 |
| Modding API | Lua/WASM |

---

## Phase C: データ駆動設計（コンテンツ追加を楽にする）

**目標**: 新コンテンツ追加 = game_spec/*.rs にデータ追加するだけ

### 現状 vs 理想

| 追加するもの | 現状 | Descriptor化後 |
|--------------|------|----------------|
| 新ブロック | 5-6箇所修正、100行 | 1箇所、10行 |
| 新アイテム | 3-4箇所修正、50行 | 1箇所、8行 |
| 新機械 | 500-600行 | 20行 |
| 新レシピ | 1箇所、5行 | ✅ 変わらず |

### C.1 BlockDescriptor

```rust
// game_spec/blocks.rs
pub struct BlockDescriptor {
    pub id: BlockType,
    pub name: &'static str,
    pub hardness: f32,              // 採掘時間係数
    pub tool: ToolType,             // 適正ツール
    pub drops: &'static [(BlockType, u32)],
    pub texture: &'static str,
    pub is_solid: bool,
    pub is_transparent: bool,
}

pub const BLOCKS: &[BlockDescriptor] = &[
    BlockDescriptor {
        id: BlockType::Stone,
        name: "石",
        hardness: 1.5,
        tool: ToolType::Pickaxe,
        drops: &[(BlockType::Cobblestone, 1)],
        texture: "stone.png",
        is_solid: true,
        is_transparent: false,
    },
    // ...
];
```

**実装タスク**:

| # | タスク | 工数 |
|---|--------|------|
| C.1-1 | BlockDescriptor構造体定義 | 小 |
| C.1-2 | 既存BlockTypeをBLOCKS配列に移行 | 中 |
| C.1-3 | meshes.rsのmatch文をBLOCKS参照に置換 | 小 |
| C.1-4 | block_operations.rsのmatch文を置換 | 小 |
| C.1-5 | inventory_ui.rsの表示名をBLOCKS参照に | 小 |

### C.2 ItemDescriptor

```rust
// game_spec/items.rs
pub struct ItemDescriptor {
    pub id: BlockType,
    pub name: &'static str,
    pub stack_size: u32,            // 最大スタック数（ツール=1）
    pub category: ItemCategory,
    pub tooltip: &'static str,
    pub sprite: &'static str,
    pub durability: Option<u32>,    // 耐久値（ツール用）
    pub mining_speed: Option<f32>,  // 採掘速度倍率
}

pub enum ItemCategory {
    Block,      // 設置可能ブロック
    Material,   // 素材（インゴット等）
    Tool,       // ツール
    Machine,    // 機械
}
```

**実装タスク**:

| # | タスク | 工数 |
|---|--------|------|
| C.2-1 | ItemDescriptor構造体定義 | 小 |
| C.2-2 | 既存アイテムをITEMS配列に移行 | 中 |
| C.2-3 | インベントリUIをITEMS参照に | 小 |
| C.2-4 | ツールチップ自動生成 | 小 |

### C.3 MachineDescriptor + UIジェネレータ

```rust
// game_spec/machines.rs
pub struct MachineDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub block_type: BlockType,

    // スロット構成（これだけでUIが自動生成される）
    pub input_slots: u8,
    pub output_slots: u8,
    pub fuel_slot: bool,

    // 処理
    pub process_time: f32,
    pub recipes: &'static [Recipe],

    // モデル
    pub model: &'static str,
}

pub const MACHINES: &[MachineDescriptor] = &[
    MachineDescriptor {
        id: "miner",
        name: "採掘機",
        block_type: BlockType::Miner,
        input_slots: 0,
        output_slots: 1,
        fuel_slot: false,
        process_time: 1.5,
        recipes: &[],  // 特殊処理
        model: "machines/miner.glb",
    },
    MachineDescriptor {
        id: "furnace",
        name: "精錬炉",
        block_type: BlockType::Furnace,
        input_slots: 1,
        output_slots: 1,
        fuel_slot: true,
        process_time: 2.0,
        recipes: FURNACE_RECIPES,
        model: "machines/furnace.glb",
    },
    // ...
];
```

**実装タスク**:

| # | タスク | 工数 |
|---|--------|------|
| C.3-1 | MachineDescriptor拡張（スロット情報追加） | 小 |
| C.3-2 | spawn_machine_ui()ジェネレータ実装 | 中 |
| C.3-3 | 既存機械UIをジェネレータ経由に移行 | 中 |
| C.3-4 | 共通machine_tick_system実装 | 中 |
| C.3-5 | 個別機械ファイル(miner.rs等)を簡略化 | 小 |

### C.4 レジストリシステム

```rust
// core/registry.rs
pub struct GameRegistry {
    pub blocks: HashMap<BlockType, &'static BlockDescriptor>,
    pub items: HashMap<BlockType, &'static ItemDescriptor>,
    pub machines: HashMap<BlockType, &'static MachineDescriptor>,
}

impl GameRegistry {
    pub fn new() -> Self {
        // 起動時にBLOCKS, ITEMS, MACHINESからHashMap構築
        // O(1)参照を実現
    }

    pub fn block(&self, id: BlockType) -> Option<&BlockDescriptor> { ... }
    pub fn item(&self, id: BlockType) -> Option<&ItemDescriptor> { ... }
    pub fn machine(&self, id: BlockType) -> Option<&MachineDescriptor> { ... }
}
```

### 実装順序

```
C.1 BlockDescriptor ─┐
C.2 ItemDescriptor ──┼─→ C.4 レジストリ ─→ 完成
C.3 MachineDescriptor┘
```

### 優先順位

| 順位 | 対象 | 理由 |
|------|------|------|
| 1 | **C.3 機械** | 最も追加工数が大きい（500行→20行） |
| 2 | **C.1 ブロック** | 次に散らばっている |
| 3 | **C.2 アイテム** | ブロックと統合して整理 |
| 4 | **C.4 レジストリ** | 全体の参照効率化 |

### 完成後の新コンテンツ追加フロー

```
1. game_spec/machines.rs にMachineDescriptor追加（10行）
2. game_spec/recipes.rs にレシピ追加（5行）
3. assets/models/ に3Dモデル配置
4. 完了（UIもシステムも自動生成）
```

**Modding APIへの発展**: Phase C完成後、game_spec/*.rsをJSONに外部化すれば、非プログラマでもコンテンツ追加可能になる

---

## 完了済みタスク

<details>
<summary>クリックで展開</summary>

### リファクタリング
- [x] block_operations.rs 分割 (1001行→3ファイル)
- [x] ui_setup.rs 分割 (977行→3ファイル)
- [x] targeting.rs 分割 (759行→4ファイル)
- [x] command_ui.rs 分割 (826行→4ファイル)
- [x] MachineSystemsPlugin 作成
- [x] UIPlugin 作成
- [x] SavePlugin 作成

### パフォーマンス改善 (旧Phase 1)
- [x] ハイライトメッシュキャッシュ化
- [x] O(N²)コンベア転送→HashMap化
- [x] Vec::contains()→HashSet化
- [x] クエストデータ変換キャッシュ

### セキュリティ・エラー処理 (旧Phase 2)
- [x] unwrap()削減 (72箇所→17箇所)
- [x] 配列インデックス範囲チェック
- [x] コマンドパス走査防止
- [x] NaN/Infinity処理

### v0.2機能 (旧Phase 3)
- [x] GlobalInventory基盤
- [x] 機械設置/撤去
- [x] 8列グリッドレイアウト
- [x] ページネーション
- [x] カテゴリタブ・検索機能
- [x] 納品ボタン
- [x] 機械入出力システム
- [x] バイオーム採掘システム

### テスト強化 (旧Phase 4)
- [x] カバレッジ計測 (8.54%全体、ロジック70%+)
- [x] コンベア統合テスト
- [x] セーブ/ロード往復テスト
- [x] UIインタラクションテスト
- [x] SSIM比較テスト
- [x] ファジング基盤

### プラットフォーム再設計 (旧Phase 10)
- [x] PlatformBlock追加
- [x] DeliveryPlatform.delivered削除
- [x] GlobalInventory経由に変更

</details>

---

## 実行順序マトリクス

```
【v0.2完成】
A.1 UIテーマ ──┐
A.2 バイオームUI ├─→ v0.2リリース
A.3 チュートリアル ┘

【アーキテクチャ再設計】（v0.2完成後）
B.1 準備 ─→ B.2 物流分離 ─→ B.3 機械統合 ─→ B.4 UI統合 ─→ B.5 セーブ ─→ B.6 最適化
                                                                              │
                                                                              ↓
                                                                         v0.3検討
```

---

## 合見積サマリー（2026-01-04）

| 観点 | Claude | Gemini | 採用 |
|------|--------|--------|------|
| 機械設計 | Machineトレイト | ECSコンポジション | **Gemini** |
| コンベア | 機械として統合 | 物流インフラ分離 | **Gemini** |
| UI | 共通化 | Entity Observers | **両方** |
| 移行 | 3Phase | 垂直分割 | **Gemini** |

**結論**: ECSの特性を活かし、`tick()`メソッドに依存せず、機能コンポーネント（ItemAcceptor, Crafter等）で構成

---

## 次のアクション

**Phase A・B 完了** ✅

現在の状態:
- v0.2機能: 全て実装済み
- アーキテクチャ: 整備完了
- テスト: 280件通過
- Clippy警告: 0件

次のステップ:
1. **v0.2リリース準備** - リリースノート作成、タグ付け
2. **v0.3検討** - 将来タスクから優先度を決定

---

## 将来設計メモ（v0.3以降）

### 納品プラットフォームのスループット制限

**設計原則**: 入力無制限・容量無制限・**出力制限**

```
納品プラットフォーム
├── 入力: 無制限（なんでも受け入れ）
├── 容量: 無制限（無限倉庫OK）
├── アイテム出力: 4個/tick（ボトルネック）
└── 電力出力: 100W（上限固定）
```

**効果**:
- 序盤は納品PFだけで十分 → シンプル
- 生産規模拡大 → スループット不足 → 専用ブロックが必要に
- 「とりあえず全部入れる」でも動くが、**最適ではない**

### ブロック別スループット比較

| ブロック | 容量 | 出力 | 用途 |
|----------|------|------|------|
| 納品PF | ∞ | 4/tick | 汎用・序盤 |
| 小型倉庫 | 1,000 | 16/tick | 中盤 |
| 大型倉庫 | 10,000 | 64/tick | 終盤 |
| 蓄電池 | 10,000Wh | 500W | 電力バッファ |
| 大型蓄電池 | 100,000Wh | 2,000W | 大規模工場 |

### 電力システム

| フェーズ | 電力源 | 出力 | プレイヤー体験 |
|----------|--------|------|---------------|
| 序盤 | 納品PF | 100W | 「電気ってこう使う」を学ぶ |
| 中盤 | 石炭発電機 | 200W | 燃料管理 |
| 中盤 | ソーラー | 50W（昼のみ） | 昼夜サイクル |
| 終盤 | 蒸気タービン | 500W | 水+熱源 |
| 終盤 | 原子炉 | 2,000W | 高コスト・高リスク |

### ゲームプレイの流れ

1. 最初は納品PF一つで全部回る
2. 機械増える → 供給追いつかない
3. 「倉庫建てて分散させよう」← 自然な動機
4. 「発電機作って電力増やそう」← 同様の動機

**液体・気体も同じパターンで統一可能**
