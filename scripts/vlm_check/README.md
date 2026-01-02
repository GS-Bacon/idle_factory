# VLM Visual Bug Checker

Claude Vision APIを使用してゲームのスクリーンショットを解析し、視覚的なバグを検出します。

## セットアップ

```bash
# 1. API キーを設定
export ANTHROPIC_API_KEY='your-api-key'

# 2. 依存関係インストール
pip3 install anthropic

# 3. スクリーンショットツール（Linux）
sudo apt-get install scrot
```

## 使い方

### クイックスタート

```bash
# ゲームを起動してスクリーンショットを撮り、チェック
./scripts/vlm_check.sh

# 既存のスクリーンショットをチェック
./scripts/vlm_check.sh screenshot.png

# 詳細チェック
./scripts/vlm_check.sh --thorough screenshot.png
```

### チェックレベル

| レベル | 内容 | 所要時間 | 用途 |
|--------|------|----------|------|
| `quick` | 基本サニティチェック (5項目) | ~2秒 | CI、日常確認 |
| `standard` | 標準ビジュアルチェック (10項目) | ~5秒 | 通常の開発 |
| `thorough` | 詳細検査 (20+項目) | ~10秒 | リリース前 |
| `conveyor` | コンベア専用チェック | ~5秒 | コンベア変更後 |
| `ui` | UI専用チェック | ~5秒 | UI変更後 |
| `chunk` | チャンク境界チェック | ~5秒 | 地形生成変更後 |

### コマンドラインオプション

```bash
./scripts/vlm_check.sh [OPTIONS] [screenshot.png]

Options:
  -q, --quick       クイックチェック
  -s, --standard    標準チェック（デフォルト）
  -t, --thorough    詳細チェック
  -c, --conveyor    コンベア専用
  -u, --ui          UI専用
  -k, --chunk       チャンク境界専用

  --full-suite      全チェックタイプを実行
  --no-game         ゲーム起動なし（既存スクショ使用）
  --delay SEC       スクショ前の待機時間（デフォルト: 10秒）

  -o, --output DIR  レポート出力先
  -h, --help        ヘルプ表示
```

### Python直接使用

```python
from scripts.vlm_check.visual_checker import check_screenshot

result = check_screenshot("screenshot.png", level="thorough")
print(result["result"]["status"])  # PASS, FAIL, WARNING
print(result["result"]["score"])   # 0-100
```

## 推奨実行タイミング

### 必須（リリース前）

```bash
# 全チェックタイプを実行
./scripts/vlm_check.sh --full-suite
```

### 変更内容に応じて

| 変更内容 | 推奨チェック | コマンド |
|----------|-------------|----------|
| テクスチャ/モデル追加・変更 | thorough | `--thorough` |
| コンベアロジック変更 | conveyor | `--conveyor` |
| UI/フォント変更 | ui | `--ui` |
| チャンク/地形生成変更 | chunk | `--chunk` |
| 機械追加・変更 | thorough + conveyor | 両方実行 |
| シェーダー/ライティング変更 | thorough | `--thorough` |

### CI/CD統合

```yaml
# .github/workflows/visual-check.yml
visual-check:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Setup
      run: |
        pip install anthropic
        sudo apt-get install -y scrot xvfb
    - name: Visual Check
      env:
        ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
      run: |
        Xvfb :99 &
        export DISPLAY=:99
        ./scripts/vlm_check.sh --quick
```

## 出力例

### PASS時

```json
{
  "status": "PASS",
  "score": 95,
  "issues": [],
  "warnings": ["FPSが28と低め"],
  "observations": ["コンベア5本確認、正常に動作"]
}
```

### FAIL時

```json
{
  "status": "FAIL",
  "score": 45,
  "critical_issues": [
    "画面左側にピンク色のテクスチャ抜けあり",
    "コンベアモデルが表示されていない"
  ],
  "major_issues": [
    "ホットバーのスロット3-5が黒く表示"
  ],
  "recommendations": [
    "テクスチャファイルの存在確認",
    "モデル読み込みログの確認"
  ]
}
```

## トラブルシューティング

### API エラー

```
Error: anthropic.AuthenticationError
```
→ `ANTHROPIC_API_KEY` が正しく設定されているか確認

### スクリーンショット取得失敗

```
Error: Failed to save screenshot
```
→ `DISPLAY` 環境変数を確認、`xvfb` が起動しているか確認

### ゲーム起動失敗

```
Error: Game crashed during startup
```
→ `cargo build --release` が成功するか確認

## コスト目安

Claude Sonnet 4 使用時:
- quick: ~$0.003/回
- standard: ~$0.005/回
- thorough: ~$0.01/回
- full-suite: ~$0.05/回

月100回実行で約$1-5程度。

## 検出できるバグの例

### テクスチャ系
- ピンク/マゼンタのmissing texture
- UVマッピングのずれ
- ミップマップの問題

### モデル系
- モデル欠損・変形
- 浮遊・めり込み
- 向きの不整合

### UI系
- 文字化け・重なり
- レイアウト崩れ
- 表示欠け

### ワールド系
- チャンク境界の隙間
- 地形の不連続
- ライティング異常
