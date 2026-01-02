# E2Eビジュアルテスト

ゲームの自動E2Eテストを実行し、スクリーンショットで視覚的異常を検出する。

## 引数
$ARGUMENTS

## クイックテスト（推奨）

```bash
./scripts/e2e-quick.sh [basic|conveyor|machines|full]
```

| オプション | 内容 | 枚数 |
|-----------|------|------|
| `basic` (b) | 起動、インベントリ、デバッグHUD | 6枚 |
| `conveyor` (c) | コンベア配置、L字、T字、スプリッター | 8枚 |
| `machines` (m) | 機械配置、接続、動作確認 | 6枚 |
| `full` (f) | 全テスト | 20枚 |

## 手動テスト（高速版）

ゲームをバックグラウンドで起動後、連続でスクリーンショットを撮る:

```bash
# 1. ゲーム起動
pkill -9 -f idle_factory 2>/dev/null || true
DISPLAY=:10 cargo run &

# 2. 20秒待機後、連続スクショ
sleep 20

# 3. スクショ撮影（コピペで一括実行）
mkdir -p screenshots/verify
DISPLAY=:10 scrot screenshots/verify/01.png
DISPLAY=:10 xdotool search --name "Idle" windowactivate
sleep 0.5
DISPLAY=:10 xdotool click 1
sleep 0.5
DISPLAY=:10 scrot screenshots/verify/02.png
DISPLAY=:10 xdotool key 2
DISPLAY=:10 xdotool mousemove 450 350 click 1
DISPLAY=:10 xdotool mousemove 500 350 click 1
DISPLAY=:10 xdotool mousemove 550 350 click 1
DISPLAY=:10 xdotool mousemove 600 350 click 1
DISPLAY=:10 scrot screenshots/verify/03.png
DISPLAY=:10 xdotool key q
DISPLAY=:10 xdotool mousemove 650 350 click 1
DISPLAY=:10 xdotool key q
DISPLAY=:10 xdotool mousemove 700 350 click 1
DISPLAY=:10 scrot screenshots/verify/04.png
DISPLAY=:10 xdotool key q
DISPLAY=:10 xdotool mousemove 450 400 click 1
DISPLAY=:10 xdotool key q
DISPLAY=:10 xdotool mousemove 500 400 click 1
DISPLAY=:10 scrot screenshots/verify/05.png
echo "Done: 5 screenshots"
ls screenshots/verify/

# 4. 終了
pkill -9 -f idle_factory
```

## スクリーンショット保存先

```
/home/bacon/idle_factory/screenshots/verify/
```

### 4. 視覚的異常の検出

#### 4.1 自動比較（推奨）

```bash
# Smart Compare: SSIM + 知覚ハッシュ + エッジ検出
python3 scripts/vlm_check/smart_compare.py baseline.png current.png

# JSON出力
python3 scripts/vlm_check/smart_compare.py baseline.png current.png --json

# ピクセル比較（シンプル版）
python3 scripts/vlm_check/pixel_compare.py baseline.png current.png
```

**Smart Compare メトリクス:**

| メトリクス | 意味 | 閾値 |
|-----------|------|------|
| SSIM | 構造的類似度 | 0.95+ で OK |
| Hash距離 | 知覚ハッシュ差 | 10以下 で OK |
| Edge類似度 | UIレイアウト | 0.8+ で OK |

**severity判定:**
- `none`: ほぼ同一
- `minor`: 微差（影、アンチエイリアス）
- `major`: 明らかな違い
- `critical`: 完全に異なる

#### 4.2 目視チェック項目

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
