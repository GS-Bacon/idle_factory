# 3Dモデル生成（Blender MCP / AI）

AI 3D生成を使ったモデル作成。複雑な機械や高品質モデルに対応。

## 引数
$ARGUMENTS

## 引数の解析

- **説明**: 必須（例: "採掘機", "コンベア", "チェスト"）
- **--style**: スタイル指定（industrial/steampunk/sci-fi/fantasy/low-poly）
- **--method**: 生成方法（auto/hyper3d/sketchfab/split）
- **--size**: ゲーム内サイズ（0.1-10.0、デフォルト: 1.0）

---

## ベストプラクティス参照

**必ず読む**: `.claude/skills/modeling/best-practices.md`

### クイックリファレンス

| 複雑さ | 推奨method |
|--------|------------|
| シンプルな小物 | `hyper3d` |
| 中程度 | `hyper3d` + 調整 |
| **複雑な機械** | `split`（分割生成） |
| 高品質必須 | `sketchfab` |

### AI生成の得意・不得意

| カテゴリ | 品質 |
|----------|------|
| シンプルな小物 | ⭐⭐⭐⭐ |
| スタイライズドキャラ | ⭐⭐⭐ |
| **産業機械・メカ** | ⭐⭐（苦手） |
| 有機的形状 | ⭐ |

---

## 実行フロー

### Step 0: デザインプレビュー（推奨）

**いきなりBlenderで作らない！** まずHTMLでデザインを確定する。

```
/review-models [モデル名] --style [スタイル]
```

1. 参考画像を検索
2. HTMLプレビューを作成（Three.js）
3. Tailscale経由でブラウザ確認
4. パラメータ調整 → JSON設定をコピー
5. 確定した設定でBlenderモデリング

**メリット:**
- リアルタイムでプロポーション確認
- やり直しが簡単
- 設定値を正確に再現可能

### Step 1: 戦略決定

```
シンプル？ → hyper3d一発
複雑な機械？ → split（パーツ分割）
品質重視？ → sketchfab検索
手動モデリング？ → HTMLプレビュー → Blender Python
```

### Step 2: プロンプト作成（hyper3d/splitの場合）

**必須要素チェックリスト:**
- [ ] メインオブジェクト（英語）
- [ ] スタイル（industrial/steampunk/sci-fi等）
- [ ] 素材（metal/iron/wood等）
- [ ] 構造ヒント（cylindrical/boxy等）
- [ ] 用途（game-ready/low-poly等）
- [ ] `single object`（複数オブジェクト防止）

**プロンプトテンプレート:**
```
[オブジェクト名], [スタイル] style,
[素材1] and [素材2] materials,
[形状/構造の説明],
game-ready, single object, clean topology
```

**採掘機の例（分割）:**
```
# パーツ1: ベース
industrial machine base, steampunk style,
dark steel and iron materials, boxy shape with rivets,
game-ready low-poly, single object

# パーツ2: ドリル
industrial drill head, steampunk style,
brass and iron spiral bit, cylindrical,
game-ready, single object

# パーツ3: アーム
mechanical arm with hydraulic pistons,
iron and copper joints,
game-ready, single object
```

### Step 3: 生成実行

#### Hyper3D（テキストから生成）
```python
mcp__blender__generate_hyper3d_model_via_text(
    text_prompt="[プロンプト]",
    bbox_condition=[1, 1, 1]  # オプション: 縦横比
)
# → subscription_key取得

mcp__blender__poll_rodin_job_status(subscription_key="...")
# → "Done"になるまでポーリング

mcp__blender__import_generated_asset(
    name="miner",
    task_uuid="..."
)
```

#### Sketchfab（検索してダウンロード）
```python
mcp__blender__search_sketchfab_models(
    query="mining machine industrial",
    downloadable=True
)
# → 結果からuid選択

mcp__blender__get_sketchfab_model_preview(uid="...")
# → プレビュー確認

mcp__blender__download_sketchfab_model(
    uid="...",
    target_size=1.5  # ゲーム内サイズ
)
```

### Step 4: 調整

```python
# スケール調整
mcp__blender__execute_blender_code(code="""
import bpy
obj = bpy.context.active_object
obj.scale = (0.5, 0.5, 0.5)
bpy.ops.object.transform_apply(scale=True)
""")

# 原点を底面中央に
mcp__blender__execute_blender_code(code="""
import bpy
bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
""")
```

### Step 5: エクスポート

```python
mcp__blender__execute_blender_code(code="""
import bpy
bpy.ops.export_scene.gltf(
    filepath='/home/bacon/idle_factory/assets/models/machines/miner.glb',
    export_format='GLB'
)
""")
```

---

## スタイル語彙

| 日本語 | 英語プロンプト |
|--------|----------------|
| 産業的 | industrial, factory |
| スチームパンク | steampunk, brass gears |
| SF・未来的 | sci-fi, futuristic, neon |
| ファンタジー | fantasy, medieval, magical |
| ローポリ | low-poly, stylized, blocky |
| リアル | realistic, photorealistic |
| ミニマル | minimalist, clean, simple |

## 素材語彙

| 日本語 | 英語プロンプト |
|--------|----------------|
| 鉄・スチール | iron, steel, dark steel |
| 銅 | copper, bronze |
| 真鍮 | brass |
| 錆びた | rusty, weathered, worn |
| 木 | wood, wooden |
| 石 | stone, rock |
| 光沢 | glossy, shiny, polished |

## ゲームサイズ目安

| オブジェクト | target_size |
|--------------|-------------|
| 小物（カップ等） | 0.1 - 0.3 |
| アイテム | 0.3 - 0.5 |
| 椅子・小型機械 | 0.5 - 1.0 |
| テーブル・中型機械 | 1.0 - 2.0 |
| 大型機械 | 1.5 - 2.5 |
| 車両・建物 | 3.0+ |

---

## 分割生成パターン（複雑な機械用）

### 採掘機
```
1. ベース/車体: "machine base, boxy metal body"
2. ドリル: "drill head, spiral bit"
3. アーム: "mechanical arm, pistons"
4. → Blenderで結合
```

### コンベア
```
1. フレーム: "conveyor frame, metal rails"
2. ベルト: "rubber belt, flat"
3. → Blenderで結合
```

### 発電機
```
1. 本体: "generator body, cylindrical"
2. タービン: "turbine blades"
3. 配管: "pipes and tubes"
4. → Blenderで結合
```

---

## トラブルシューティング

| 問題 | 原因 | 解決策 |
|------|------|--------|
| 形が崩れる | プロンプトが曖昧 | スタイル・素材を具体的に |
| 複数オブジェクト | "single object"未指定 | 明示的に追加 |
| ディテール不足 | Quality低い | Detail/HighPack使用 |
| 機械が不正確 | AI限界 | split or sketchfab |
| タイムアウト | 処理重い | 分割生成 |

---

## VOXベース生成との使い分け

| 方法 | 用途 | コマンド |
|------|------|----------|
| **Blender MCP/AI** | 複雑な形状、リアル系 | `/generate-model` |
| **VOXベース** | ボクセルスタイル、シンプル | `/generate-asset` |

ゲームのアートスタイルに合わせて選択。
