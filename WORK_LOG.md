# 作業ログ

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
