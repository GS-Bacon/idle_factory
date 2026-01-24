# AI 3Dモデリング ベストプラクティス

## 1. AI生成の得意・不得意

| カテゴリ | 品質 | 備考 |
|----------|------|------|
| シンプルな小物 | ⭐⭐⭐⭐ | カップ、椅子など |
| スタイライズドキャラ | ⭐⭐⭐ | Low-polyなら良好 |
| **産業機械・メカ** | ⭐⭐ | **構造的精度が不足** |
| 有機的形状（人体等） | ⭐ | 破綻しやすい |

### なぜ機械が難しいか

1. **学習データの問題** - ネット上の3Dモデルは品質がバラバラ
2. **構造的整合性** - 動く部品、噛み合わせの理解が不十分
3. **詳細の欠落** - ネジ、パイプ、配線などの細部は再現困難

---

## 2. プロンプトの書き方

### 悪い例

```
採掘機を作って
```

### 良い例

```
Industrial mining drill machine, steampunk style,
metallic bronze and iron materials, cylindrical drill head,
compact body with visible gears, game-ready low-poly,
single object, clean topology
```

### 必須要素チェックリスト

| 要素 | 例 | 重要度 |
|------|-----|--------|
| **メインオブジェクト** | mining drill, excavator | 必須 |
| **スタイル** | steampunk, industrial, sci-fi | 必須 |
| **素材** | metal, iron, bronze, rusty steel | 必須 |
| **用途** | game-ready, low-poly, high-detail | 推奨 |
| **構造ヒント** | cylindrical body, visible gears | 推奨 |
| **単一指定** | single object | 推奨 |

### 避けるべきこと

- ❌ 複数オブジェクトを一度に（「机と椅子とランプ」）
- ❌ 詳細すぎる指定（5個以上の詳細）
- ❌ 矛盾した指定（「シンプルで複雑な」）
- ❌ 専門用語の乱用
- ❌ 抽象的な概念（具体的な視覚参照なし）

---

## 3. 複雑なオブジェクトの分割戦略

### 採掘機の例

**一発生成（失敗しやすい）:**
```
mining machine
```

**分割生成（推奨）:**
```
Step 1: "industrial machine base, boxy metal body, steel plates, low-poly"
Step 2: "industrial drill head, spiral metal bit, cylindrical, metallic"
Step 3: "mechanical arm, hydraulic pistons, metal joints"
Step 4: Blenderで結合・調整
```

### 分割の原則

1. **機能単位で分ける** - ベース、駆動部、作業部
2. **各パーツはシンプルに** - 1つの主要形状
3. **素材を統一** - 後で調整しやすい
4. **最後に手動調整** - 位置、スケール、接続

---

## 4. Hyper3D設定

| 設定 | 推奨値 | 用途 |
|------|--------|------|
| Quality | Medium (18k) | プロトタイプ |
| Quality | High (50k) | 本番用 |
| Mode | Rodin Detail | 機械・精密物 |
| HighPack | ON | 高品質テクスチャ |
| bbox_condition | [1,1,2] | 縦長オブジェクト |

### bbox_condition例

```python
# 横長（車両など）
bbox_condition = [2, 1, 0.5]

# 縦長（タワーなど）
bbox_condition = [1, 1, 3]

# 立方体（箱など）
bbox_condition = [1, 1, 1]
```

---

## 5. 代替戦略: Sketchfab

AI生成より既存モデルが適切な場合:

### 検索のコツ

```python
# 具体的なキーワード
query = "mining drill machine"  # ✓
query = "excavator industrial"  # ✓
query = "machine"  # ✗ 広すぎ

# フィルタ活用
downloadable = True  # 必須
```

### よく使うカテゴリ

| 用途 | 検索キーワード |
|------|----------------|
| 採掘機 | mining, drill, excavator |
| 工場機械 | factory, industrial, machinery |
| コンベア | conveyor, belt, transport |
| 炉 | furnace, smelter, forge |
| 発電機 | generator, power, turbine |

---

## 6. ワークフロー比較

### シンプルなオブジェクト

```
[Hyper3D一発生成] → [スケール調整] → 完了
```

### 中程度の複雑さ

```
[Hyper3D生成] → [Blenderで調整] → 完了
```

### 複雑な機械

```
[パーツ分割] → [各パーツ生成] → [Blender結合] → [手動調整] → 完了
```

### 高品質必須

```
[Sketchfab検索] → [ダウンロード] → [スケール調整] → 完了
```

---

## 7. トラブルシューティング

| 問題 | 原因 | 解決策 |
|------|------|--------|
| 形が崩れる | プロンプトが曖昧 | 具体的なスタイル・素材追加 |
| 複数オブジェクト化 | "single object"未指定 | 明示的に指定 |
| ディテール不足 | Quality設定が低い | High/Detail使用 |
| サイズがおかしい | target_size未指定 | 適切なサイズ指定 |
| タイムアウト | 処理が重い | 分割生成 |
| 機械が不正確 | AI限界 | Sketchfab使用 or 分割 |

---

## 8. ゲーム用チェックリスト

### 生成後の確認

- [ ] サイズが適切か（他オブジェクトとの比較）
- [ ] ポリゴン数が許容範囲か
- [ ] テクスチャが適用されているか
- [ ] 原点位置が適切か
- [ ] 回転が正しいか（Y-up）

### 修正コマンド例

```python
# スケール調整
bpy.ops.transform.resize(value=(0.5, 0.5, 0.5))

# 原点をジオメトリ中心に
bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY')

# Y-up回転修正
obj.rotation_euler = (math.radians(90), 0, 0)
```

---

## 9. HTMLプレビュー → Blenderコード生成（推奨）

### 概要

最も効率的なワークフロー。HTMLでデザインを確定し、「Blender Code」ボタンでPythonコードを自動生成。

### フロー

```
[HTMLプレビュー] → [パラメータ調整] → [Blender Codeボタン] → [MCP実行] → [GLBエクスポート]
```

### Step 1: HTTPサーバー起動

```bash
cd /home/bacon/idle_factory/UIプレビュー
python3 -m http.server 8080 --bind 0.0.0.0 &

# アクセス: http://100.84.170.32:8080/mining_drill_preview.html
```

### Step 2: ブラウザでデザイン調整

- スライダーでパラメータ調整
- プリセットボタンでスタイル切替
- リアルタイムで3Dプレビュー確認
- 1ブロック境界線でサイズ確認

### Step 3: Blenderコード生成

HTMLの2つのボタン:

| ボタン | 機能 |
|--------|------|
| **Blender Code** | Pythonコードのみ生成 |
| **+ Export GLB** | コード + GLBエクスポート付き |

クリックすると:
1. モデル名を入力（例: miner）
2. Blender Pythonコードが自動生成
3. クリップボードにコピー

### Step 4: Blender MCPで実行

```python
mcp__blender__execute_blender_code(
    code="[クリップボードのコード]",
    user_prompt="採掘機モデル作成"
)
```

### 生成されるコード構造

```python
import bpy
import math

# === 設定値（HTMLプレビューから自動生成） ===
config = {
    "bodyWidth": 0.26,
    "bodyHeight": 0.18,
    ...
}

# マテリアル作成
# シーンクリア
# Body, Motor, Indicator, Outlet, Legs, Shaft, Drill 作成
# GLBエクスポート（オプション）
```

### 利点

| 利点 | 説明 |
|------|------|
| **即時プレビュー** | HTMLでリアルタイム確認 |
| **正確な再現** | パラメータ値がそのまま反映 |
| **やり直し簡単** | 値を変えて再実行するだけ |
| **コード確認可能** | 生成コードを編集可能 |
| **エクスポート統合** | GLB出力まで一発 |

### 既存プレビュー

| モデル | ファイル | 出力先 |
|--------|----------|--------|
| 採掘機 | `mining_drill_preview.html` | `assets/models/machines/miner.glb` |

---

## 10. Blender MCP Pythonモデリング（Claude直接操作）

### 基本原則

| 原則 | 詳細 |
|------|------|
| **段階的に作成** | 一度に全パーツを作らない。1パーツ作成→確認→次へ |
| **各ステップで確認** | レンダリングして視覚確認、問題あれば即修正 |
| **パーツを深く重ねる** | 接触ではなく「重なり」で一体感を出す |
| **本体から生やす** | ドリル等は本体に埋め込む形で接続 |

### 避けるべきパターン

| ❌ 悪い | ✅ 良い |
|---------|--------|
| パーツを接触させる | パーツを重ねる（overlap: 0.02-0.05） |
| 全パーツを一度に作成 | 1パーツずつ作成→確認 |
| 浮いた脚 | 本体に埋め込んだ脚 |
| 分離したドリル | 本体から生えているドリル |

### 段階的作成フロー

```
Step 1: 本体（メインボックス）作成 → レンダリング確認
Step 2: 本体にドリルを埋め込み → レンダリング確認
Step 3: 脚を本体に接続 → レンダリング確認
Step 4: ディテール追加 → レンダリング確認
Step 5: 全体調整 → 完成
```

### 重なり（Overlap）の実装

```python
# 悪い: 接触のみ
body_bottom = 0.5
drill_top = 0.5  # 接触点

# 良い: 重なりを持たせる
body_bottom = 0.5
drill_top = 0.52  # 本体に0.02埋め込み
```

### 一体感を出すコツ

1. **脚は本体の内側から生やす** - 本体の角ではなく、本体に埋め込む
2. **ドリルシャフトは本体を貫通** - 上から下まで一本で繋がっている感
3. **ディテールは面に密着** - 浮かせない
4. **色の連続性** - 隣接パーツは近い色

### Factorioデザイン哲学（参考）

- **平面的な正方形を避ける** - 3D感を出す
- **ドリルビットが主役** - 機能が一目でわかる
- **ファミリー感** - 同じ機械シリーズは統一感

---

## Sources

- [Blender MCP公式](https://blender-mcp.com/)
- [Blender MCP GitHub](https://github.com/ahujasid/blender-mcp)
- [Blender MCP Tutorials](https://blender-mcp.com/tutorials.html)
- [Factorio Friday Facts #350](https://factorio.com/blog/post/fff-350)
- [Hyper3D公式](https://hyper3d.ai/)
- [Meshy AI - Prompt Engineering](https://www.meshy.ai/blog/meshy-5-text-to-3d)
- [3D AI Studio Guide](https://www.3daistudio.com/3d-generator-ai-comparison-alternatives-guide/what-to-write-in-text-prompts-for-3d)
