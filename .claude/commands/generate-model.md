# 3Dモデル生成

Blenderでローポリ3Dモデルを生成する。

## 引数
$ARGUMENTS

## 引数の解析

- **モデル名**: 必須（例: "pickaxe", "hammer"）
- **カテゴリ**: item/machine/structure（デフォルト: item）
- **色指定**: オプション（赤/blue/copper/#FF5500等）

---

## ★ローポリデザイン原則（重要）★

### 採用スタイル: Minecraft/Unturned + Astroneerハイブリッド

| 要素 | 手法 | 根拠 |
|------|------|------|
| **形状** | ブロック感、明確なシルエット | Minecraft/Unturned風 |
| **カラー** | テクスチャレス、マテリアルカラーのみ | Astroneer風（制作効率UP） |
| **シェーディング** | フラット + 頂点カラーでエッジ暗化 | 共通原則 |
| **プリミティブ** | 八角形、面取りキューブ、台形 | 円形禁止 |

### シェーディング設定（必須）
```python
# フラットシェーディング + Auto Smooth 30°
for obj in bpy.data.objects:
    if obj.type == 'MESH':
        obj.data.use_auto_smooth = True
        obj.data.auto_smooth_angle = 0.523599  # 30度
        bpy.ops.object.shade_flat()
```

### 頂点カラーによるエッジ暗化（推奨）
```python
# エッジ付近を85%暗くして立体感を出す
def apply_edge_darkening(obj, edge_factor=0.85):
    """隣接面が少ない頂点を暗くする"""
    if not obj.data.vertex_colors:
        obj.data.vertex_colors.new()
    # 実装は_base.pyのapply_vertex_color_shading参照
```

### ポリゴン予算（調査結果準拠）

| カテゴリ | 推奨三角形数 | 最大 | 参考 |
|----------|-------------|------|------|
| 手持ちアイテム | 50-200 | 500 | Crossy Road: 極小 |
| 小道具 | 20-100 | 200 | - |
| 1ブロック機械 | 200-800 | 1500 | Astroneer: テクスチャレス |
| 大型機械 | 500-2000 | 4000 | - |
| キャラクター | 500-1000 | 3500 | TABS風 |

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

| 名前 | RGB | Metallic | Roughness | 用途 |
|------|-----|----------|-----------|------|
| iron | (0.29, 0.29, 0.29) | 1.0 | 0.5 | 鉄製パーツ |
| copper | (0.72, 0.45, 0.20) | 1.0 | 0.4 | 配線、熱交換器 |
| brass | (0.79, 0.64, 0.15) | 1.0 | 0.4 | ギア、装飾 |
| dark_steel | (0.18, 0.18, 0.18) | 1.0 | 0.6 | 重機、産業機械 |
| wood | (0.55, 0.41, 0.08) | 0.0 | 0.8 | ハンドル、サポート |
| stone | (0.41, 0.41, 0.41) | 0.0 | 0.7 | 基礎、炉 |

### アクセントカラー（機能表示用）
| 名前 | Hex | 用途 |
|------|-----|------|
| danger | #CC3333 | 危険、高温 |
| warning | #CCAA33 | 警告 |
| power | #3366CC | 電力 |
| active | #33CC66 | 稼働中 |

## 出力先
- glTF: `assets/models/{category}s/{name}.gltf`
- スクリプト: `tools/blender_scripts/{name}.py`

## 参考ゲーム（調査済み）

詳細は `.specify/memory/lowpoly-style-research.md` 参照

| ゲーム | 特徴 | 本プロジェクトへの適用 |
|--------|------|----------------------|
| **Astroneer** | テクスチャレス、マテリアルカラーのみ | ✅ 採用 |
| **Unturned** | 極端にシンプルなブロック形状 | ✅ 採用 |
| **Valheim** | 低解像度テクスチャ拡大（ピクセル境界維持） | 参考 |
| **A Short Hike** | 低解像度レンダリング + フラットシェーディング | 参考 |
| **Superhot** | 3色のみ、極限ミニマリズム | 参考 |
| **TABS** | ウォブリー物理、ポップな色使い | キャラクター参考 |
| **Crossy Road** | ボクセル、極小テクスチャ | 小物参考 |

---

## 自動品質改善モード（--training）

`--training` オプションで自動品質改善ループを有効化。

### 概要

```
生成 → 評価 → 改善提案 → 再生成 → ... → 閾値達成 → エクスポート
```

スコアが閾値（デフォルト7.5）に達するまで自動でイテレーション。

### 使用方法

```bash
# トレーニングモードで生成
/generate-model axe --training

# 課題を指定して実行
/generate-model tool_wrench --training --challenge
```

### 評価基準（スタイル準拠重視）

| 基準 | 重み | 内容 |
|------|------|------|
| ratios | 25% | 比率準拠（ハンドル60-70%等） |
| primitives | 20% | 許可プリミティブのみ |
| materials | 15% | MATERIALSプリセットのみ |
| triangle_budget | 15% | ポリゴン予算内 |
| connectivity | 10% | パーツ接続 |
| origin | 10% | 原点位置 |
| edge_darkening | 5% | エッジ暗化 |

### イテレーション制御

- **最大イテレーション**: 5回
- **成功閾値**: 7.5（課題により変動）
- **早期終了**: スコア9.0以上
- **停滞検出**: 3回連続で改善0.5未満

### 出力例

```
=== Training Mode: axe ===
Challenge: Axe (difficulty: 2)

Iteration 1: Score 6.2/10
  - ratios: 5.0 (handle too short)
  - primitives: 10.0
  - materials: 8.0 (custom wood color)

Generating improvement feedback...

Iteration 2: Score 7.8/10
  - ratios: 8.0 (improved)
  - primitives: 10.0
  - materials: 10.0 (fixed)

SUCCESS: Score 7.8 >= threshold 7.5
Exported: assets/models/items/axe.gltf

Score History:
█▓
6.2 → 7.8 (+1.6)
```

### 視覚確認ループ（重要）

**数値評価だけでは不十分**。生成後は必ず視覚的に確認する。

```
生成 → スクリーンショット取得 → 視覚確認 → 問題あれば修正 → 再生成
```

#### 視覚確認の実行手順

1. **スクリーンショット取得**
   ```
   mcp__blender__get_viewport_screenshot()
   ```

2. **視覚的チェック項目**
   - [ ] パーツが浮いていないか（物理的に接続されているか）
   - [ ] 向きが正しいか（刃は前方/上方を向いているか）
   - [ ] 全体のシルエットが「それらしい」か
   - [ ] 比率が自然か（頭でっかち、細すぎ等でないか）

3. **問題発見時**
   - 具体的な修正を特定（例: "blade のZ位置を0.003下げる"）
   - コードを修正して再生成
   - 再度スクリーンショットで確認

#### パーツ配向ガイド

| パーツタイプ | 期待する主軸 | 向き |
|-------------|-------------|------|
| handle | Z軸（縦） | 底から上に伸びる |
| blade（斧） | X軸（横） | 前方を向く |
| blade（ピッケル） | X軸（横） | 前方を向く |
| head（ハンマー） | Y軸（横） | 左右に広がる |
| shaft | Z軸（縦） | 上下に伸びる |

#### 接続確認

全パーツのバウンディングボックスが0.003以上重なっていることを確認:
```python
def check_connectivity():
    mesh_objects = [o for o in bpy.context.scene.objects if o.type == 'MESH']
    for i, obj1 in enumerate(mesh_objects):
        connected = False
        for j, obj2 in enumerate(mesh_objects):
            if i != j and bboxes_overlap(obj1, obj2, tolerance=0.003):
                connected = True
                break
        if not connected:
            print(f"WARNING: {obj1.name} is floating!")
```

### 課題定義ファイル

課題は `tools/model_training/challenges.yaml` で定義:

```yaml
- id: tool_axe
  name: "Axe"
  category: tool
  difficulty: 2
  constraints:
    primitives: [octagon, chamfered_cube, trapezoid]
    materials: [wood, iron]
    triangle_budget: [80, 200]
  ratios:
    handle_ratio: {min: 0.60, max: 0.70}
    head_width_ratio: {min: 4.0, max: 6.0}
  success_threshold: 7.5
```

### 学習データベース

成功/失敗パターンは `tools/model_training_data/learning.json` に蓄積。
過去の成功パターンを参照して生成品質を向上。

### 関連ファイル

- `tools/model_training/rubric.py` - 評価ルーブリック
- `tools/model_training/evaluator.py` - 評価エンジン
- `tools/model_training/feedback_generator.py` - 改善提案生成
- `tools/model_training/iteration_controller.py` - ループ制御
- `tools/model_training/learning_db.py` - 学習データベース
- `tools/model_training/human_feedback.py` - 人間フィードバック記録

---

## 人間フィードバックモード（--feedback）

生成後に人間の評価を記録し、将来の生成に活用する。

### ワークフロー

```
1. モデル生成
2. スクリーンショット撮影 + glTFエクスポート
3. 人間に評価を依頼（5段階 x 5項目）
4. フィードバックを記録
5. 必要なら修正して再生成
```

### 必須：生成後の出力

モデル生成後は**必ず**以下を実行：

```python
# 1. スクリーンショット撮影（Cycles推奨）
bpy.context.scene.render.engine = 'CYCLES'
bpy.context.scene.cycles.samples = 128
bpy.context.scene.render.filepath = f"/home/bacon/github/idle_factory/screenshots/{model_name}.png"
bpy.ops.render.render(write_still=True)

# 2. glTFエクスポート
bpy.ops.export_scene.gltf(
    filepath=f"/home/bacon/github/idle_factory/assets/models/{category}s/{model_name}.gltf",
    export_format='GLTF_SEPARATE',
    use_selection=True,
    export_materials='EXPORT',
    export_yup=True,
)
```

### 評価項目（1-5）

| 項目 | 説明 |
|------|------|
| shape | 形状：そのモデルらしさ、シルエット |
| style | スタイル：ローポリ感、ゲームに合うか |
| detail | ディテール：パーツのバランス |
| color | 色/マテリアル：色味、金属感 |
| overall | 総合：ゲームで使いたいか |

### フィードバック記録

```python
from tools.model_training.human_feedback import get_feedback_db, ModelGeneration

db = get_feedback_db()

# 生成を記録
gen = ModelGeneration(
    model_name="pipe_straight",
    category="machine",
    parameters={"pipe_r": 0.12, "bolt_position": "inner"},
    screenshot_path="screenshots/pipe_straight.png",
    export_path="assets/models/machines/pipe_straight.gltf",
)
gen_id = db.record_generation(gen)

# 人間評価を追加
db.add_feedback(gen_id, {
    "shape": 4,
    "style": 5,
    "detail": 4,
    "color": 4,
    "overall": 4,
    "comments": "ボルトは内側に統一",
    "issues": ["bolt_position"],
    "fixes_applied": ["両方内側に変更"],
})
```

### 過去の学習を活用

```python
# ガイダンス取得
guidance = db.get_guidance_for_model("pipe_elbow", "machine")
print(guidance["successful_patterns"])  # 成功パターン
print(guidance["issues_to_avoid"])  # 避けるべき問題

# レポート生成
print(db.generate_report())
```

### データ保存先

- `tools/model_training_data/human_feedback.json`
