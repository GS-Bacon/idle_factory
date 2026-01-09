# 作業ログ

## 2026-01-09: テクスチャシステム実装（T1-T5）

### 概要

ブロックにMinecraft風テクスチャシステムを実装。テクスチャアトラス、面別テクスチャ、blockstates JSON、リソースパック、MOD拡張ポイントを完成。

### 実装内容

#### T1: テクスチャアトラス基盤

| ファイル | 内容 |
|----------|------|
| `src/textures/atlas.rs` | `TextureAtlas`, `UVRect`, `UVCache`, `BlockTextureConfig` |
| 仕様 | 256x256アトラス、16x16タイル、UV座標マッピング |

**BlockTextureConfig**:
- `All`: 全面同一テクスチャ
- `TopSideBottom`: 上/横/下で異なるテクスチャ
- `PerFace`: 各面で異なるテクスチャ

#### T2: 面別テクスチャ対応

| ファイル | 内容 |
|----------|------|
| `src/world/mod.rs` | `generate_mesh_textured()` - UV付きGreedy meshing |
| `src/systems/chunk.rs` | `receive_chunk_meshes` でテクスチャ版メッシュ使用 |

**草ブロック設定**:
- 上面: `grass_top` (緑)
- 側面: `grass_side` (茶緑)
- 底面: `dirt` (茶)

#### T3: blockstates JSONシステム

| ファイル | 内容 |
|----------|------|
| `src/textures/blockstates.rs` | `BlockstateRegistry`, `BlockstateDefinition` |
| `assets/blockstates/` | `grass.json`, `stone.json`, `conveyor.json` |

#### T4: リソースパック読み込み

| ファイル | 内容 |
|----------|------|
| `src/textures/resource_pack.rs` | `ResourcePackManager` - テクスチャ上書き機能 |

#### T5: MOD拡張ポイント

| ファイル | 内容 |
|----------|------|
| `src/textures/resolver.rs` | `TextureResolver` trait, Connected/Random resolvers |
| `src/modding/handlers/textures.rs` | WebSocket API (`texture.list`, `texture.get_atlas_info`, `texture.register_resolver`) |

### 修正した問題

| 問題 | 対応 |
|------|------|
| `receive_chunk_meshes` がテクスチャなし版を使用 | `generate_chunk_mesh_textured` に変更 |
| 隣接チャンク再生成もテクスチャなし | 同様に修正 |

### 動作確認

| 項目 | 結果 |
|------|------|
| アトラス構築 | ✅ 7テクスチャ |
| UVキャッシュ | ✅ 30エントリ (5ブロック × 6面) |
| 地形レンダリング | ✅ 草の緑色が表示 |
| 全テスト | ✅ 通過 |

### 技術判断

| 判断 | 理由 |
|------|------|
| フォールバック色を使用 | 実テクスチャPNGがなくても動作するため |
| UVCacheをClone可能に | 非同期タスクに渡すため |
| Resolver構造体は未使用警告許容 | 将来のMOD用API |

### 学び

| 問題 | 教訓 |
|------|------|
| メッシュ生成関数が複数箇所で呼ばれる | **全箇所を確認してテクスチャ版に統一** |
| ログで問題を特定 | 「Greedy mesh」vs「Textured mesh」でどちらが使われているか判別 |

---

## 2026-01-09: シナリオテスト改善・イベント購読システム実装

### 概要

シナリオテストにイベント購読機能を追加。E2Eテストを新構造に分割。全9シナリオテスト通過を確認。

### 実装内容

#### 1. イベント購読システム (Rust)

| ファイル | 内容 |
|----------|------|
| `src/modding/handlers/test.rs` | `test.subscribe_event`, `test.unsubscribe_event` ハンドラ追加 |
| `src/modding/event_notifier.rs` | イベント通知プラグイン新規作成 |
| `src/modding/server.rs` | イベント購読ルーティング追加 |

**対応イベント**:
- `item.delivered` - アイテム納品
- `machine.completed` - 機械加工完了
- `block.placed` - ブロック配置
- `block.removed` - ブロック削除

#### 2. シナリオランナー拡張 (JavaScript)

`scripts/run-scenario.js` に以下アクションを追加:

| アクション | 説明 |
|-----------|------|
| `subscribe` | イベント購読開始 |
| `unsubscribe` | イベント購読解除 |
| `wait_for_event` | イベント待機（タイムアウト付き） |
| `assert_event` | イベントデータ検証 |

#### 3. E2Eテスト構造改善

| 変更前 | 変更後 |
|--------|--------|
| `e2e_test.rs` (5593行, 148テスト) | `tests/e2e/` ディレクトリに分割 |
| 単一ファイル | 10ファイル（カテゴリ別） |

**新構造**:
```
tests/
├── e2e_new.rs (エントリポイント, 58テスト)
├── e2e/
│   ├── common.rs (共通ヘルパー)
│   ├── world.rs
│   ├── inventory.rs
│   ├── machines.rs
│   ├── automation.rs
│   ├── quest.rs
│   ├── events.rs
│   ├── save.rs
│   ├── ui.rs
│   └── bugs.rs
└── e2e_test.rs (レガシー, 148テスト)
```

### シナリオテスト結果

全9シナリオ通過:

| シナリオ | 結果 |
|----------|------|
| bug_pause_cursor_lock.toml | ✓ |
| bug_tutorial_quest_ui.toml | ✓ |
| esc_pause_menu.toml | ✓ |
| event_subscription.toml | ✓ |
| inventory_esc_close.toml | ✓ |
| inventory_toggle.toml | ✓ |
| pause_menu_cursor.toml | ✓ |
| pause_menu_resume.toml | ✓ |
| ui_cursor_lock.toml | ✓ |

### 発見・修正した問題

| 問題 | 対応 |
|------|------|
| `TogglePause`アクションが未使用 | シナリオで`Cancel`を使用するよう修正 |
| `tests/scenarios/mod.rs` 不要ファイル | 削除 |

### 学び

| 項目 | 内容 |
|------|------|
| GameAction設計 | `TogglePause`と`Cancel`が両方ESCバインドだが、実装は`Cancel`のみ使用 |
| イベント駆動テスト | ポーリングより信頼性高い（wait_for_eventでタイミング問題回避） |

### テスト結果

| 項目 | 値 |
|------|-----|
| 全テスト | 718件通過 |
| シナリオテスト | 9件通過 |
| Clippy警告 | 0件 |

---

## 2026-01-09: 並列実行システム構築

### 概要

サブエージェントの並列化システムを設計・実装。調査フェーズ（worktree不要）と実装フェーズ（ファイル分割並列）の2段階並列化を可能にした。

### 作成物

| ファイル | 内容 |
|----------|------|
| `.claude/plan-template.md` | 並列計画のテンプレート |
| `scripts/parallel-plan.sh` | 計画の検証・実行コマンド生成 |
| `CLAUDE.md` 更新 | 並列化ルールの強化 |

### 並列化アーキテクチャ

```
Phase 1: 調査（並列、worktree不要）
  Task(Explore) × N → masterを読むだけ
      ↓
  結果を集約
      ↓
Phase 2: 実装（worktree内でファイル分割並列）
  worktree作成
      ↓
  Group A: Task × N（依存なし）→ 同時実行
  Group B: Task × N（A依存）→ A完了後に同時実行
  Group C: Task × N（B依存）→ B完了後に同時実行
      ↓
Phase 3: 検証（直列）
  cargo build && cargo test && cargo clippy
```

### 設計判断

| 判断 | 理由 |
|------|------|
| サブエージェントはビルドしない | 依存関係でビルドエラーになる、最後に1回で十分 |
| 調査はworktree不要 | 読み取り専用なので競合しない |
| 同一ファイルは1エージェント限定 | 競合防止 |
| 依存関係をグループ化 | 型定義→使用の順序を保証 |

### 学び

| 問題 | 対策 |
|------|------|
| 階層的worktreeは不可 | フラット並列 + ファイル分割並列で代替 |
| サブエージェント間の依存 | グループ化して順序制御 |
| コンテキスト断絶 | 調査結果を集約してから実装フェーズへ |

---

## 2026-01-09: UI Visibility システム アーキテクチャレビュー

### 概要

現在進行中のUI表示制御一元化（UIVisibilityController）の設計をレビュー。技術的負債と改善計画を策定。

### レビュー内容

#### 良い点

| 観点 | 評価 |
|------|------|
| Single Source of Truth | ✅ `create_default_rules()` に全ルール集約 |
| テスト可能性 | ✅ Controller は単体テスト可能 |
| Mod対応 | ✅ `UIId::Mod` と `ConditionKey::Custom` で拡張可能 |
| コード削減 | ✅ -75行（143削除、68追加） |
| 責務分離 | ✅ 各システムは副作用のみ、表示制御は委譲 |

#### 発見した問題

| 問題 | 重大度 | 詳細 |
|------|--------|------|
| Query の Without チェーン | 中 | UIが増えるたびにシステム修正が必要 |
| Machine UI の除外 | 中 | 一元管理から除外されており不整合 |

**Without チェーン問題の例**:
```rust
// 現状: UIごとにQueryが必要、Withoutが爆発
Query<&mut Visibility, (With<InventoryUI>, Without<QuestUI>, Without<TutorialPanel>, ...)>
```

### 改善計画（UI.1）

| タスク | 内容 |
|--------|------|
| UI.1.1 | `UIVisibilityTarget` コンポーネント追加 |
| UI.1.2 | 各UI Spawn時に `UIVisibilityTarget` を付与（8箇所） |
| UI.1.3 | `update_all_ui_visibility` を単一Query化 |
| UI.1.4 | Machine UI を UIVisibilityController に統合 |
| UI.1.5 | Mod UI 登録 API 実装 (`ui.register`) |

**改善後の設計**:
```rust
#[derive(Component)]
pub struct UIVisibilityTarget {
    pub id: UIId,
}

// 1つのQueryで全UI処理
fn update_all_ui_visibility(
    controller: Res<UIVisibilityController>,
    mut query: Query<(&UIVisibilityTarget, &mut Visibility)>,
) {
    for (target, mut vis) in query.iter_mut() {
        *vis = controller.evaluate(&target.id);
    }
}
```

### 学び

| 問題 | 教訓 |
|------|------|
| M2設計時に予測できなかった | **実装してみないとわからない問題がある** |
| Bevy の Query 制約 | 書くまで「どれくらい酷いか」は不明 |
| Machine UI 除外の理由消失 | **「なぜ除外したか」をコメントに残すべき** |

**結論**: 設計時に完璧を目指すより、**実装 → レビュー → 改善** のサイクルが現実的。

### Mod UI の統合について

**改善後のフロー**:
```
1. Mod が ui.register API を呼ぶ（TOMLまたはWebSocket）
2. UIVisibilityController に VisibilityRules 登録
3. Mod が UI Entity を spawn（UIVisibilityTarget 付き）
4. update_all_ui_visibility が自動で表示制御
```

**Mod作者のメリット**:
- 表示条件をTOMLに書くだけ
- 他のUIとの衝突を心配しなくていい
- `InputState` の仕組みを理解しなくていい

---

## 2026-01-08: 設計議論セッション

### 概要

バイブコーディングの進め方、ゲームデザイン、Mod API設計について議論。

### 議論トピック

#### 1. バイブコーディング評価

| 項目 | 状態 |
|------|------|
| 期間 | 約25日（12/14開始） |
| コード行数 | 36,000行 |
| テスト数 | 397件 |
| 評価 | 非エンジニアとしては前例のない規模 |

**課題**: ドキュメントが多すぎる、複雑化のリスク

#### 2. ゲームデザイン方針

| 決定事項 | 内容 |
|----------|------|
| コア体験 | shapez 2風のライン引き + マイクラ工業の複数ルート |
| 開発順序 | シンプル1本道でクリア可能 → 複雑性追加（M5） |
| 見た目強化 | M4.5で実施（シェーダー、アニメーション等） |
| レシピ調整 | WebUIエディタで非エンジニアでも調整可能に |

#### 3. Mod API設計

**ファイル構成（カテゴリ別集約）決定**:
```
src/modding/
├── api/
│   ├── mod.rs         ← 集約
│   ├── inventory.rs
│   ├── machine.rs
│   ├── world.rs
│   ├── event.rs
│   └── log.rs
├── hooks/
│   ├── mod.rs         ← 集約
│   ├── machine.rs
│   ├── inventory.rs
│   └── world.rs
└── registry.rs
```

**設計方針**:
- 最初からカテゴリ別構造
- マクロ方式は不採用（デバッグ難易度回避）
- API追加 = 該当ファイルに追記
- 新カテゴリ = 新ファイル + mod.rsに1行

#### 4. 物理演算の整理

| 種類 | 対応 |
|------|------|
| リアル物理（エリトラ等） | ❌ 本体では対応しない |
| 簡易物理（水流、ファン） | ✅ ルールベースで実装可能 |
| Create風回転 | ✅ Core Modで実装可能 |

**結論**: マイクラと同じルールベース方式で十分

#### 5. 実装計画への追加

| タスク | 内容 |
|--------|------|
| W.7 | 特殊機械のCore Mod化（納品プラットフォーム等） |
| M4.5 E | レシピエディタ（WebUI） |
| M4.5 V | ビジュアル強化（シェーダー等） |

### 学び

- 「AIすごい」ではなく「人間+AIの協業パターン」として価値がある
- 見た目のインパクトは一般受けに重要
- API/フックは後から追加可能、変更は既存Modを壊す

---

## 2026-01-08: BlockType → ItemId 移行完了

### 概要

新アーキテクチャへの移行作業を継続。公開APIをItemIdベースに統一。

### 完了タスク

| タスク | 内容 |
|--------|------|
| core/inventory.rs | `ItemStack.item` を `BlockType` → `ItemId` に変更、legacy互換モジュール追加 |
| game_spec/machines.rs | `get_machine_spec_by_id(ItemId)` 追加、`MachineSpec.item_id()` メソッド追加 |
| game_spec/mod.rs | `QuestSpec.required_items_id()`, `rewards_id()`, `unlocks_id()` 追加 |
| components/machines.rs | `MachineInventory` を ItemId ベースに移行、`InputPort.filter` を ItemId に移行 |
| レガシー関数 | `get_smelt_output`, `can_crush`, `get_crush_output` に `#[deprecated]` 追加 |

### 移行状況

| 項目 | 状態 |
|------|------|
| BlockType 参照 | 584箇所（多くは const 定義で変更不可） |
| ItemId 使用 | 436箇所 |
| セーブデータ | ✅ 文字列ID化完了 |
| イベントシステム | ✅ 実装済み |
| レガシーコード | ✅ 削除済み |

### 技術的判断

- Rust の `const` 定義は BlockType が必要（コンパイル時評価の制約）
- 公開 API は ItemId ベース、内部実装は BlockType 許容
- Mod アイテムは ItemId で識別、レンダリング時は BlockType にフォールバック

### テスト結果

- 全テスト通過: 614件
- Clippy警告: 0件

---

## 2026-01-07: Phase D.0-D.14 完全実装

### 完了タスク

今日1セッションでPhase D.0からD.14まで全15モジュールを実装完了。

#### 基盤システム (D.0-D.5)

| Phase | 機能 | 実装内容 |
|-------|------|----------|
| D.0 | マルチ準備 | LocalPlayer(Entity) + Query<&PlayerInventory>パターン |
| D.1 | イベントシステム | GuardedEventWriter、BlockPlaced/Broken、MachineStarted等 |
| D.2 | 動的ID | Id<T> Phantom Type + StringInterner |
| D.3 | Mod API | ApiRequest/Response、ApiRegistry（12メソッド） |
| D.4 | データ駆動Mod | ItemDefinition、MachineDefinition、ModDataPack |
| D.5 | Blockbench | テクスチャ（base64→Image）、Bone階層、Keyframe |

#### QoL機能 (D.6-D.14)

| Phase | 機能 | 実装内容 |
|-------|------|----------|
| D.6 | マップ | チャンク探索、MapMarker、ズーム |
| D.7 | ブループリント | Blueprint、BlueprintLibrary、プレビュー |
| D.8 | クラフト | CraftingStation、CraftingRecipe、キュー管理 |
| D.9 | ストレージ | StorageBlock、StorageNetwork、容量管理 |
| D.10 | 統計 | TimeSeries、ProductionStats、ボトルネック分析 |
| D.11 | サウンド | SoundCategory、SoundSettings、SoundEmitter |
| D.12 | 実績 | Achievement、AchievementCondition、PlayerAchievements |
| D.13 | スキン | SkinCategory、SkinItem、EquippedSkins、レアリティ |
| D.14 | ロボット | RobotType(4種)、RobotCommand、RobotCommandQueue |

### 新規ファイル一覧

```
src/events/mod.rs
src/events/guarded_writer.rs
src/events/game_events.rs
src/core/id.rs
src/modding/mod.rs
src/modding/api.rs
src/modding/data.rs
src/map/mod.rs
src/blueprint/mod.rs
src/craft/mod.rs
src/storage/mod.rs
src/statistics/mod.rs
src/audio/mod.rs
src/achievements/mod.rs
src/skin/mod.rs
src/robot/mod.rs
```

### 追加・修正した既存ファイル

- `src/lib.rs`: 全モジュールのre-export追加
- `src/utils.rs`: GridPos に Serialize/Deserialize追加
- `src/components/machines.rs`: Direction に Serialize/Deserialize追加

### 現在の状態

| 項目 | 値 |
|------|-----|
| バージョン | **0.3.78** |
| コード行数 | **~25,000行** |
| テスト | **232件** 通過 |
| Clippy警告 | **0件** |

### 次の作業予定

D.15-D.20（高度機能）は以下の順序で実装予定:
1. D.15 電力 → D.16 液体 → D.17 信号 → D.18 線路
2. D.19 Mob
3. D.20 マルチプレイ（最後）

**理由**: D.0でマルチ準備済みのため、先にコンテンツを充実させる方が効率的。

---

## 2026-01-07: 設定UI・ボクセル最適化完了

### 完了タスク

#### Phase D: 設定システム完成 (v0.3.48)

| タスク | 内容 |
|--------|------|
| D.1 | GameSettings基盤（前日実装済み） |
| D.2 | ポーズメニューUI（Resume/Settings/Quit） |
| D.3 | 設定画面UI（スライダー、トグル、タブ切替） |
| D.4 | 設定の即時反映（VSync、Fullscreen、FOV） |

**新規ファイル**: `src/setup/ui/settings_ui.rs`

**設定画面機能**:
- グラフィック設定: 描画距離、FOV、VSync、フルスクリーン
- 操作設定: マウス感度、Y軸反転
- オーディオ設定: マスター音量、BGM、SE（UIのみ、音声未実装）

#### Phase E: ボクセル最適化 (v0.3.48)

| タスク | 内容 | 効果 |
|--------|------|------|
| E.1 | DirtyChunks導入 | ブロック変更時のバッチメッシュ更新（最大4チャンク/フレーム） |
| E.2 | LODシステム | 距離に応じた詳細度切替（Full/Medium/Low） |
| E.3 | パレット方式 | スキップ（BlockType 16種=1バイト、効果なし） |

**LOD詳細**:
- `ChunkLod::Full`: 距離0-1チャンク、全ブロック描画
- `ChunkLod::Medium`: 距離2-3チャンク、上部3層のみ（y >= GROUND_LEVEL - 2）
- `ChunkLod::Low`: 距離4+チャンク、表面層のみ（y == GROUND_LEVEL）
- `update_chunk_lod`システム: プレイヤー移動時にLOD自動更新（最大2更新/フレーム）

**DirtyChunks詳細**:
- ブロック変更時に即時メッシュ再生成せず、`DirtyChunks`にマーク
- `process_dirty_chunks`システムで毎フレーム最大4チャンク再生成
- 境界ブロック変更時は隣接チャンクも自動マーク

### 現在の状態

| 項目 | 値 |
|------|-----|
| バージョン | 0.3.48 |
| コード行数 | 約23,000行 |
| テスト | **344件** 通過 |
| Clippy警告 | **2件**（既存の複雑型警告） |

### 実装計画の完了状況

| Phase | 内容 | 状態 |
|-------|------|------|
| A | v0.2完成 | ✅ |
| B | アーキテクチャ再設計 | ✅ |
| C | データ駆動設計 | ✅ |
| D | 設定システム | ✅ |
| E | ボクセル最適化 | ✅ |

**残り**: 将来タスク（v0.3以降）のみ
- 電力システム、流体パイプ、マルチプレイ、Modding API

---

## 2026-01-06: パフォーマンス最適化・設定システム実装

### 完了タスク

#### 1. ChunkData HashMap削除 (v0.3.40)
- **変更**: `blocks_map: HashMap<IVec3, BlockType>`を削除
- **効果**: メモリ使用量の削減（HashMapオーバーヘッド分）
- **影響ファイル**:
  - `src/world/mod.rs`: ChunkData構造体、generate()、get_block()、set_block()、remove_block()
  - `src/systems/chunk.rs`: generate_chunk_sync()、receive_chunk_meshes()
  - `src/systems/block_operations/breaking.rs`: .copied()呼び出し削除
- **API変更**: `WorldData::get_block()` の戻り値: `Option<&BlockType>` → `Option<BlockType>`

#### 2. Greedy meshing実装 (v0.3.41)
- **変更**: チャンクメッシュ生成にGreedy meshingアルゴリズムを導入
- **効果**: 同じブロックタイプの隣接面を大きなクワッドに結合し、頂点数を大幅削減
- **技術詳細**:
  - 各面方向（±X, ±Y, ±Z）ごとにスライスを処理
  - 2Dマスクを作成し、可視面を記録
  - 同じブロックタイプの隣接セルを貪欲に結合
  - 正しいCCWワインディング順序で頂点を生成（各面方向に対応した外積計算）
- **影響ファイル**: `src/world/mod.rs`

#### 3. GameSettings基盤実装 (v0.3.42)
- **新規**: ユーザー設定の管理・永続化システム
- **設定項目**:
  | 設定 | 範囲 | デフォルト |
  |------|------|------------|
  | mouse_sensitivity | 0.0001-0.01 | 0.002 |
  | view_distance | 1-8 | 3 |
  | master_volume | 0.0-1.0 | 1.0 |
  | sfx_volume | 0.0-1.0 | 1.0 |
  | music_volume | 0.0-1.0 | 0.5 |
  | shadows_enabled | bool | true |
  | vsync_enabled | bool | true |
  | fullscreen | bool | false |
  | fov | 45-120 | 70.0 |
  | invert_y | bool | false |
- **機能**:
  - JSON形式での設定ファイル保存/読み込み
  - 開発環境: プロジェクトルートに保存
  - リリース環境: OSのconfig_dirに保存（dirs crate使用）
  - 自動保存（変更から1秒後にデバウンス）
  - 設定値のバリデーション
- **新規ファイル**: `src/settings.rs`
- **依存追加**: `dirs = "5.0"`

### 現在の状態

| 項目 | 値 |
|------|-----|
| バージョン | 0.3.42 |
| コード行数 | 約23,000行 |
| テスト | **131件** 通過 |
| Clippy警告 | **0件** |

---

## 2026-01-06: バイブコーディング実験の方針整理

### 議論の要約

このプロジェクトを「非エンジニアがAIだけで大規模ゲームを作れるか」の限界実験として位置づけ、手法を確立することを目指す方針を固めた。

### 技術選定の振り返り

| 選択 | 評価 | 理由 |
|------|------|------|
| Rust | ◎ 正解 | 型システムがAIのミスを防ぐ |
| Bevy | ○ 妥当 | 全部コードで完結、GUIエディタ不要 |
| 自前ボクセル | △ 微妙 | ライブラリ未成熟で結局同じ工数 |
| 依存最小 | ◎ 正解 | 破壊的変更リスク軽減 |

**結論**: Rust + Bevy は「AIバイブコーディング」に最適な選択だった。UnityはGUI操作が必要でAI完結できない。C++は型が弱くランタイムエラーになる。

### v1.0の定義

**「コンテンツ追加が作業になる状態」**

- 新機械追加 = game_specに20行 + モデル配置
- 新レシピ追加 = game_specに5行
- 新クエスト追加 = game_specに10行

→ Phase C（データ駆動化）が核心

### 追加したドキュメント

1. **PRD（製品要件定義書）**: `.claude/prd.md`
   - ターゲットユーザー、コア体験、成功指標
   - 判断に迷ったときのガイド

2. **定期リファクタリング命令**: CLAUDE.mdに追加
   - マイルストーンごとに構造チェック

3. **バイブコーディング限界実験の記録ルール**: CLAUDE.mdに追加
   - 破綻パターンの観測
   - 暴走検知ルール

### 現在の状態（実験記録）

| 項目 | 値 |
|------|-----|
| コード行数 | **22,747行**（最大ファイル: 875行）|
| テスト | **318件** 通過 |
| ファイル数 | 82ファイル |
| 破綻の兆候 | なし |
| 人間介入 | 方向性の判断のみ |

### 次のステップ

- **Phase C（データ駆動化）** に集中
- マイルストーンごとに実験記録を更新

---

## 2026-01-06: ディスク容量管理・タスク完了

### ディスク容量問題の解決

- **問題**: 放置されたworktree（4個×6.5GB = 26GB）でディスク容量不足
- **対応**:
  - 未マージブランチ2件をmasterにマージ後、全worktree削除
  - ディスク使用率: 54% → 26%（+25GB回復）

### 容量チェック機能追加

`scripts/parallel-run.sh` に安全機能を追加:

| コマンド | 説明 |
|----------|------|
| `check <数>` | N個のworktree用容量があるか事前確認 |
| `cleanup` | 放置worktreeを自動削除 |
| `start` | 開始前に自動容量チェック（不足なら中止）|

設定値:
- worktree 1つ: 7GB（安全マージン込み）
- 最低確保: 10GB

### 完了したタスク（並列実行）

1. **fix-furnace-input**: 製錬炉の搬入位置判定を`Furnace.position`を使用するよう修正
2. **tutorial-progress-bar**: チュートリアルの個数目標にプログレスバー追加
3. **hide-main-quest-tutorial**: チュートリアル中メインクエストUI非表示（前回からの継続）
4. **simplify-biome-hud**: バイオームHUD簡略化（前回からの継続）

### CLAUDE.md更新

- 容量チェック必須ルールを追記
- worktree放置禁止を明文化

---

## 2026-01-04: UI/UX改善・バグ修正

### 修正内容

#### 1. コンベア左曲がり時のアイテム移動方向バグ修正
- **問題**: コンベアのモデル位置は正しいが、左曲がり時にアイテムが右に曲がってしまう
- **原因**: コーナーコンベアで出力方向が常に`direction`（前方）になっていた
- **修正**:
  - `Conveyor`コンポーネントに`output_direction`フィールドを追加
  - `update_conveyor_shapes`でコーナーコンベアの正しい出力方向を計算
  - `conveyor_transfer`で`output_direction`を使用してアイテムを正しい方向に移動
- **関連ファイル**:
  - `src/components/machines.rs`
  - `src/systems/targeting/conveyor.rs`
  - `src/systems/conveyor.rs`
  - `src/systems/block_operations/placement.rs`
  - `src/systems/save_systems.rs`
  - `src/systems/command/handlers.rs`

#### 2. コンベア設置ゴーストの浮き問題修正
- **問題**: コンベアの設置プレビュー（ゴースト）が地面から浮いている
- **原因**: プレビュー位置が`pos.y + 0.5`（ブロック中心）になっていた
- **修正**: コンベア用のY座標を`pos.y`（地面レベル）に変更
- **関連ファイル**: `src/systems/targeting/highlight.rs`

#### 3. 機械にも設置ゴーストを表示
- **変更**: 機械（Miner, Furnace, Crusher）選択時に半透明青のプレビューキューブを表示
- **関連ファイル**: `src/systems/targeting/highlight.rs`

#### 4. 機械にも方向矢印を表示
- **変更**: 機械プレビューにも黄色い3D矢印を表示（Rキーで回転可能）
- **関連ファイル**: `src/systems/targeting/highlight.rs`

#### 5. 矢印の視認性改善
- **問題**: 矢印が細くて見にくい
- **修正**:
  - LineListから3D TriangleList（立体的なシャフト+ピラミッド型）に変更
  - 明るい黄色（`Color::srgb(1.0, 0.9, 0.0)`）で視認性向上
  - `create_3d_arrow_mesh()`関数を新規作成
- **関連ファイル**: `src/systems/targeting/highlight.rs`

#### 6. クエストUIをビジュアル改善
- **問題**: テキストベースでダサい
- **修正**:
  - プログレスバー付きの新UIに変更
  - 各アイテムの進捗を個別表示（バー+テキスト）
  - 完了状態に応じた色分け（青=進行中、黄緑=納品可能、緑=完了）
  - 日本語UI（「クエスト」「納品する」「報酬を受け取る」等）
  - ボーダーと背景のスタイリング改善
- **新規コンポーネント**:
  - `QuestProgressContainer`
  - `QuestProgressItem(usize)`
  - `QuestProgressBarBg(usize)`
  - `QuestProgressBarFill(usize)`
  - `QuestProgressText(usize)`
- **関連ファイル**:
  - `src/components/mod.rs`
  - `src/setup/ui/mod.rs`
  - `src/systems/quest.rs`

### テスト結果
- 全280件のテストがパス
- Clippy警告: 0件（許容範囲の警告のみ）

---

## 2026-01-09: ワールド最適化実装（Phase 1-4）

### 概要

チャンクデータのメモリ最適化を実装。Section分割、Palette圧縮、Occlusion Culling基盤、高さ拡張を完了。

### 実装内容

#### Phase 1: Section分割

| ファイル | 内容 |
|----------|------|
| `src/world/section.rs` | `ChunkSection` enum (Empty/Uniform/Paletted) |
| `src/constants.rs` | `SECTION_HEIGHT=16`, `SECTIONS_PER_CHUNK` 追加 |

**ChunkSection enum**:
- `Empty`: 完全に空気（~16バイト）
- `Uniform(ItemId)`: 全ブロック同一（~16バイト）
- `Paletted(Box<PalettedSection>)`: 混合ブロック

#### Phase 2: Palette圧縮

| ファイル | 内容 |
|----------|------|
| `src/world/palette.rs` | `PackedArray`, `PalettedSection` |

**PackedArray**:
- 可変ビット幅（1/2/4/8 bit）
- ビット圧縮配列

**PalettedSection**:
- パレット + 圧縮インデックス
- 動的ビット幅調整
- 自動コンパクション

#### Phase 3: Occlusion Culling基盤

| ファイル | 内容 |
|----------|------|
| `src/world/visibility.rs` | `SectionFaces`, `SectionVisibility`, `ChunkVisibility` |

**機能**:
- 6面の不透明度フラグ
- 隣接セクション遮蔽判定
- 将来の洞窟実装用インフラ

#### Phase 4: 高さ拡張

| 定数 | 変更前 | 変更後 |
|------|--------|--------|
| `CHUNK_HEIGHT` | 32 | 64 |
| `GROUND_LEVEL` | 7 | 32 |

### ベンチマーク結果

| VIEW_DISTANCE | チャンク数 | 最適化前 | 最適化後 | 削減率 |
|---------------|-----------|----------|----------|--------|
| 3 | 49 | 6,272 KB | **110 KB** | **98.2%** |
| 5 | 121 | 15,488 KB | **267 KB** | **98.3%** |
| 7 | 225 | 28,800 KB | **495 KB** | **98.3%** |
| 10 | 441 | 56,448 KB | **962 KB** | **98.3%** |

**セクション内訳（4セクション/チャンク）**:
- Empty: 25%（空気層）
- Uniform: 25%（地下の石層）
- Paletted: 50%（地表付近）

**単一チャンク詳細**:
- Section 0 (y=0..16): Paletted - 1,168 bytes
- Section 1 (y=16..32): Uniform - 16 bytes
- Section 2 (y=32..48): Paletted - 1,168 bytes
- Section 3 (y=48..64): Empty - 16 bytes
- 合計: **2,392 bytes**（最適化前131,072 bytes → **54倍削減**）

### 技術判断

| 判断 | 理由 |
|------|------|
| Palette圧縮をSection内部に統合 | 管理の簡素化、自動最適化 |
| Occlusion Cullingはインフラのみ | 洞窟なしで効果限定、将来用 |
| 高さ64に段階的拡張 | 128への移行準備、互換性維持 |
| `try_optimize()`で自動コンパクション | メモリリーク防止 |

### 学び

| 問題 | 教訓 |
|------|------|
| 借用チェッカー問題 | `opacity`を事前収集して回避 |
| テストがハードコード座標使用 | 定数`GROUND_LEVEL`を参照するよう修正 |
| ベンチマーク実行方法 | `examples/`に配置してリリースビルド |

### テスト結果

- 全テスト通過
- Clippy警告: 0件
- ベンチマーク: VIEW_DISTANCE=10で962KBのメモリ使用
