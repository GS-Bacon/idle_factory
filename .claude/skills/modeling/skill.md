# 3Dモデリングスキル

Blender MCP / HTMLプレビューを使ったアセット作成。

## トリガー

- `/generate-model` - 3Dモデル生成
- `/generate-asset` - アセット生成（テクスチャ含む）

## スタイルガイド

**→ `style-guide.md` 参照**

| 項目 | 値 |
|------|-----|
| カラー | グレースケール基調 (#4a4a4a〜#888888) |
| インジケーター | 緑 (#00ff00) |
| modelScale | 1.5〜1.8 |
| 当たり判定 | 1ブロック内に収める |

---

## 推奨ワークフロー（HTML直接変換）

```
[設定値JSON] → [export-html-model.js] → [GLB] → [Blenderインポート] → [確認/調整]
```

### フロー詳細

1. **ユーザー**: 設定値をJSONで渡す
2. **Claude**: スクリプト実行でGLB生成
3. **Claude**: BlenderにインポートしてMCPで表示

### 実行コマンド

```bash
# 設定値からGLB生成
node scripts/export-html-model.js '{"bodyWidth": 0.26, ...}' /tmp/model.glb

# Blenderにインポート（MCP経由）
mcp__blender__execute_blender_code: bpy.ops.import_scene.gltf(filepath="/tmp/model.glb")
```

---

## 設定値パラメータ（採掘機 - リファレンス）

```json
{
  "bodyWidth": 0.31,
  "bodyHeight": 0.16,
  "bodyDepth": 0.32,
  "bodyColor": "#4a4a4a",
  "shaftLength": 0.17,
  "shaftWidth": 0.08,
  "shaftColor": "#666666",
  "drillStyle": "cone",
  "drillLength": 0.17,
  "drillWidth": 0.14,
  "drillColor": "#888888",
  "legCount": 4,
  "legSpread": 0.34,
  "legThickness": 0.04,
  "legColor": "#555555",
  "indicatorColor": "#00ff00",
  "outletSize": 0.15,
  "outletColor": "#666666",
  "collisionSize": 1.0,
  "showCollision": true,
  "modelScale": 1.7
}
```

---

## プリセット

| スタイル | 特徴 |
|----------|------|
| **Factorio** | 脚なし、螺旋ドリル、黄色インジケーター |
| **Satisfactory** | オレンジ本体、太い脚、大きめ |
| **Minecraft** | 角ばった、四角錐ドリル、小さめ |
| **Steampunk** | 茶色系、3本脚、八角形ドリル |

---

## スクリプト

| ファイル | 用途 |
|----------|------|
| `scripts/export-html-model.js` | HTML→GLB直接変換 |
| `scripts/generate-miner-model.py` | Blender Pythonで生成（バックアップ） |
| `scripts/blender-mcp.sh` | Blender MCP起動 |

---

## HTMLプレビュー

```bash
# サーバー起動
cd UIプレビュー && python3 -m http.server 8080 --bind 0.0.0.0 &

# URL
http://100.84.170.32:8080/mining_drill_preview.html
```

### プレビュー機能

| 機能 | 説明 |
|------|------|
| スライダー | 各パーツのサイズ調整 |
| カラーピッカー | 色の調整 |
| プリセット | スタイル一括適用 |
| Copy/Paste Settings | JSON設定のコピー/適用 |
| Download GLB | GLBダウンロード（ブラウザ用） |

---

## Blender MCP

### 起動

```bash
./scripts/blender-mcp.sh --bg  # バックグラウンド起動
./scripts/blender-mcp.sh --stop # 停止
```

### よく使うMCPツール

| ツール | 用途 |
|--------|------|
| `execute_blender_code` | Pythonコード実行 |
| `get_viewport_screenshot` | スクショ（バックグラウンドでは黒くなる） |
| `get_scene_info` | シーン情報取得 |

### レンダリング確認

バックグラウンドモードではビューポートスクショが黒くなるため、レンダリングで確認:

```python
bpy.context.scene.render.filepath = "/tmp/render.png"
bpy.ops.render.render(write_still=True)
```

---

## GLBエクスポート

```python
bpy.ops.export_scene.gltf(
    filepath="/path/to/model.glb",
    export_format='GLB',
    use_selection=True,
    export_apply=True
)
```

出力先: `assets/models/machines/[name].glb`

---

## 既存プレビュー

| モデル | ファイル | 状態 |
|--------|----------|------|
| 採掘機 | `mining_drill_preview.html` | ✅ 完成 |
| 精錬炉 | `furnace_preview.html` | ✅ 完成 |
| 粉砕機 | `crusher_preview.html` | ✅ 完成 |
| 組立機 | `assembler_preview.html` | ✅ 完成 |
| コンベア | `conveyor_preview.html` | ✅ 完成（5形状） |
| アイテム | `items_preview.html` | ✅ 完成（6種類） |
| インデックス | `index.html` | ✅ 完成 |

---

## 新規モデル作成フロー

**コード仕様を先に調査してから作成する**

```
[コード調査] → [仕様抽出] → [HTMLプレビュー作成] → [デプロイ] → [確認]
```

### 1. コード調査

```bash
# 機械仕様を確認
src/game_spec/machines.rs  # MACHINE_BLOCK_SIZE, I/Oポート
src/components/machines/conveyor.rs  # コンベア形状
```

### 2. 仕様抽出（チェックリスト）

| 項目 | 確認先 |
|------|--------|
| 当たり判定サイズ | `MACHINE_BLOCK_SIZE` (1.0m) |
| I/Oポート位置 | `MachineSpec.ports` (Front/Back/Left/Right) |
| ブロック形状 | `ConveyorShape` 等 |

### 3. HTMLプレビュー作成

- 既存プレビューをテンプレートとしてコピー
- I/Oポートマーカーを仕様通りに配置
- 当たり判定ボックス表示機能を含める

### 4. デプロイ確認

```bash
cd UIプレビュー && python3 -m http.server 8080 --bind 0.0.0.0 &
# http://100.84.170.32:8080/
```

---

## 詳細

→ `style-guide.md` - **デザイン統一ルール（必読）**
→ `best-practices.md` - AI生成のコツ
→ `guides/index.md` - 機械別ガイド
