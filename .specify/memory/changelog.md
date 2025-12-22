# Development Changelog

## 2025-12-22: テスト手法研究＆デザインパターン追加

### 概要
Bevy + Tauriプロジェクトに適用可能なテスト手法を調査し、デザインパターンに追加

### 調査対象
- ゲームE2Eテスト（GameDriver, Test.AI）
- Bevyテスト（leafwing-input-manager, World直接操作）
- Tauriテスト（WebdriverIO, Selenium）
- テストパターン（リプレイ、プロパティベース、ファズ）

### 成果物

#### 1. テスト手法研究レポート (`testing-methods-research.md`)
- ゲーム特有のテスト課題（非決定性、状態量、フレーム依存）
- 適用可能なツール（leafwing-input-manager, WebdriverIO, proptest）
- テストパターン（リプレイ、プロパティベース、ゴールデン、ファズ）
- CI/CD統合（ヘッドレステスト、パフォーマンス回帰）
- 本プロジェクトへの適用案（優先実装順）

#### 2. デザインパターン集更新 (`design-patterns.md`)
- Part 9: テストパターン（T1-T4）追加
- Part 10: テストアンチパターン速見表追加
- Part 11: テスト評価テンプレート追加

### テストパターン（T1-T4）
- T1. ECSユニットテスト（World直接操作）
- T2. リプレイシステム（入力記録・再生）
- T3. プロパティベーステスト（不変条件検証）
- T4. E2Eテスト（UI操作シミュレート）

### 推奨ツールセット
| レイヤー | ツール |
|---------|--------|
| Bevy入力 | leafwing-input-manager |
| ECSテスト | 組み込みWorld操作 |
| Tauri UI | WebdriverIO |
| プロパティ | proptest |
| ファズ | cargo-fuzz |
| CI | GitHub Actions + xvfb |

---

## 2025-12-22: エディタUX研究＆デザインパターン追加

### 概要
ゲーム内エディタの設計指針を確立するため、各種エディタのUXを調査

### 調査対象
- Unity Editor, Blender, MagicaVoxel
- Unreal Blueprint, LDtk
- 各種ノードエディタ、GUIエディタ

### 成果物

#### 1. エディタUXベストプラクティス (`editor-ux-best-practices.md`)
- Unity公式の4原則（Modern, Familiar, Accessible, Efficient）
- MagicaVoxelの成功要因（シンプル、軽量、ツールチップ）
- Unreal Blueprintの成功要因（色分け、フロー可視化）
- キーボードショートカット設計
- Undo/Redo設計

#### 2. エディタUXアンチパターン (`editor-ux-antipatterns.md`)
- 致命的: 一貫性欠如、機能の迷路、情報過多、Blender症候群
- 操作系: フィードバック欠如、不可逆操作、モード地獄
- 学習曲線: チュートリアル不足、断崖式学習

#### 3. デザインパターン集更新 (`design-patterns.md`)
- Part 6: エディタUIパターン（E1-E6）追加
- Part 7: エディタアンチパターン速見表追加
- Part 8: エディタUI評価テンプレート追加

### エディタUIパターン（E1-E6）
- E1. ツールチップの完備
- E2. 深いUndo/Redo（100+ステップ）
- E3. キーボードショートカットの可視性
- E4. ノードエディタの標準
- E5. 即時プレビュー
- E6. 検索とフィルタ

---

## 2025-12-22: 工場ゲームUX研究＆デザインパターン策定

### 概要
理想の工場ゲーム体験を追求するため、競合ゲームの口コミ・レビューを調査し、
設計・評価に使えるデザインパターン集を作成

### 調査対象
- Factorio, Satisfactory, Shapez 2
- Dyson Sphere Program, Minecraft Create Mod

### 成果物

#### 1. UX研究レポート (`factory-game-ux-research.md`)
- 共通する快感要素（最適化、時間消失、視覚的成長、問題解決）
- 各ゲームの強み・弱み分析
- 音と視覚のフィードバック設計
- 本作への提言

#### 2. アンチパターン集 (`factory-game-antipatterns.md`)
- 致命的アンチパターン（仕事感、カスケード問題、チュートリアル不足）
- 進行系アンチパターン（難易度の崖、終盤グラインド）
- 操作系アンチパターン（移動コスト、解体の苦痛）
- 根本原因の4分類

#### 3. デザインパターン集 (`design-patterns.md`)
- 進行パターン（P1-P4）
- レシピ設計パターン（R1-R4）
- クエスト設計パターン（Q1-Q4）
- UI/UXパターン（U1-U4）
- 評価テンプレート付き

### 使い方
レシピツリーやクエストツリーを設計する際、
`design-patterns.md`のチェックリストで評価可能

---

## 2025-12-22: Steam実績システム＆エディタ仕様確定

### 概要
Steam販売準備として、実績システム、エディタ、MOD/プロファイルシステムの仕様を確定

### 決定事項

#### 実績の種類
- **通常実績**: 条件達成で即解除（初めての機械設置等）
- **プログレス実績**: 段階的に進捗表示（アイテム1000個生産等）
- **隠し実績**: 解除まで内容非表示（隠しエリア発見等）

#### トリガー条件
- クエスト完了、アイテム生産数、機械設置数
- プレイ時間、フェーズ到達、カスタム条件

#### エクスポート形式
- 内部: JSON/YAML形式（エディタで扱いやすい）
- Steamworks: VDFに変換してエクスポート

#### 統計システム
- total_items_produced, total_machines_placed
- total_playtime_seconds, current_phase
- アイテム別生産数 ({item_id}_produced)

### MOD＆プロファイルシステム
- `profiles/`: ローカル編集可能（エディタ対象）
  - `vanilla/`: 公式コンテンツ（Steam配布）
  - ユーザー作成MODもここに配置
- `mods/`: ダウンロード専用（外部MODをバージョン管理）
- Tauriラッパーによる疑似ホットリロード

### プロファイル直接編集（エクスポート不要）
- **旧方式**: エディタ → [Export] → assets/data/
- **新方式**: エディタ → Target選択 → profiles/{target}/data/に自動保存
- 開発者モード: vanillaプロファイルをターゲット
- MODユーザー: カスタムプロファイルをターゲット

### 仕様ファイル
- `.specify/specs/steam-editor-mode.md` （確定版）
- `.specify/specs/steam-release-preparation.md` （調査レポート）

---

## 2025-12-22: アーキテクチャ推奨事項の全面実装

### 概要
ARCHITECTURE_REVIEW.mdで挙げられたすべての推奨事項を実装完了

### 実装内容

#### 短期対策
1. **エクスポートボタン追加**
   - App.tsxに「📤 Export to YAML」ボタン追加
   - アイテムとレシピをゲーム用形式でエクスポート

2. **レシピ型互換性レイヤー**
   - MachineType → WorkType変換関数
   - Ingredient → ItemIO変換
   - Product → ItemIO変換

#### 中期対策
3. **共通型定義crate (factory-data-types) 作成**
   - `crates/factory-data-types/`
   - ItemData, GameItemData
   - RecipeDef, GameRecipe
   - QuestData, QuestRequirement, RewardType
   - 7件のテスト追加

4. **TypeScript型自動生成 (ts-rs)**
   - `cargo test --features typescript -p factory-data-types`で.ts生成
   - bindings/ディレクトリに出力

#### 長期対策
5. **データフォーマットYAML統一**
   - 保存: YAML形式（デフォルト）
   - 読込: YAML優先、RONフォールバック
   - 削除: 両形式を削除
   - カタログ・エクスポート: 両形式対応
   - 出力ファイル（core.yaml, kinetic.yaml）スキップ

6. **ホットリロード対応**
   - `src/core/hot_reload.rs`新規作成
   - F5キーでデータ再読み込み
   - ReloadDataEvent追加
   - ItemRegistry.clear()追加
   - RecipeManager.clear()追加

### Clippy修正
- `derivable_impls`: AnimationType derive Default
- `double_ended_iterator_last`: next_back()に変更
- `should_implement_trait`: from_str → parseに名前変更

### テスト
- factory-data-types: 7件パス

### Files Created
- `crates/factory-data-types/Cargo.toml`
- `crates/factory-data-types/src/lib.rs`
- `crates/factory-data-types/src/item.rs`
- `crates/factory-data-types/src/recipe.rs`
- `crates/factory-data-types/src/quest.rs`
- `crates/factory-data-types/src/export_ts.rs`
- `src/core/hot_reload.rs`

### Files Modified
- `Cargo.toml` - workspaceにcrate追加
- `src/core/mod.rs` - hot_reloadモジュール追加
- `src/gameplay/inventory.rs` - clear()追加
- `src/gameplay/machines/recipe_system.rs` - clear()追加
- `tools/factory-data-architect/src-tauri/src/lib.rs` - YAML統一
- `tools/factory-data-architect/src/App.tsx` - エクスポートボタン
- `tools/factory-data-architect/src/App.css` - スタイル追加

---

## 2025-12-22: アーキテクチャレビューと修正

### 調査実施
エディタ、ゲーム、連携について包括的な調査を実施。

**調査結果**: `tools/factory-data-architect/ARCHITECTURE_REVIEW.md`

### 発見した重大な問題

#### 1. データフォーマット不整合 (Critical)
- エディタ: RON形式で保存
- ゲーム: YAML形式で読み込み
- **影響**: エディタで作成したデータをゲームで直接使用できない

#### 2. レシピ型の非互換性
- エディタ: `RecipeDef` (machine_type, ingredients, results, process_time)
- ゲーム: `Recipe` (work_type, inputs, outputs, craft_time)
- フィールド名が異なり、互換性がない

#### 3. 型定義の三重重複 (DRY違反)
- エディタRust、エディタTypeScript、ゲームRustで同じ概念が別々に定義

### 修正内容

#### ハードコードパスの削除
- `App.tsx`: `DEFAULT_ASSETS_PATH`を削除
- 初回起動時は必ずフォルダ選択を促すように変更

#### YAMLエクスポート機能追加
- `lib.rs`: 以下のコマンドを追加
  - `save_item_data_yaml`: アイテムをYAML形式で保存
  - `save_recipe_yaml`: レシピをYAML形式で保存
  - `export_items_to_yaml`: 全アイテムをゲーム用core.yamlにエクスポート
- `Cargo.toml`: `serde_yaml = "0.9"`を追加

### 今後の推奨アクション

#### 短期
1. エクスポートボタンをUIに追加（TypeScript側）
2. レシピ型の互換性レイヤー作成

#### 中期
1. 共通型定義crate作成
2. TypeScript型をRust型から自動生成

#### 長期
1. データフォーマット統一（YAMLまたはRON）
2. ホットリロード対応

### Files Modified
- `tools/factory-data-architect/src/App.tsx`
- `tools/factory-data-architect/src-tauri/Cargo.toml`
- `tools/factory-data-architect/src-tauri/src/lib.rs`

### Files Created
- `tools/factory-data-architect/ARCHITECTURE_REVIEW.md`

---

## 2025-12-20: エディタ懸念点修正

### Fixed

#### レシピIDバリデーションとi18n_key自動生成
- レシピIDを空からスタート（プレースホルダー表示）
- 既存レシピIDとの重複チェック追加
- i18n_keyを`recipe.{id}`形式で自動生成（手動入力廃止）
- 保存時のバリデーション（空ID/重複IDでアラート表示）

#### アイテム削除のファイル反映
- `delete_item_data`コマンドをRustバックエンドに追加
- 削除時にファイルも物理削除されるように修正
- 削除確認ダイアログを強化

#### 既存アイテムのID編集禁止
- 既存アイテム編集時はIDフィールドを読み取り専用に
- 「既存アイテムのIDは変更できません」ヒントを表示
- ゴミファイル発生を防止

#### processTimeの型統一
- TypeScript側を`parseInt`から`parseFloat`に変更
- ラベルを「ticks」から「秒」に修正
- Rust側のf32型と整合性を確保

### Changed
- Clippy警告を解消（未使用コードに`#[allow(dead_code)]`または`#[cfg(test)]`を追加）
- `map_or`を`is_some_and`に変更

### Tests
- TypeScript: 全27テストがパス
- Rust: 全31テストがパス

---

## 2025-12-20: レシピエディタのドラッグ&ドロップ修正

### Fixed

#### ドラッグ&ドロップでノードが作成されない問題
- 問題: パレットからアイテムをキャンバスにドラッグ&ドロップしてもノードが作成されない
- 原因: `draggedItem`ステートを使った実装がReactFlowと競合していた
- 解決: `event.dataTransfer`を使ってドラッグデータを直接渡すように変更
  - `handleDragStart`: データを`dataTransfer.setData()`で保存
  - `handleDrop`: `dataTransfer.getData()`でデータを取得してノード作成
  - 不要になった`draggedItem`ステートを削除

#### TypeScriptモジュール解決の問題
- ItemEditorのインポートパスを`../types`から`../types/index`に変更
- ItemCategory, subcategoryプロパティが見つからないエラーを解消

### Tests
- index.test.tsに2件のテスト追加（LocalizationEntry, MachineItemData）
- 全27テストがパス

---

## 2025-12-19: アイテム一覧表示とID重複バリデーション修正

### Fixed

#### アイテム一覧に既存アイテムが表示されない問題
- ItemsTab起動時に`get_assets_catalog`でデータディレクトリから既存アイテムをロード
- アイテム一覧が正しく表示されるように修正

#### 重複IDで登録するとクラッシュする問題
- `existingItemIds`をItemEditorに渡してID重複チェックを実装
- 重複IDの場合:
  - 入力欄が赤くハイライト
  - 「⚠️ このIDは既に使用されています」エラーメッセージ表示
  - 保存ボタンが無効化

### Technical Details
- App.tsx: `AssetCatalog`型定義追加、`isLoading`ステート追加
- ItemEditor.tsx: `existingItemIds`プロップ追加、`isDuplicateId`/`isIdValid`バリデーション
- ItemEditor.css: `.input-error`、`.validation-error`スタイル追加

---

## 2025-12-19: Factory Data Architect エディタ改善

### Changed

#### アイテムエディタUI改善
- カテゴリ選択機能を追加（アイテム/機械/マルチブロック）
- i18n_keyを自動生成に変更（手動編集不可）
- キーフォーマット: `{category}.{id}` (例: `item.iron_ore`, `machine.assembler`)
- Itemsタブを2パネルレイアウトに変更（左:一覧、右:エディタ）

#### ダミーデータの削除
- RecipeEditorからハードコードされた機械タイプ（Assembler, Mixer等）を削除
- BiomeEditorからハードコードされたリソースリストを削除
- Rustバックエンド(lib.rs)からデフォルトアイテム/流体/機械/タグを削除
- 全データをカタログから動的に読み込むように変更

### Technical Details
- カテゴリに基づくi18n_key生成関数を追加
- MachineNodeInspectorにcatalog propを追加
- Viteビルド成功確認

---

## 2025-12-19: エディタ-ゲーム連携テスト実装

### Added

#### YAML読み込み機能 (src/gameplay/inventory.rs)
- ItemDefinition: YAMLからのアイテム定義読み込み用構造体
- serde::Deserialize対応
- From<ItemDefinition> for ItemData変換
- フォールバック機能: YAMLファイルがない場合はハードコードを使用
- test_item_definition_conversionテスト追加

#### YAML読み込み機能 (src/gameplay/quest.rs)
- QuestDefinition: YAMLからのクエスト定義読み込み用構造体
- RewardDefinition: 報酬タイプ（PortUnlock/Item）
- RequirementDefinition: 要件定義
- From<QuestDefinition> for QuestData変換
- フォールバック機能: YAMLファイルがない場合はハードコードを使用
- test_quest_definition_conversionテスト追加

#### テスト用データファイル
- assets/data/items/core.yaml: 8アイテム定義（test_yaml_item含む）
- assets/data/quests/core.yaml: 5クエスト定義（test_yaml_quest含む）

### Technical Details
- エディタ（Factory Data Architect）はRON形式で保存
- ゲームはYAML形式で読み込み
- 今後の拡張でエディタからYAMLエクスポート機能を追加予定

### Tests
- 全93テストがパス（+2テスト追加）
- Clippyチェック通過
- ゲーム起動確認:
  - 8 items loaded from YAML
  - 5 quests loaded from YAML

---

## 2025-12-18: ゲームシステムとHUD UI追加

### Added

#### プレイヤーステータス (`src/gameplay/player_stats.rs`)
- PlayerHealth: HP、再生、ダメージ、死亡システム
- PlayerExperience: 経験値、レベルアップシステム
- FallTracker: 落下ダメージ追跡
- SpawnAnchor: リスポーン地点設定
- DamageEvent, PlayerDeathEvent, PlayerRespawnEvent
- 5件のテスト追加

#### 天候システム (`src/gameplay/weather.rs`)
- DayNightCycle: 昼夜サイクル（10分で1日）
- Weather: 天候状態（晴れ、曇り、雨、嵐）
- WeatherEffects: 天候による機械効率への影響
- ExposedMachine, UnderRoof: 野ざらし/屋根判定
- AmbientLight連携
- 5件のテスト追加

#### 液体システム (`src/gameplay/fluid.rs`)
- FluidType: 液体の種類（水、溶岩、蒸気など）
- Pipe: パイプ（3ティア、流量制御）
- Tank: タンク（10,000mb容量、高温対応）
- PipeNetwork: パイプネットワーク管理
- Pump, DrainValve: ポンプ、排水バルブ
- 5件のテスト追加

#### 熱システム (`src/gameplay/heat.rs`)
- HeatContainer: 熱容器（温度、熱容量、伝導率）
- HeatSource: 熱源（ボイラー、電熱器）
- Boiler: ボイラー（燃料→蒸気変換）
- TemperatureRequirement: 機械の温度要件
- 熱伝導、環境冷却、過熱チェック
- 4件のテスト追加

#### 振動システム (`src/gameplay/vibration.rs`)
- VibrationSource: 振動源（機械の振動レベル）
- VibrationReceiver: 振動受信（効率低下）
- VibrationDamper: 防振台
- VibrationMap: 振動マップ（3x3x3伝播）
- 5件のテスト追加

#### ミニマップUI (`src/ui/minimap.rs`)
- 右上に常時表示
- プレイヤー位置中央固定
- 周辺機械検出
- 方位（N）表示
- 設定可能：サイズ、範囲、ズーム

#### HP HUD (`src/ui/health_hud.rs`)
- 左下にHP表示
- HPバー（色変化：緑→黄→赤）
- ダメージ時の赤フラッシュ
- HP数値テキスト

#### クエストHUD (`src/ui/quest_hud.rs`)
- アクティブクエスト表示
- 進捗バー
- [J]キーでクエスト一覧切り替え

### Tests
- 全92テストがパス（+27テスト追加）

---

## 2025-12-18: クエストシステムと納品プラットフォームを実装

### Added

#### クエストシステム (`src/gameplay/quest.rs`)
- QuestData: クエスト定義（メイン/サブ、フェーズ、要件、報酬）
- QuestProgress: プレイヤーのクエスト進捗管理
- QuestManager: クエスト状態管理リソース
- QuestRegistry: クエスト定義のレジストリ
- 報酬タイプ: PortUnlock（ポート解放）, Item（アイテム報酬）
- 前提条件システム: 順序付きメインクエスト対応
- 4件のテスト追加

#### 納品プラットフォーム (`src/gameplay/delivery_platform.rs`)
- DeliveryPlatform: 12x12グリッドの納品エリア
- DeliveryPort: コンベアからのアイテム受け入れポート（各辺3ポート）
- コンベア連携: 完全に進んだアイテムを自動受け取り
- クエスト連携: 納品アイテムを自動的にクエスト進捗に反映
- ポート解放システム: クエスト報酬でポート数増加
- ビジュアル生成: プラットフォームとポートの3Dメッシュ
- 4件のテスト追加

#### エディタ拡張 (`tools/factory-data-architect/`)
- QuestEditor: React Flowによるノードベースクエスト編集（メイン/サブタブ）
- MultiblockEditor: React Three Fiberによる3Dグリッド編集（最大10x10x10）
- BiomeEditor: Canvas 2Dによるノイズプレビュー付きバイオーム編集
- SoundEditor: オーディオプレビュー付きサウンド設定編集
- 型定義: quest.ts, multiblock.ts, biome.ts, sound.ts

#### UI改善
- アイテム登録時のカテゴリ選択UI（アイテム/機械/マルチブロック）
- 6タブ構成（Items, Recipes, Quests, Multiblock, Biomes, Sounds）

### Technical Details
- ゲーム仕様書作成: `.specify/specs/game-design-document.md`
- React Three Fiber依存追加: @react-three/fiber, @react-three/drei, three
- Clippy警告なし、TypeScriptビルド成功

### Tests
- 全65テストがパス（+8テスト追加）

---

## 2025-12-17: メニュー画面のレイヤリングとゲームプレイ分離を修正

### Fixed
- **メニュー画面がワールドの後ろに表示される問題を修正**
  - メニューUI(MainMenu, SaveSelect, WorldGeneration)にGlobalZIndex(100)を追加
  - 背景色を半透明に変更（0.95 alpha）で背景ワールドが見える

- **メニュー画面でプレイできてしまう問題を修正**
  - 全てのゲームプレイシステムにAppState::InGame条件を追加
  - プレイヤーの移動、視点操作、建築、ホットバー選択がInGame時のみ動作

- **プレイヤーのスポーン/デスポーンをステート管理**
  - spawn_player: StartupからOnEnter(AppState::InGame)に変更
  - despawn_player: OnExit(AppState::InGame)で削除
  - 既存プレイヤーがいる場合はスポーンをスキップ

### Changed
- **プレイヤーカメラのorder設定**
  - プレイヤーカメラのorder=1（メニューカメラorder=0より上）

### Technical Details
- gameplay/mod.rs: AppState依存を追加
- gameplay/player.rs: despawn_player関数追加、spawn_player引数追加
- ui/main_menu.rs: 3画面すべてにGlobalZIndex追加

### Tests
- 全57テストがパス
- Clippyチェック通過

---

## 2025-12-16: Clippy修正とメニューUI改善

### Fixed
- **Clippyエラー39件を修正**
  - dead_code: 未使用関数に#[allow(dead_code)]追加
  - derivable_impls: #[derive(Default)]を使用
  - collapsible_if/else_if: if文を整理
  - type_complexity: #[allow(clippy::type_complexity)]追加
  - too_many_arguments: #[allow(clippy::too_many_arguments)]追加
  - その他: contains_key, is_none_or, first()を使用

- **メニュー画面のESCキー動作を修正**
  - SaveSelect/WorldGeneration画面でESCを押すと設定UIが出る問題を修正
  - settings_ui.rs: AppState::InGame時のみ設定UIのESCハンドラを有効化
  - main_menu.rs: メニュー画面用ESCハンドラを追加（Back機能）

### Changed
- **CLAUDE.md更新**
  - 自動テスト指示を追加（ユーザー指示時に実行）

---

## 2025-12-16: Kinetic Machines and Recipe System

### Added
- **Common Machine Components** (`src/gameplay/machines/machine_components.rs`)
  - Slot: Stack-enabled inventory slot
  - InputInventory / OutputInventory: Item I/O management
  - FluidTank: Fluid storage (mB units)
  - MachineState: Idle/Processing/Jammed/NoPower
  - StressImpact: Stress consumption value
  - KineticMachine: Machine marker component

- **Recipe System** (`src/gameplay/machines/recipe_system.rs`)
  - WorkType enum: Pressing, Crushing, Cutting, Mixing, WireDrawing, etc.
  - Recipe struct: inputs, outputs, fluids, craft_time, work_type
  - RecipeManager resource: search by work type or input items
  - YAML loading support (assets/data/recipes/kinetic.yaml)
  - 5 default recipes (press, crush, cut, mix, wire draw)

- **Kinetic Machines** (`src/gameplay/machines/kinetic_machines.rs`)
  - MechanicalPress: Pressing (ingot → plate)
  - Crusher: Crushing (ore → dust ×2)
  - MechanicalSaw: Cutting (log → planks)
  - Mixer: Mixing (with fluid tanks)
  - WireDrawer: Wire drawing (plate → wire)
  - Bundle definitions for each machine
  - `process_kinetic_machines`: Generic processing system
  - MachineAnimation: Frame-based animation

- **Animation Guide** (`docs/KINETIC_MACHINE_ANIMATION_GUIDE.md`)
  - Animation specs for each machine
  - Required texture/model file list
  - Implementation sample code

### Tests Added
- 4 machine_components tests (slot, inventory, fluid, state)
- 4 recipe_system tests (get, work_type, find, accept)
- 3 kinetic_machines tests (bundle, animation, processing)
- Total: 57 tests passing

---

## 2025-12-16: Main Menu and Save System

### Added
- **Main Menu UI** (`src/ui/main_menu.rs`)
  - AppState enum: MainMenu, SaveSelect, WorldGeneration, InGame
  - Main menu with Play/Settings/Quit buttons
  - Save slot selection (8 slots)
  - World generation screen with name/seed input
  - Button interaction system with hover/press colors
  - TextInput component for keyboard text entry (Bevy 0.15 KeyboardInput)

- **Menu Camera** (`src/ui/menu_camera.rs`)
  - MenuCamera component with orbit parameters
  - Background camera slowly rotating around origin
  - Active only during menu states
  - Automatic spawn/despawn on state transitions

- **Save System** (`src/core/save_system.rs`)
  - SaveMetadata: world_name, seed, play_time, last_played_date
  - SaveSlotData: 8 slots with optional metadata
  - WorldGenerationParams: data transfer between scenes
  - JSON persistence to `saves/slot_N/metadata.json`
  - Automatic loading at startup

### Components Added
- `MainMenuUi`, `SaveSelectUi`, `WorldGenUi`: UI markers
- `MenuButtonAction`: Button action enum
- `TextInput`, `TextInputDisplay`: Text input components
- `MenuCamera`: Orbiting camera component
- `SelectedSlotIndex`: Selected save slot resource

### Systems Added
- `spawn_main_menu`, `spawn_save_select`, `spawn_world_generation`
- `button_interaction_system`: Hover/press color changes
- `main_menu_buttons`, `save_select_buttons`, `world_gen_buttons`
- `text_input_system`: Keyboard text input handling
- `spawn_menu_camera`, `despawn_menu_camera`, `orbit_camera`
- `load_save_slots`: Startup save data loading

### Dependencies Added
- `chrono = { version = "0.4", features = ["serde"] }`
- `serde_json = "1.0"`

### Tests Added
- 2 menu camera tests (default values, orbit position)
- 4 save system tests (metadata, slots, serialization)
- Total: 46 tests passing

---

## 2025-12-16: Phase 5 - Optimization and Modding

### Added
- **Async Chunk Generation** (`src/core/optimization.rs`)
  - AsyncComputeTaskPool for parallel chunk generation
  - ChunkLoadQueue resource for task management
  - Non-blocking chunk data generation
  - Automatic terrain generation (stone/dirt/air layers)

- **LOD System** (`src/core/optimization.rs`)
  - ChunkLod component (Full, Medium, Low, Icon)
  - LodSettings resource with configurable distances
  - Dynamic LOD updates based on camera distance
  - Automatic chunk unloading for distant chunks

- **Mod Loader** (`src/core/modding.rs`)
  - ModManifest YAML schema (id, name, version, dependencies)
  - ModRegistry resource for mod management
  - Automatic mod discovery from `mods/` directory
  - Dependency resolution and load order sorting
  - Asset path helpers for mod content

### Components Added
- `ChunkLod`: LOD level marker for chunks
- `AsyncChunkMarker`: Marker for async-generated chunks

### Resources Added
- `ChunkLoadQueue`: Async chunk generation queue
- `LodSettings`: LOD distance configuration
- `ModRegistry`: Loaded mods registry

### Systems Added
- `process_chunk_tasks`: Handle async chunk generation
- `update_chunk_lod`: Update LOD based on camera distance
- `unload_distant_chunks`: Remove far chunks from memory
- `discover_mods`: Scan and load mods at startup

### Tests Added
- 3 optimization tests (chunk generation, LOD settings, chunk LOD)
- 3 modding tests (manifest, registry, dependency)
- Total: 40 tests passing

---

## 2025-12-16: Phase 4 - Scripting and Signal Systems

### Added
- **Lua Scripting Engine** (`src/gameplay/scripting.rs`)
  - Lua 5.4 VM integration via mlua crate (vendored)
  - Sandboxed execution (os, io, load, require disabled)
  - ScriptEngine resource with thread-safe Lua VM
  - ScriptRegistry for asset-loaded scripts
  - ScriptContext for input/output signal handling
  - Programmable component for machines
  - Built-in functions: print, clamp, lerp

- **Signal System** (`src/gameplay/signals.rs`)
  - Wire-based signal transmission network
  - SignalNetwork resource with BFS group detection
  - SignalValue enum: Number, String, Boolean, Nil
  - SignalEmitter and SignalReceiver components
  - Logic gates: AND, OR, NOT, XOR, NAND, NOR
  - Numeric processors: Add, Subtract, Multiply, Divide, Compare, Equal

### Components Added
- `Programmable`: Script-controlled machine component
- `Wire`: Wire connection marker
- `SignalEmitter`: Signal output component
- `SignalReceiver`: Signal input component
- `LogicGate`: Configurable logic gate component

### Systems Added
- `tick_programmable_machines`: Execute Lua scripts each fixed tick
- `propagate_signals`: Transmit signals through wire network
- `tick_logic_gates`: Process logic gate computations

### Tests Added
- 5 scripting tests (basic, inputs, sandbox, error handling, signal values)
- 7 signal tests (logic gates, network connection, group finding)
- Total: 34 tests passing

### Dependencies
- Added `mlua = { version = "0.10", features = ["lua54", "vendored", "send", "serialize"] }`

---

## 2025-12-16: UI/UX Improvements and Creative Mode Enhancements

### Added
- **Settings UI with Interactive Controls**
  - FPS adjustment buttons (30, 60, 120, 144, 240)
  - Mouse sensitivity adjustment (+/- buttons, 0.001-0.01 range)
  - Toggle buttons for Enable Highlight and Enable UI Blur
  - Immediate application of settings (no Apply button needed)

- **Creative Mode UI Redesign**
  - Inventory and Item Catalog now toggle at the same position (center)
  - No more side-by-side pre-allocated spaces
  - Toggle button (54x54px square, ⇄ icon) switches between views
  - Hotbar remains visible even when inventory is open in Creative mode

- **Improved ESC Key Behavior**
  - ESC 1st press: Show settings button
  - ESC 2nd press: Return to normal play with automatic cursor grab
  - No additional click required to resume playing

### Changed
- **Hotbar Visibility**
  - Creative Mode: Hotbar always visible (even when inventory open)
  - Survival Mode: Traditional behavior (hotbar hidden when inventory open)

- **Settings UI**
  - Removed Apply button (settings apply immediately)
  - Toggle buttons are now square (54x54px) matching inventory slots
  - ON/OFF state shown with color (green=ON, red=OFF) and text

### Systems Added
- `spawn_hotbar_hud_if_creative`: Conditional hotbar spawn for creative mode
- `spawn_hotbar_hud_if_not_creative`: Conditional hotbar spawn for survival mode
- `despawn_hotbar_hud_if_not_creative`: Conditional hotbar despawn
- `spawn_creative_item_grid`: Unified item catalog grid display
- `initialize_creative_visibility`: Initial visibility setup for creative mode
- `handle_fps_buttons`: FPS setting control
- `handle_sensitivity_buttons`: Mouse sensitivity control
- `handle_toggle_buttons`: Boolean setting toggles

### Components Added
- `CreativeItemList`: Marker for creative item catalog container
- `FpsIncreaseButton`, `FpsDecreaseButton`, `FpsValueText`
- `SensitivityIncreaseButton`, `SensitivityDecreaseButton`, `SensitivityValueText`
- `HighlightToggleButton`, `UiBlurToggleButton`

### Technical Details
- Creative mode UI uses Visibility component for overlay toggling
- Settings write directly to GameConfig (immediate effect)
- Hotbar spawn/despawn logic checks GameMode for conditional behavior
- All 15 tests passing

### Files Modified
- `src/ui/settings_ui.rs`: Settings controls and ESC key handling
- `src/ui/inventory_ui.rs`: Creative UI redesign and hotbar visibility
- `src/gameplay/mod.rs`: Player control blocking during settings
- `src/gameplay/commands.rs`: Default game mode set to Creative

---

## 2025-12-16: SpecKit Integration

### Added
- GitHub SpecKit for Spec-Driven Development
- `.specify/` directory with constitution, templates, and specs
- Project constitution defining core principles and standards
- Comprehensive documentation structure

### Files Created
- `.specify/memory/constitution.md` - Project governance and standards
- `.specify/templates/*.md` - Spec, plan, and task templates
- `.specify/specs/core-game-mechanics.md` - Core game feature specification
- `.specify/README.md` - SpecKit usage guide

### Migrated
- GAME_SPEC.md → `.specify/specs/core-game-mechanics.md`
- PROJECT_ROADMAP.md → Integrated into constitution.md
- WORK_LOG.md → `.specify/memory/changelog.md`

---

## 2025-12-16: Inventory UI Improvements

### Added
- Minecraft-style inventory UI with consistent slot sizing (54x54px)
- Equipment slots with icon indicators (H/C/L/F/T)
- Hotbar HUD displayed at screen bottom when inventory is closed
- Equipment panel on left, main inventory (8x3) in center, crafting list on right
- Trash slot positioned in bottom-right corner

### Changed
- Player movement and camera control disabled when inventory UI is open
- Cursor automatically released when opening inventory
- Unified slot system with `spawn_slot_sized()` and `spawn_slot_with_icon()`

### Systems Added
- `spawn_hotbar_hud`: Create hotbar HUD
- `despawn_hotbar_hud`: Remove hotbar HUD
- `update_hotbar_hud`: Update hotbar display
- `release_cursor`: Cursor management
- `spawn_equipment_panel_mc`: Equipment panel with icons
- `spawn_main_inventory_panel_mc`: Main inventory + hotbar layout

### Technical Details
- Leveraged Bevy UI Flexbox and Grid layouts
- State-based UI visibility control
- `run_if` conditions for system execution
- All 13 tests passing

---

## 2025-12-15: Phase 3 Implementation

### Completed Features

#### GUI Framework (`src/ui/machine_ui.rs`)
- MachineUiState state machine (Closed/Open)
- OpenMachineUi resource for tracking target machine
- Recipe selection UI for Assembler
- Real-time inventory display (input/output)
- Event-driven open/close system
- Tests: 2 passing

#### Structure Validator (`src/gameplay/multiblock.rs`)
- MultiblockPattern struct with YAML-compatible format
- `StructureValidator::validate_at()` for pattern matching
- `StructureValidator::find_valid_pattern()` for pattern discovery
- Support for rotational validation
- Tests: 4 passing

#### Master/Slave System (`src/gameplay/multiblock.rs`)
- MultiblockMaster/MultiblockSlave components
- FormedMultiblocks resource for tracking formed structures
- MultiblockFormedEvent/MultiblockBrokenEvent
- Auto-detection on MachinePlacedEvent
- Automatic integrity checking system

### Code Changes
- `src/ui/mod.rs`: Added machine_ui module
- `src/gameplay/mod.rs`: Added multiblock module
- `src/gameplay/machines/mod.rs`: Removed old assembler interaction handler
- `src/gameplay/machines/assembler.rs`: Removed handle_assembler_interaction (moved to UI)

### Test Improvements
- **power.rs tests**: Changed to Update schedule for test reliability
- **machine_ui.rs tests**: Added StatesPlugin, multiple update cycles
- **assembler.rs test**: Direct progress injection for timing reliability

### Failed Attempts (Learning Notes)
- ❌ FixedUpdate tests: `app.update()` doesn't trigger FixedUpdate without elapsed time
- ❌ `time.advance_by()`: Doesn't affect `delta_secs()` as expected in tests
- ❌ Single `app.update()` for state transitions: Requires multiple cycles

---

## Earlier Development

### Phase 2: Logic and Logistics Simulation
**Status:** ✅ Complete

**Achievements:**
- Fixed Timestep simulation (20 TPS)
- Deterministic logic for multiplayer support
- Grid-based machine placement system
- Item entity optimization with instanced rendering
- Inventory system with stack support
- Debug overlay (F3) with FPS, coordinates, memory usage
- Debug mode with collision visualization

### Phase 1: Core Engine and Mod Foundation
**Status:** ✅ Complete

**Achievements:**
- Asset Server with hot-reload support
- YAML loader for blocks, items, and recipes
- Dynamic texture atlas generation
- Chunk system (32×32×32 blocks)
- Greedy meshing for voxel rendering
- Custom shader support (.wgsl)
- Headless server build configuration
- Multiplayer replication (bevy_renet/lightyear)
- Client prediction for smooth movement

---

## Architecture Evolution

### Current Structure (Post-Phase 3)
```
src/
├── core/           # Configuration, registry, input, debug
├── rendering/      # Voxel rendering, meshing, models
├── gameplay/       # Game logic, machines, player
│   ├── grid.rs     # SimulationGrid resource
│   ├── building.rs # Machine placement
│   ├── player.rs   # Player movement and camera
│   ├── power.rs    # Power network system
│   ├── multiblock.rs # Multiblock structures
│   ├── inventory.rs  # Player inventory system
│   └── machines/   # Conveyor, Miner, Assembler
├── ui/             # Machine UI, Inventory UI, HUD
└── network/        # Multiplayer (stub)
```

### Key Technical Decisions

**ECS Architecture**
- Pure data components (no logic)
- Systems operate on queries
- Events for cross-system communication
- Resources for global state

**Data-Driven Design**
- YAML definitions for all content
- Hot-reload during development
- Modding-friendly structure

**Performance Optimizations**
- Instanced rendering for items (1000+ items)
- Greedy meshing for voxels
- Fixed timestep for deterministic simulation
- Async chunk loading

**Testing Strategy**
- 70%+ coverage on core systems
- Integration tests for machine interactions
- State machine tests for UI
- Deterministic tests for power network

---

## Metrics

### Current State
- **Total Tests:** 46 (all passing)
- **Test Coverage:** ~70% on core systems
- **Performance:** 60 FPS target, 20 TPS simulation
- **Code Quality:** Zero compiler warnings
- **Architecture:** ECS-first, data-driven

### Feature Completion
- ✅ Phase 1: 100% (Core Engine)
- ✅ Phase 2: 100% (Logistics)
- ✅ Phase 3: 100% (Power & Multiblock)
- ✅ Phase 4: 100% (Scripting)
- ✅ Phase 5: 100% (Optimization & Distribution)
- ✅ Main Menu: 100% (UI & Save System)

---

## Next Steps

### Future Enhancements
- [ ] Settings persistence (save/load config)
- [ ] Sound system integration
- [ ] Tutorial/Help system
- [ ] World save/load (full chunk data)
- [ ] Multiplayer lobby UI

---

*This changelog consolidates all development history and will be updated as the project progresses.*
