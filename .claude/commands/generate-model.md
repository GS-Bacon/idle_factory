# 3Dモデル生成

Blenderでローポリ3Dモデルを生成する。

## 引数
$ARGUMENTS

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

**テンプレート**:
```python
exec(open("tools/blender_scripts/_base.py").read())
clear_scene()
parts = []

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
# マテリアル適用（Blender 4.0対応）
mat = bpy.data.materials.new("Mat")
mat.use_nodes = True
bsdf = mat.node_tree.nodes.get("Principled BSDF")
if bsdf:
    bsdf.inputs["Base Color"].default_value = (0.3, 0.3, 0.3, 1)
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
