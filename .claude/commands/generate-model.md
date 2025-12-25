# 3Dモデル生成

Blenderでローポリ3Dモデルを生成する。

## 引数
$ARGUMENTS

## 引数の解析

引数から以下を抽出:
- **モデル名**: 必須（例: "pickaxe", "hammer", "conveyor"）
- **カテゴリ**: item/machine/structure（デフォルト: item）
- **色指定**: オプション（例: "赤", "青", "緑", "#FF5500", "copper"）

**色指定の形式**:
- 日本語色名: 赤, 青, 緑, 黄, 紫, オレンジ, 白, 黒, 灰色, 茶色, ピンク, 水色, 金, 銀
- 英語色名: red, blue, green, yellow, purple, orange, white, black, gray, brown, pink, cyan, gold, silver
- HEXコード: #RRGGBB（例: #FF5500）
- プリセット名: iron, copper, brass, dark_steel, wood, stone

**色変換テーブル**:
```python
COLOR_MAP = {
    # 日本語
    "赤": (0.8, 0.1, 0.1), "青": (0.1, 0.3, 0.8), "緑": (0.1, 0.6, 0.1),
    "黄": (0.9, 0.8, 0.1), "紫": (0.6, 0.1, 0.6), "オレンジ": (0.9, 0.4, 0.1),
    "白": (0.9, 0.9, 0.9), "黒": (0.1, 0.1, 0.1), "灰色": (0.5, 0.5, 0.5),
    "茶色": (0.4, 0.2, 0.1), "ピンク": (0.9, 0.5, 0.6), "水色": (0.4, 0.7, 0.9),
    "金": (0.83, 0.69, 0.22), "銀": (0.75, 0.75, 0.75),
    # 英語
    "red": (0.8, 0.1, 0.1), "blue": (0.1, 0.3, 0.8), "green": (0.1, 0.6, 0.1),
    "yellow": (0.9, 0.8, 0.1), "purple": (0.6, 0.1, 0.6), "orange": (0.9, 0.4, 0.1),
    "white": (0.9, 0.9, 0.9), "black": (0.1, 0.1, 0.1), "gray": (0.5, 0.5, 0.5),
    "brown": (0.4, 0.2, 0.1), "pink": (0.9, 0.5, 0.6), "cyan": (0.4, 0.7, 0.9),
    "gold": (0.83, 0.69, 0.22), "silver": (0.75, 0.75, 0.75),
    # プリセット（既存）
    "iron": (0.29, 0.29, 0.29), "copper": (0.72, 0.45, 0.20),
    "brass": (0.79, 0.64, 0.15), "dark_steel": (0.18, 0.18, 0.18),
    "wood": (0.55, 0.41, 0.08), "stone": (0.41, 0.41, 0.41),
}
```

**使用例**:
- `/generate-model 赤いピッケル` → ピッケルを赤色で生成
- `/generate-model 青い剣` → 剣を青色で生成
- `/generate-model copper conveyor` → コンベアを銅色で生成
- `/generate-model #FF5500 ハンマー` → ハンマーをオレンジ色で生成

## 実行手順

### 1. リファレンス読み込み（必須・最初に実行）
`.specify/memory/modeling-compact.md` を読む

### 2. スクリプト作成

**パーツ接続の鉄則**（最重要）:
```python
# ❌ 間違い: パーツを独立した位置に配置
head_z = 0.17
handle_top = 0.15  # 隙間ができる！

# ✅ 正解: 基準パーツから相対位置で計算
head_z = 0.17
handle_top = head_z + 0.01  # ヘッド中心を超える（貫通）
handle_length = handle_top - handle_bottom
```

**色指定がある場合**:
```python
# 色変換関数（スクリプト冒頭に追加）
def parse_color(color_spec):
    """色指定を(R, G, B)タプルに変換"""
    COLOR_MAP = {
        # 日本語
        "赤": (0.8, 0.1, 0.1), "青": (0.1, 0.3, 0.8), "緑": (0.1, 0.6, 0.1),
        "黄": (0.9, 0.8, 0.1), "紫": (0.6, 0.1, 0.6), "オレンジ": (0.9, 0.4, 0.1),
        "白": (0.9, 0.9, 0.9), "黒": (0.1, 0.1, 0.1), "灰色": (0.5, 0.5, 0.5),
        "茶色": (0.4, 0.2, 0.1), "ピンク": (0.9, 0.5, 0.6), "水色": (0.4, 0.7, 0.9),
        "金": (0.83, 0.69, 0.22), "銀": (0.75, 0.75, 0.75),
        # 英語
        "red": (0.8, 0.1, 0.1), "blue": (0.1, 0.3, 0.8), "green": (0.1, 0.6, 0.1),
        "yellow": (0.9, 0.8, 0.1), "purple": (0.6, 0.1, 0.6), "orange": (0.9, 0.4, 0.1),
        "white": (0.9, 0.9, 0.9), "black": (0.1, 0.1, 0.1), "gray": (0.5, 0.5, 0.5),
        "brown": (0.4, 0.2, 0.1), "pink": (0.9, 0.5, 0.6), "cyan": (0.4, 0.7, 0.9),
        "gold": (0.83, 0.69, 0.22), "silver": (0.75, 0.75, 0.75),
        # プリセット
        "iron": (0.29, 0.29, 0.29), "copper": (0.72, 0.45, 0.20),
        "brass": (0.79, 0.64, 0.15), "dark_steel": (0.18, 0.18, 0.18),
        "wood": (0.55, 0.41, 0.08), "stone": (0.41, 0.41, 0.41),
    }
    if color_spec in COLOR_MAP:
        return COLOR_MAP[color_spec]
    if color_spec.startswith("#") and len(color_spec) == 7:
        r = int(color_spec[1:3], 16) / 255
        g = int(color_spec[3:5], 16) / 255
        b = int(color_spec[5:7], 16) / 255
        return (r, g, b)
    return (0.5, 0.5, 0.5)  # デフォルトグレー

# 色を適用
MAIN_COLOR = parse_color("指定された色")  # 例: "赤", "blue", "#FF5500"
```

**テンプレート**:
```python
exec(open("tools/blender_scripts/_base.py").read())
clear_scene()
parts = []

# 色指定（オプション）
MAIN_COLOR = (0.8, 0.1, 0.1)  # 例: 赤色。指定がなければironなどのプリセットを使用

# 1. 基準となるパーツの位置を決める
main_z = 0.17

# 2. 他パーツは基準から相対的に計算（貫通/重なりで接続）
sub_top = main_z + overlap  # 基準を超える = 貫通
sub_length = sub_top - sub_bottom
sub_center = sub_bottom + sub_length / 2

# 3. パーツ生成
main = create_chamfered_cube(size, location=(0, 0, main_z), name="Main")
sub = create_octagonal_prism(radius, sub_length, location=(0, 0, sub_center), name="Sub")
parts.extend([main, sub])

# 4. 結合・マテリアル・エクスポート
result = join_all_meshes(parts, "ModelName")
# マテリアル適用（Blender 4.0対応）- 色指定対応版
mat = bpy.data.materials.new("Mat")
mat.use_nodes = True
bsdf = mat.node_tree.nodes.get("Principled BSDF")
if bsdf:
    bsdf.inputs["Base Color"].default_value = (*MAIN_COLOR, 1)  # RGBAで指定
result.data.materials.append(mat)

bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
result.location = (0, 0, 0)
export_gltf("assets/models/items/name.gltf")
```

### 3. 生成実行
```bash
DISPLAY=:10 blender --background --python tools/blender_scripts/{name}.py
```

### 4. スクリーンショット検証（必須・一括実行）

**3方向から同時に撮影**:
```bash
DISPLAY=:10 f3d --camera-azimuth-angle=45 --camera-elevation-angle=30 --output screenshots/{name}_angle.png assets/models/{cat}s/{name}.gltf &
DISPLAY=:10 f3d --camera-azimuth-angle=0 --camera-elevation-angle=0 --output screenshots/{name}_front.png assets/models/{cat}s/{name}.gltf &
DISPLAY=:10 f3d --camera-azimuth-angle=90 --camera-elevation-angle=0 --output screenshots/{name}_side.png assets/models/{cat}s/{name}.gltf &
wait
```

**確認ポイント**:
- [ ] パーツ間に隙間がないか（浮いていないか）
- [ ] 全パーツが接続されているか
- [ ] 形状が意図通りか

### 5. 問題があれば修正

接続問題の修正パターン:
```python
# パーツAとBが離れている場合
# → Bの位置をAから相対的に計算し直す

# 例: ヘッドとハンドルが離れている
# 修正前: handle_length = 0.15 (固定値)
# 修正後: handle_top = head_z + 0.01; handle_length = handle_top - handle_bottom
```

## 寸法ガイド

| カテゴリ | サイズ | 三角形 | 原点 |
|---------|--------|--------|------|
| item | 0.2-0.3 | 50-300 | center |
| machine | 0.9-1.0 | 200-800 | bottom_center |

## マテリアル
`iron`(#4A4A4A), `copper`(#B87333), `brass`(#C9A227), `dark_steel`(#2D2D2D), `wood`(#8B6914), `stone`(#696969)

## 出力先
- glTF: `assets/models/{category}s/{name}.gltf`
- スクリプト: `tools/blender_scripts/{name}.py`
