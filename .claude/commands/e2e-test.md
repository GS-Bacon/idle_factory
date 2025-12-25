# E2Eテスト実行コマンド

ゲームの自動操作テストを実行し、結果を確認します。

## 引数
$ARGUMENTS

## 利用可能なテストシナリオ

| シナリオ名 | 説明 | キーバインド |
|-----------|------|-------------|
| `full_test` | フルテスト（メインメニューから全機能） | F11 |
| `interaction_test` | インタラクションテスト（全操作パターン） | F8 |
| `ui_inventory_test` | インベントリUIテスト | F10 |
| `ui_main_menu_test` | メインメニューテスト | - |
| `gameplay_basic_test` | 基本ゲームプレイテスト | - |

## 指示

### 1. 引数の解釈

引数が指定されている場合:
- `full` / `all` / `フル` → full_testを実行
- `interaction` / `操作` → interaction_testを実行
- `inventory` / `インベントリ` → ui_inventory_testを実行
- `menu` / `メニュー` → ui_main_menu_testを実行
- `gameplay` / `基本` → gameplay_basic_testを実行
- `report` / `レポート` / `結果` → 最新のテストレポートを表示
- `list` / `一覧` → 利用可能なシナリオ一覧を表示

引数なし → full_testを実行

### 2. テストの実行

```bash
# コマンドライン引数でテストを自動実行
DISPLAY=:10 timeout 120 cargo run -- --e2e-test

# または手動でキーを押す
# F8: インタラクションテスト
# F10: インベントリUIテスト
# F11: フルテスト
# F12: UIダンプ
```

### 3. 結果の確認

テスト完了後、以下のファイルを確認:

1. **最優先**: `screenshots/test_report.txt` - テスト結果サマリ（トークン消費小）
2. **必要時**: `screenshots/full_test_*.txt` - UIダンプ（トークン消費小）
3. **問題時のみ**: `screenshots/full_test_*.png` - スクリーンショット（トークン消費大）

### 4. レポートの読み方

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

### 5. トークン消費最適化

- 通常: テキストレポート（test_report.txt）のみ確認
- 問題発生時: 該当のUIダンプを確認
- 視覚確認が必要な場合のみ: スクリーンショットを確認

### 6. 手動テストの場合

ゲームを起動して以下のキーを押す:
- F8: インタラクションテスト開始
- F9: 手動スクリーンショット撮影
- F10: インベントリUIテスト開始
- F11: フルテスト開始
- F12: UIダンプ出力

## 出力ファイル

| ファイル | 内容 | トークン消費 |
|---------|------|-------------|
| `screenshots/test_report.txt` | テスト結果レポート | 小 |
| `screenshots/*_ui_dump_*.txt` | UI構造ダンプ | 小 |
| `screenshots/*.png` | スクリーンショット | 大 |

## 関連ファイル

- `src/core/e2e_test.rs` - E2Eテストシステム本体
- `src/ui/ui_test_scenarios.rs` - UIテストシナリオ定義
