# 3Dモデル生成

Blenderでローポリ3Dモデルを生成する。

## 引数
$ARGUMENTS

## 引数の解析

- **モデル名**: 必須（例: "pickaxe", "hammer"）
- **カテゴリ**: item/machine/structure（デフォルト: item）
- **色指定**: オプション（赤/blue/copper/#FF5500等）

---

## 実行手順

### 1. スクリプト作成

`tools/blender_scripts/{name}.py` に以下の構造で作成:

```python
import bpy
from mathutils import Vector
from math import pi, cos, sin
import os

# === 関数定義（必須：MCPでは各実行が独立するため） ===
def clear_scene():
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete()

def create_octagonal_prism(radius, height, location, name):
    verts = []
    for i in range(8):
        angle = i * pi / 4 + pi / 8
        verts.append((cos(angle) * radius, sin(angle) * radius, -height / 2))
        verts.append((cos(angle) * radius, sin(angle) * radius, height / 2))
    faces = []
    for i in range(8):
        j = (i + 1) % 8
        faces.append((i * 2, j * 2, j * 2 + 1, i * 2 + 1))
    faces.append(tuple(i * 2 for i in range(8)))
    faces.append(tuple(i * 2 + 1 for i in reversed(range(8))))
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    obj.location = Vector(location)
    bpy.context.collection.objects.link(obj)
    return obj

def create_chamfered_cube(size, chamfer, location, name):
    sx, sy, sz = [s / 2 for s in size]
    c = chamfer if chamfer else min(size) * 0.1
    verts = [
        (-sx + c, -sy, -sz), (sx - c, -sy, -sz), (sx, -sy + c, -sz), (sx, sy - c, -sz),
        (sx - c, sy, -sz), (-sx + c, sy, -sz), (-sx, sy - c, -sz), (-sx, -sy + c, -sz),
        (-sx + c, -sy, sz), (sx - c, -sy, sz), (sx, -sy + c, sz), (sx, sy - c, sz),
        (sx - c, sy, sz), (-sx + c, sy, sz), (-sx, sy - c, sz), (-sx, -sy + c, sz),
    ]
    faces = [
        (0, 1, 2, 3, 4, 5, 6, 7), (15, 14, 13, 12, 11, 10, 9, 8),
        (0, 8, 9, 1), (1, 9, 10, 2), (2, 10, 11, 3), (3, 11, 12, 4),
        (4, 12, 13, 5), (5, 13, 14, 6), (6, 14, 15, 7), (7, 15, 8, 0),
    ]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    obj.location = Vector(location)
    bpy.context.collection.objects.link(obj)
    return obj

def create_mat(name, color, metallic=0.0, roughness=0.8):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    for node in mat.node_tree.nodes:
        if node.type == 'BSDF_PRINCIPLED':
            node.inputs["Base Color"].default_value = (*color, 1)
            node.inputs["Metallic"].default_value = metallic
            node.inputs["Roughness"].default_value = roughness
            break
    return mat

def apply_mat(obj, mat):
    obj.data.materials.append(mat) if not obj.data.materials else obj.data.materials.__setitem__(0, mat)

# === シーンクリア ===
clear_scene()

# === モデル生成 ===
parts = []
# ... パーツ作成 ...

# === 結合 ===
bpy.ops.object.select_all(action='DESELECT')
for obj in parts:
    obj.select_set(True)
bpy.context.view_layer.objects.active = parts[0]
bpy.ops.object.join()
result = bpy.context.active_object
result.name = "ModelName"

# === エクスポート ===
output_dir = "/home/bacon/github/idle_factory/assets/models/items"
os.makedirs(output_dir, exist_ok=True)
bpy.ops.export_scene.gltf(
    filepath=os.path.join(output_dir, "name.gltf"),
    export_format='GLTF_SEPARATE',
    use_selection=True,
    export_materials='EXPORT',
    export_yup=True,
)
print("Exported!")
```

### 2. 生成実行（2つの方法）

**方法A: Blender MCP経由**（推奨）
- `mcp__blender__execute_blender_code` でスクリプト全体を1回で実行
- 注意: 関数定義を毎回含める必要あり

**方法B: バックグラウンドBlender**
```bash
DISPLAY=:10 blender --background --python tools/blender_scripts/{name}.py
```

### 3. 検証
```bash
# f3dでプレビュー
DISPLAY=:10 f3d --camera-azimuth-angle=45 --output screenshots/{name}.png assets/models/items/{name}.gltf
```

---

## カテゴリ別仕様

| カテゴリ | サイズ | 三角形 | 原点 |
|---------|--------|--------|------|
| item | 0.2-0.3 | 50-300 | center |
| machine | 0.9-1.0 | 200-800 | bottom |
| structure | 1.0+ | 300-1500 | bottom |

## マテリアルプリセット

| 名前 | RGB | 用途 |
|------|-----|------|
| iron | (0.29, 0.29, 0.29) | 鉄 |
| copper | (0.72, 0.45, 0.20) | 銅 |
| wood | (0.55, 0.41, 0.08) | 木 |
| dark_steel | (0.18, 0.18, 0.18) | 鋼 |

## 出力先
- glTF: `assets/models/{category}s/{name}.gltf`
- スクリプト: `tools/blender_scripts/{name}.py`
