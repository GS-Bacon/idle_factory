# E2Eテスト実行コマンド

ゲームまたはエディタの自動操作テストを実行し、結果を確認します。

## 引数
$ARGUMENTS

## テスト対象

| 対象 | 引数 | 説明 |
|------|------|------|
| ゲーム | `game`, `ゲーム`, (default) | Bevy製ゲーム本体のテスト |
| エディタ | `editor`, `エディタ` | Factory Data Architectのテスト |

## ゲームテストシナリオ

| シナリオ名 | 説明 | キーバインド |
|-----------|------|-------------|
| `full_test` | フルテスト（メインメニューから全機能） | F11 |
| `interaction_test` | インタラクションテスト（全操作パターン） | F8 |
| `ui_inventory_test` | インベントリUIテスト | F10 |
| `ui_main_menu_test` | メインメニューテスト | - |
| `gameplay_basic_test` | 基本ゲームプレイテスト | - |
| `human_test` | 人間らしい挙動テスト（新機能） | - |
| `button_click_test` | UI要素クリックテスト（新機能） | - |

### 新機能（人間らしい挙動）

- **Think**: ランダムな思考時間
- **WaitRandom**: ランダムな待機時間
- **TapKeyHuman**: 人間らしいキータップ
- **MouseMoveSmooth**: ベジェ曲線による滑らかなマウス移動
- **MouseClickHuman**: 揺れを伴うクリック
- **ClickElement**: UI要素を名前/テキストで検索してクリック
- **Scroll**: マウスホイールスクロール
- **DoubleClick**: ダブルクリック
- **DragDrop**: ドラッグ&ドロップ

## エディタテストシナリオ

| シナリオ名 | 説明 |
|-----------|------|
| `full` | 全タブナビゲーション + アイテム作成 |
| `tabs` | タブ切り替えテスト |
| `items` | アイテムエディタテスト |
| `recipes` | レシピエディタテスト |
| `design` | UIデザインパターン準拠チェック |

## 指示

### 1. 引数の解釈

**ゲームテスト:**
- `full` / `all` / `フル` → full_testを実行
- `interaction` / `操作` → interaction_testを実行
- `inventory` / `インベントリ` → ui_inventory_testを実行
- `menu` / `メニュー` → ui_main_menu_testを実行
- `gameplay` / `基本` → gameplay_basic_testを実行
- `report` / `レポート` / `結果` → 最新のテストレポートを表示
- `list` / `一覧` → 利用可能なシナリオ一覧を表示

**エディタテスト:**
- `editor` / `エディタ` → エディタのフルテストを実行
- `editor full` → 全機能テスト
- `editor tabs` → タブナビゲーションテスト
- `editor report` → エディタテストレポート表示

引数なし → ゲームのfull_testを実行

### 2. ゲームテストの実行

```bash
# コマンドライン引数でテストを自動実行
DISPLAY=:10 timeout 120 cargo run -- --e2e-test

# または手動でキーを押す
# F8: インタラクションテスト
# F10: インベントリUIテスト
# F11: フルテスト
# F12: UIダンプ
```

### 3. エディタテストの実行

```bash
# エディタディレクトリに移動
cd tools/factory-data-architect

# Playwrightテスト実行
DISPLAY=:10 npm run e2e

# ヘッドモード（ブラウザ表示）
DISPLAY=:10 npm run e2e:headed

# 特定のテストのみ
DISPLAY=:10 npx playwright test --grep "Tab Navigation"
```

### 4. 結果の確認

**ゲームテスト:**
1. **最優先**: `screenshots/test_report.txt` - テスト結果サマリ
2. **必要時**: `screenshots/full_test_*.txt` - UIダンプ
3. **問題時のみ**: `screenshots/full_test_*.png` - スクリーンショット

**エディタテスト:**
1. **最優先**: `tools/factory-data-architect/e2e-results.json` - JSON結果
2. **必要時**: `tools/factory-data-architect/e2e-screenshots/*.png` - スクリーンショット
3. **レポート**: `npm run e2e:report` でHTMLレポート表示

### 5. レポートの読み方

```
# E2E Test Report
Generated: 2025-12-25 12:00:00

## Summary
- Total: 15
- Passed: 14 ✅
- Failed: 1 ❌

## Failed Tests
### ❌ TestName
- Message: 詳細メッセージ
- Expected: 期待値
- Actual: 実際の値
```

### 6. トークン消費最適化

- 通常: テキストレポート（test_report.txt / e2e-results.json）のみ確認
- 問題発生時: 該当のUIダンプやスクリーンショットを確認
- 視覚確認が必要な場合のみ: 画像ファイルを確認

### 7. 手動テストの場合

**ゲーム:**
- F8: インタラクションテスト開始
- F9: 手動スクリーンショット撮影
- F10: インベントリUIテスト開始
- F11: フルテスト開始
- F12: UIダンプ出力

**エディタ:**
- `npm run dev` でVite開発サーバー起動
- ブラウザで `http://localhost:1420` を開く
- 手動で操作を確認

## 出力ファイル

### ゲームテスト

| ファイル | 内容 | トークン消費 |
|---------|------|-------------|
| `screenshots/test_report.txt` | テスト結果レポート | 小 |
| `screenshots/*_ui_dump_*.txt` | UI構造ダンプ | 小 |
| `screenshots/*.png` | スクリーンショット | 大 |

### エディタテスト

| ファイル | 内容 | トークン消費 |
|---------|------|-------------|
| `tools/factory-data-architect/e2e-results.json` | JSON結果 | 小 |
| `tools/factory-data-architect/e2e-screenshots/*.png` | スクリーンショット | 中 |
| `tools/factory-data-architect/e2e-report/` | HTMLレポート | - |

## 関連ファイル

### ゲーム
- `src/core/e2e_test.rs` - E2Eテストシステム本体
- `src/ui/ui_test_scenarios.rs` - UIテストシナリオ定義

### エディタ
- `tools/factory-data-architect/e2e/editor.spec.ts` - E2Eテストシナリオ
- `tools/factory-data-architect/playwright.config.ts` - Playwright設定
