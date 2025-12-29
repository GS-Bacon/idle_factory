# E2Eビジュアルテスト

WASMゲームとネイティブ版の自動E2Eテストを実行し、スクリーンショットで視覚的異常を検出する。

## 引数
$ARGUMENTS

## 引数の解析

- **--quick**: 基本テストのみ（デフォルト、6テスト）
- **--full**: 全テスト実行（15テスト）
- **--native**: ネイティブ版でテスト（xdotool使用、推奨）
- **--wasm**: WASM版でテスト（Playwright、制限あり）
- **--skip-build**: ビルドをスキップ

---

## テスト方式の選択

### ネイティブ版（--native）推奨 ✅

- xdotoolでキー入力を完全にシミュレート可能
- Pointer Lock制限なし
- 全ての操作テストが動作

### WASM版（--wasm）制限あり ⚠️

- Playwrightの制限でPointer Lock取得不可
- **キー入力テスト不可**（Tキー、Eキー、F3等）
- 描画確認のみ（初期画面、UI表示）

---

## 実行手順

### ネイティブ版テスト（推奨）

```bash
# 1. 既存プロセスを停止
pkill -f "idle_factory" || true

# 2. ゲームをバックグラウンドで起動
DISPLAY=:10 cargo run --release 2>&1 &
sleep 15

# 3. 初期画面スクリーンショット
DISPLAY=:10 scrot /home/bacon/idle_factory/screenshots/verify/native_01_initial.png

# 4. ウィンドウをアクティブ化
DISPLAY=:10 xdotool search --name "Idle Factory" windowactivate
sleep 0.5

# 5. クリックしてゲーム開始
DISPLAY=:10 xdotool mousemove --window $(DISPLAY=:10 xdotool search --name "Idle Factory" | head -1) 400 300
DISPLAY=:10 xdotool click 1
sleep 0.5
DISPLAY=:10 scrot /home/bacon/idle_factory/screenshots/verify/native_02_active.png

# 6. Eキーでインベントリを開く
DISPLAY=:10 xdotool key e
sleep 0.5
DISPLAY=:10 scrot /home/bacon/idle_factory/screenshots/verify/native_03_inventory.png

# 7. ESCで閉じる
DISPLAY=:10 xdotool key Escape
sleep 0.5
DISPLAY=:10 scrot /home/bacon/idle_factory/screenshots/verify/native_04_closed.png

# 8. F3でデバッグHUD
DISPLAY=:10 xdotool key F3
sleep 0.5
DISPLAY=:10 scrot /home/bacon/idle_factory/screenshots/verify/native_05_debug.png

# 9. Tキーでコマンド入力
DISPLAY=:10 xdotool key t
sleep 0.3
DISPLAY=:10 xdotool type "/creative"
DISPLAY=:10 xdotool key Return
sleep 0.5
DISPLAY=:10 scrot /home/bacon/idle_factory/screenshots/verify/native_06_creative.png

# 10. ゲーム終了
pkill -f "idle_factory" || true
```

### WASM版テスト（描画確認のみ）

```bash
# 1. WASMビルド（--skip-buildでスキップ可）
./deploy-wasm.sh

# 2. テスト実行
node /home/bacon/idle_factory/test-wasm-interactions.js

# 注意: キー入力はPointer Lock制限で動作しない
# 描画確認のみ有効
```

### 3. スクリーンショット確認

テスト完了後、以下のファイルを確認:

```
/home/bacon/idle_factory/screenshots/verify/
├── native_01_initial.png   # 初期状態
├── native_02_active.png    # アクティベート後
├── native_03_inventory.png # インベントリUI（Eキー）
├── native_04_closed.png    # UI閉じた後（ESC）
├── native_05_debug.png     # デバッグHUD（F3）
├── native_06_creative.png  # クリエイティブモード（Tキー+/creative）
└── test_*.png              # WASM版（参考）
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
| インベントリUI | 左右パネル表示 | 欠け、重なり |
| デバッグHUD | 左上に情報表示 | 非表示、文字化け |

---

## テストケース一覧

### ネイティブ版テスト（--native）

| # | テスト | 操作 | 確認内容 |
|---|--------|------|----------|
| 1 | initial | 起動 | ゲーム起動、ロード完了 |
| 2 | active | クリック | ゲーム開始 |
| 3 | inventory | Eキー | インベントリUI表示 |
| 4 | close_ui | ESC | UI閉じる |
| 5 | debug_hud | F3 | デバッグHUD表示 |
| 6 | command | Tキー | コマンド入力欄表示 |
| 7 | creative | /creative | クリエイティブモード切替 |
| 8 | hotbar | 1-9キー | ホットバー選択 |
| 9 | movement | WASD | プレイヤー移動 |
| 10 | block_break | 左クリック | ブロック破壊 |
| 11 | block_place | 右クリック | ブロック設置 |

### WASM版テスト（--wasm）制限あり

| # | テスト | 確認内容 | 動作 |
|---|--------|----------|------|
| 1 | initial_state | ゲーム起動、ロード完了 | ✅ |
| 2 | ui_display | UI表示（Quest, Deliveries, Hotbar） | ✅ |
| 3-15 | キー入力テスト | 各種操作 | ❌ Pointer Lock制限 |

---

## 結果の解釈

### 判定基準

| 結果 | 判定 |
|------|------|
| 全スクリーンショットで異常なし | 成功 |
| 黒い穴、透け | レンダリングバグ（要修正） |
| UI非表示 | キー入力バグ（要修正） |
| コンソールエラー（アセット読み込み） | 警告（ゲームに影響なし） |

---

## トラブルシューティング

### ネイティブ版が起動しない

```bash
# ディスプレイ確認
echo $DISPLAY
# Xvfb起動（必要な場合）
Xvfb :10 -screen 0 1280x720x24 &
export DISPLAY=:10
```

### xdotoolでウィンドウが見つからない

```bash
# ウィンドウ一覧
DISPLAY=:10 wmctrl -l
# 名前で検索
DISPLAY=:10 xdotool search --name "Idle"
```

### WASM版でゲームがロードされない

```bash
# サーバー確認
curl http://localhost:8080
# プロセス確認
ss -tlnp | grep 8080
# 再起動
./deploy-wasm.sh
```

### スクリーンショットが真っ黒

```bash
# scrotの代わりにimport使用
DISPLAY=:10 import -window root screenshot.png
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
| **Pointer Lock** | ESC後の復帰、UI閉じた後の操作 |

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
| WASM版キー入力不可（自動テスト） | Pointer Lock API制限 | ネイティブ版で代替 |
