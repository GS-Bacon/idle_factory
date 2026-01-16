# 3Dモデリングスキル

Blender MCP / AI 3D生成を使ったアセット作成。

## トリガー

- `/generate-model` - 3Dモデル生成
- `/generate-asset` - アセット生成（テクスチャ含む）

## 使用ツール

| ツール | 用途 |
|--------|------|
| `mcp__blender__generate_hyper3d_model_via_text` | テキストから3D生成 |
| `mcp__blender__search_sketchfab_models` | 既存モデル検索 |
| `mcp__blender__download_sketchfab_model` | モデルインポート |
| `mcp__blender__execute_blender_code` | Blender操作 |

## 実行フロー

```
1. 要件確認（何を作るか、スタイル、用途）
2. 戦略選択:
   - シンプル → AI一発生成
   - 複雑な機械 → パーツ分割生成
   - 高品質必要 → Sketchfab検索
3. プロンプト作成（best-practices.md参照）
4. 生成・調整
5. スケール調整（ゲーム用サイズ）
```

## クイックリファレンス

### プロンプト必須要素

```
[メインオブジェクト], [スタイル] style,
[素材] materials, [構造ヒント],
[用途], single object
```

### スタイル語彙

| 日本語 | 英語 |
|--------|------|
| 産業的 | industrial |
| スチームパンク | steampunk |
| SF | sci-fi, futuristic |
| ローポリ | low-poly, stylized |
| リアル | realistic, photorealistic |
| ファンタジー | fantasy, medieval |
| ミニマル | minimalist, clean |

### 素材語彙

| 日本語 | 英語 |
|--------|------|
| 金属 | metal, steel, iron |
| 錆びた | rusty, weathered |
| 木 | wood, wooden |
| 石 | stone, rock |
| 光沢 | glossy, shiny |
| マット | matte, dull |

### ゲーム用サイズ目安

| オブジェクト | target_size |
|--------------|-------------|
| 小物（カップ等） | 0.1 - 0.3 |
| 椅子・小型機械 | 0.5 - 1.0 |
| テーブル・中型機械 | 1.0 - 2.0 |
| 人・大型機械 | 1.5 - 2.5 |
| 車両・建物 | 3.0+ |

## 詳細

→ `best-practices.md` 参照
