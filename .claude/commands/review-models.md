# モデルレビュー

HTMLプレビューでモデルデザインを確認・調整するワークフロー。

## 引数
$ARGUMENTS

## 引数の解析

- **モデル名**: 必須（例: "採掘機", "コンベア", "チェスト"）
- **--style**: スタイル指定（industrial/steampunk/sci-fi/fantasy/minecraft）
- **--size**: ゲーム内サイズ（デフォルト: 1ブロック = 1m）

---

## ワークフロー

### Step 1: 参考調査

1. Web検索で参考画像・デザインを収集
2. 類似ゲーム（Factorio, Satisfactory, Minecraft等）のデザインを参照
3. Sketchfabで参考モデルを検索

### Step 2: HTMLプレビュー作成

1. `UIプレビュー/` フォルダにHTMLファイルを作成
2. Three.jsでインタラクティブな3Dプレビューを実装
3. 以下の機能を含める:
   - パラメータ調整スライダー
   - プリセット（複数スタイル）
   - 設定値のエクスポート/インポート（JSON）
   - 1ブロック境界線表示
   - サイズ表示

### Step 3: HTTPサーバー起動

```bash
# Tailscale経由でアクセス可能にする
cd /home/bacon/idle_factory/UIプレビュー
python3 -m http.server 8080 --bind 0.0.0.0 &

# アクセスURL
tailscale ip -4  # IPを確認
# → http://[TAILSCALE_IP]:8080/[ファイル名].html
```

### Step 4: デザイン確定

1. ユーザーがブラウザでパラメータを調整
2. 気に入った設定をCopy Settingsでコピー
3. そのJSON設定を元にBlenderでモデリング

### Step 5: Blenderで実装

HTMLで確定したパラメータを使って`mcp__blender__execute_blender_code`でモデリング。

---

## HTMLテンプレート構造

```html
<!DOCTYPE html>
<html>
<head>
    <script type="importmap">
    {
        "imports": {
            "three": "https://unpkg.com/three@0.160.0/build/three.module.js",
            "three/addons/": "https://unpkg.com/three@0.160.0/examples/jsm/"
        }
    }
    </script>
</head>
<body>
    <div id="canvas-container"></div>
    <div id="controls">
        <!-- スライダー、カラーピッカー、プリセットボタン -->
        <!-- Export/Import用のtextarea -->
    </div>
    <script type="module">
        import * as THREE from 'three';
        import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
        // 3Dシーン構築
    </script>
</body>
</html>
```

## 必須機能

| 機能 | 説明 |
|------|------|
| OrbitControls | ドラッグで回転、スクロールでズーム |
| パラメータスライダー | 各パーツのサイズ・位置調整 |
| カラーピッカー | 色の調整 |
| プリセット | 複数スタイルをワンクリックで切替 |
| JSON Export | 設定値をコピー可能に |
| JSON Import | 設定値を貼り付けて適用 |
| 1ブロック境界線 | サイズ確認用ワイヤーフレーム |
| サイズ表示 | W x D x H をリアルタイム表示 |

## 既存プレビュー

| モデル | ファイル |
|--------|----------|
| 採掘機 | `UIプレビュー/mining_drill_preview.html` |

---

## 例: 新規モデルのレビュー

```
/review-models コンベア --style industrial
```

1. コンベアの参考画像を検索
2. `UIプレビュー/conveyor_preview.html` を作成
3. HTTPサーバーでアクセス可能にする
4. ユーザーがデザインを確定
5. Blenderでモデリング

---

## 注意事項

- **HTTPサーバーは使用後に停止**: `pkill -f "python3 -m http.server"`
- **UIプレビューフォルダはgitignore済み**: 一時ファイルとして扱う
- **設定JSONは保存推奨**: 後でBlenderで再現するため
