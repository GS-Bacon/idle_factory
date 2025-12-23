# Development Changelog

最新の開発履歴。詳細は git log 参照。

---

## 2025-12-23

### インタラクションテストシナリオを追加

**新機能**

- `interaction_test`: 全操作パターンのE2Eテスト（10フェーズ）
- F8キー: インタラクションテスト実行

**テストフェーズ**

| Phase | 内容 | 検証項目 |
|-------|------|----------|
| 1 | メニュー遷移 | MainMenu↔SaveSelect↔WorldGen |
| 2 | 移動操作 | WASD, Space, Shift |
| 3 | ホットバー | 1-9キー選択 |
| 4 | マウス操作 | 左/右クリック、ホールド |
| 5 | インベントリ | E開閉、ソート |
| 6 | ポーズメニュー | ESC、各ボタン |
| 7 | コンテナ | 右クリック開閉 |
| 8 | クイックアクセス | J, F3 |
| 9 | 複合操作 | 移動+ジャンプ、斜め移動 |
| 10 | 終了 | MainMenuに戻る |

**テスト結果**: 11/11 成功 ✅

### E2Eテストシステムを実装・トークン消費最適化

**新機能**

- `src/core/e2e_test.rs`: 自動テスト・スクリーンショット撮影システム
- F9キー: 手動スクリーンショット撮影
- F10キー: UIテストシナリオ実行
- F11キー: フルテストシナリオ実行（全画面遷移）
- F12キー: UIダンプ（テキストベース）
- `--e2e-test`: コマンドライン引数で自動テスト開始

**トークン消費最適化**

| 出力ファイル | トークン消費 | 用途 |
|-------------|-------------|------|
| `test_report.txt` | 極小（1-2KB） | Pass/Fail結果のみ |
| `*_ui_dump.txt` | 小（5-10KB） | UI構造のテキストダンプ |
| `*.png` | 大（画像） | 視覚確認（必要時のみ） |

**推奨ワークフロー**
1. `test_report.txt` を読んで検証結果を確認
2. 失敗した検証がある場合のみ、該当のUIダンプを確認
3. 視覚的な問題の場合のみスクリーンショットを確認

**テストステップ**

- `DumpUi(name)`: UIツリーをテキストでダンプ
- `VerifyElement(UiVerification)`: UI要素の自動検証
- `SaveReport`: 検証結果をレポートファイルに保存
- `ClearReport`: レポートをクリア

**UiVerification構造体**
```rust
UiVerification {
    name: String,           // 検証名
    component_name: Option<String>,  // コンポーネント名で検索
    text_contains: Option<String>,   // テキスト内容で検索
    min_count: Option<usize>,        // 最小要素数
    max_count: Option<usize>,        // 最大要素数
}
```

**検証結果** ✅

| UI | 状態 | 確認項目 |
|----|------|----------|
| メインメニュー | ✅ | Play/Settingsボタン存在 |
| セーブ選択 | ✅ | Select World/Backボタン存在 |
| ワールド生成 | ✅ | Createボタン存在 |
| インゲーム | ✅ | HP表示存在 |
| インベントリ | ✅ | Sort/Trashボタン存在 |
| ポーズメニュー | ✅ | Resume/MainMenuボタン存在 |

**技術詳細**

- `TestScenarioBuilder`: カスタムシナリオ作成用ビルダー（verify_text等追加）
- `SimulateInputEvent`: キー・マウス入力シミュレーション
- `TakeScreenshotEvent`: スクリーンショット撮影イベント
- `DumpUiEvent`: UIダンプイベント
- `VerifyUiEvent`: UI検証イベント
- `TestReport`: 検証結果を蓄積しレポート生成
- スクリーンショット・ダンプは `screenshots/` に保存

### ワールド作成時のゲームモード選択を追加

**新機能**

- ワールド作成画面にSurvival/Creativeの選択ボタンを追加
- 選択中のモードは緑色でハイライト表示
- CreateWorld時にGameModeリソースを更新

**技術詳細**

- `SelectedGameMode`リソース: 選択中のゲームモードを保持
- `GameModeButtonMarker`コンポーネント: ボタンの識別用
- `MenuButtonAction::SelectGameMode(GameMode)`: 選択アクション

### インベントリUIをCSS Gridで自動整列に改善

**改善点**

- メインコンテナをCSS Gridに変更（`Display::Grid`）
- ハードコードされた`margin: UiRect::top(Val::Px(35.0))`を削除
- 装備パネル、メインインベントリ、クラフトリストが自動で上揃え
- コンテナUIも同様にCSS Gridで自動整列

**技術詳細**

- `grid_template_columns: vec![GridTrack::auto(), ...]`で列を定義
- `align_items: AlignItems::Start`でグリッドセル内で上揃え

### UIデザインルール策定

**新規ドキュメント**

- `.specify/memory/ui-design-rules.md`: UIデザインルールを策定
  - レイアウト整列ルール（横並びは上揃え）
  - スペーシング定数（スロット54px、間隔4px）
  - 色定数
  - コンポーネント規約

**インベントリUI修正**

- 装備パネルとインベントリグリッドの高さを揃えた
- `align_items: AlignItems::Start`で上揃え
- 装備パネルに35pxの上余白を追加

### UI改善

**修正**

- Settingsボタン重複: ESC→PauseMenuに統一、角のSettingsボタンを削除
- テキスト入力: デフォルト値（"New World"）をクリックでクリア可能に

**技術詳細**

- settings_ui.rs: InGame時のESCハンドリングを削除（PauseMenuに委譲）
- main_menu.rs: PauseMenuのSettingsボタンからSettingsUiState::SettingsOpenに遷移
- TextInput: is_defaultフラグ追加、クリック時にデフォルト値をクリア

### ワールドセーブ/ロード実装

**新機能**

- ワールドセーブ: プレイヤー位置、インベントリ、ゲームモードを保存
- ワールドロード: 既存ワールド選択時にデータ復元
- プレイ時間トラッキング: セッション時間を累計、メタデータに保存
- テキスト入力修正: ワールド名が正しく入力可能に

### ポーズメニュー・プロファイルシステム実装

**新機能**

- ポーズメニュー: ESCでゲーム中断、Resume/Save&Quit/MainMenu
- プロファイル選択画面: MOD/データプロファイルの選択
- プロファイル設定画面: プロファイル管理の説明

**ESCキー対応**

- InGame → PauseMenu → InGame（トグル）
- 各メニュー画面から前画面に戻れる

### ゲームステート制御の改善

**修正内容**

- HUD: クロスヘアをInGame時のみ表示、CrosshairRootマーカー追加
- 納品プラットフォーム: InGameステートでのみシステム実行
- マシンシステム: InGameステートでのみ実行
- インベントリUI: ホットバーHUDの表示/非表示ロジック改善
- サウンド: SoundCategoryのDefault実装をderiveに変更
- アクセシビリティ: 高コントラストモードの色変換修正

**テスト**: 109件すべて成功

---

## 2025-12-22

### 仕様に基づく実装

**新規モジュール**:
- `src/core/encryption.rs` - AES-256-GCM暗号化 (C3)
  - セーブデータ保護、Steam実績改ざん防止
  - encrypt/decrypt関数、JSON対応
- `src/core/accessibility.rs` - アクセシビリティ (A1-A3)
  - 色覚モード (P/D/T型、高コントラスト)
  - UIスケール、字幕、視覚音響インジケーター
  - 入力モード (ホールド/トグル)、感度調整
  - プリセット (視覚/聴覚/運動障害)
- `src/core/sound.rs` - サウンドシステム (S1-S4)
  - ミキシング階層 (Master > Music/SFX/Voice)
  - ピッチバリエーション (±10%)
  - 空間オーディオ、同時再生数制限
  - カテゴリ別ボリューム
- `src/ui/feedback.rs` - UIフィードバック (U2)
  - 視覚フィードバック (フラッシュ、テキスト)
  - 音声フィードバック
  - 成功/失敗/警告の区別

**更新**:
- `Cargo.toml`: aes-gcm, rand, base64 追加
- `API_REFERENCE.md`: 新モジュール追記

### 仕様変更（デザインパターン適用）
52パターンに基づき仕様を更新。詳細: `spec-change-report-2025-12-22.md`

**追加仕様**:
- サウンドシステム (S1-S4)
- アクセシビリティ拡張 (A1-A3)
- ローカライズ対応 (I1-I2)
- セーブ暗号化 (C3)
- UI応答時間要件 (U2)
- レシピ設計ガイドライン (R1-R4)
- 進行フェーズ設計 (P1-P4, L1-L3)
- テスト戦略拡張 (T1-T4)
- エディタUX原則 (E1-E6)
- マルチプレイヤー詳細 (N1-N4)

**更新ファイル**: constitution.md, core-game-mechanics.md, steam-editor-mode.md

### ドキュメント圧縮
- `patterns-compact.md`: 52パターンを表形式に圧縮
- `index-compact.md`: 全仕様を1ファイルに集約
- `.claude/skills/index.md`: スキルを統合
- 冗長ファイル削除

### 大規模調査完了
- 10レポート作成（マルチプレイ、セキュリティ、UI、MOD、レベルデザイン、Bevy、Rust、アクセシビリティ等）
- 52デザインパターン策定

---

## 2025-12-22（Earlier）

### アーキテクチャ実装
- 共通型crate作成 (factory-data-types)
- YAML統一、ホットリロード対応
- TypeScript型自動生成 (ts-rs)

### Steam/エディタ仕様
- 実績システム設計
- MOD/プロファイルシステム
- プロファイル直接編集方式

### 研究完了
- 工場ゲームUX（Factorio, Satisfactory分析）
- エディタUX（Unity, Blender, MagicaVoxel）
- テスト手法（Bevy, Tauri, E2E）
- サウンド/グラフィック実装

---

## Phase完了状況

|Phase|内容|状態|
|-----|----|----|
|1|コアエンジン、MOD基盤|✅|
|2|ロジック、物流シミュレーション|✅|
|3|電力、マルチブロック|✅|
|4|スクリプト、シグナル|✅|
|5|最適化、MODローダー|✅|
|Menu|メインメニュー、セーブ|✅|

---

## 実装済みシステム

### コア
- YAML/Luaホットリロード
- 32³チャンク + Greedy Meshing
- LOD (4段階)
- 非同期チャンク生成

### ゲームプレイ
- 電力ネットワーク (BFS)
- マルチブロック検証
- Luaスクリプト (サンドボックス)
- シグナル/ロジックゲート
- インベントリ (40スロット)
- クエスト/納品プラットフォーム
- 天候/昼夜サイクル
- 液体/熱/振動システム

### UI
- Minecraft風インベントリ
- 機械GUI
- ミニマップ/HP/クエストHUD
- 設定UI

### エディタ (Factory Data Architect)
- アイテム/レシピ/クエスト編集
- マルチブロック3D編集
- バイオーム/サウンド編集
- YAMLエクスポート

---

## テスト状況
- 全テスト: 90+件パス
- コアシステム: 90%+カバレッジ
- Clippy警告: 0

---

*git log で完全な履歴を確認可能*
