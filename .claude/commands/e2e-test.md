# E2Eビジュアルテスト

WASMゲームの自動E2Eテストを実行し、スクリーンショットで視覚的異常を検出する。

## 引数
$ARGUMENTS

## 引数の解析

- **--quick**: 基本テストのみ（デフォルト、6テスト）
- **--full**: 全テスト実行（15テスト）
- **--skip-build**: WASMビルドをスキップ

---

## 実行手順

### 1. WASMビルド（--skip-buildでスキップ可）

```bash
./deploy-wasm.sh
```

### 2. テスト実行

```bash
node /home/bacon/idle_factory/e2e-visual-test.js $ARGUMENTS
```

### 3. スクリーンショット確認

テスト完了後、以下のファイルを確認:

```
/home/bacon/idle_factory/screenshots/e2e/
├── 01_initial.png      # 初期状態
├── 02_activated.png    # アクティベート後
├── 03_inventory_open.png   # インベントリUI
├── 04_inventory_close.png  # UI閉じた後
├── 05_hotbar.png       # ホットバー選択
├── 06_debug_hud.png    # デバッグHUD
└── results.json        # テスト結果
```

### 4. 視覚的異常の検出

各スクリーンショットで以下をチェック:

| チェック項目 | 正常 | 異常 |
|-------------|------|------|
| 黒い穴 | なし | 地面や壁に黒い部分 |
| UI表示 | 正しく表示 | 欠け、ずれ、透明 |
| テキスト | 読める | 文字化け、欠け |
| ホットバー | 9スロット表示 | 欠け、位置ずれ |
| アイテム | アイコン表示 | 欠け、色異常 |
| 背景 | 地形が見える | 真っ黒、真っ白 |

---

## テストケース一覧

### 基本テスト（--quick）

| # | テスト | 確認内容 |
|---|--------|----------|
| 1 | initial_state | ゲーム起動、ロード完了 |
| 2 | activated | ポインターロック取得 |
| 3 | inventory_open | Eキーでインベントリ開く |
| 4 | inventory_close | ESCで閉じる |
| 5 | hotbar_selection | 1-9キーで選択 |
| 6 | debug_hud | F3でデバッグ表示 |

### 拡張テスト（--full）

| # | テスト | 確認内容 |
|---|--------|----------|
| 7 | command_input | Tキーでコマンド欄 |
| 8 | creative_mode | /creativeでモード切替 |
| 9 | creative_inventory | クリエイティブUI |
| 10 | movement | WASD移動 |
| 11 | block_break | 左クリック破壊 |
| 12 | block_place | 右クリック設置 |
| 13 | machine_ui | 機械右クリック |
| 14 | survival_mode | /survivalでモード切替 |
| 15 | final_state | 最終状態確認 |

---

## 結果の解釈

### results.json の構造

```json
{
  "passed": ["test1", "test2"],
  "failed": [{"name": "test3", "error": "reason"}],
  "screenshots": [{"name": "initial", "filename": "01_initial.png", "description": "..."}],
  "consoleErrors": ["error message"],
  "startTime": "...",
  "endTime": "..."
}
```

### 判定基準

| 結果 | 判定 |
|------|------|
| failed = 0, consoleErrors = 0 | 成功 |
| failed > 0 | テスト失敗（要修正） |
| consoleErrors > 0 | 警告（確認推奨） |

---

## トラブルシューティング

### ゲームがロードされない

```bash
# サーバー確認
curl http://localhost:8080
# プロセス確認
pgrep -f "python.*8080"
# 再起動
./deploy-wasm.sh
```

### スクリーンショットが真っ黒

```bash
# ディスプレイ確認
echo $DISPLAY
# Xvfb起動
Xvfb :99 -screen 0 1280x720x24 &
export DISPLAY=:99
```

### Playwrightエラー

```bash
# ブラウザ再インストール
npx playwright install chromium
```

---

## バグ探しノウハウ

### 重点チェック項目

| カテゴリ | 具体的な確認ポイント |
|----------|---------------------|
| **境界値** | スタック上限（999）、機械スロット（64）、チャンク境界 |
| **状態遷移** | UI開閉、モード切替、ポーズ復帰時の変数リセット |
| **同時操作** | UI表示中のブロック操作、複数機械の同時動作 |
| **リソース** | エンティティ削除時の子要素クリーンアップ、メモリリーク |

### テストで見つけやすいバグ

```bash
# cargo test で確認
cargo test 2>&1 | grep -E "(FAILED|panicked)"
```

1. **オーバーフロー**: スタック上限超過時の動作
2. **状態不整合**: UI表示中の操作可否
3. **境界処理**: チャンク境界でのメッシュ生成

### テストで見つけにくいバグ（目視確認）

1. **視覚的問題**: 黒い穴、透け、描画欠け
2. **アニメーション**: カクつき、ちらつき、不自然な動き
3. **パフォーマンス**: FPS低下、フリーズ

### 発見したバグの記録

新しいバグを発見したら:

1. `CLAUDE.md` の「よくあるバグと対策」に追記
2. 対応するテストを `tests/e2e_test.rs` に追加
3. このスキルの「重点チェック項目」を更新

### 直近で発見したバグパターン

| バグ | 原因 | テスト名 |
|------|------|----------|
| 機械スロット無限追加 | 上限チェック漏れ | `test_furnace_ui_state_consistency` |
| コンベアアイテム重複 | 転送時のvisual_entity未引継ぎ | `test_visual_entity_handoff` |
| 横合流アニメ不自然 | lateral_offset未対応 | `test_conveyor_side_merge_offset` |
| レイキャスト貫通 | DDA未使用 | 手動確認 |
| 機械破壊時にアイテム消失 | 入出力スロットの返却漏れ | `test_crusher_break_returns_items`, `test_furnace_break_returns_items` |
