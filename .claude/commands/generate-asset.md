# アセット生成（3Dモデル・テクスチャ・スプライト）

ゲーム用アセットを生成。VOXモデル、ブロックテクスチャ、アイテムスプライトに対応。

## 引数
$ARGUMENTS

## 引数の解析

- **アセット名**: 必須（例: "miner", "grass_top", "iron_ingot"）
- **カテゴリ**: model/texture/sprite（デフォルト: model）
  - `model`: 3Dモデル（VOX→GLB）
  - `texture`: ブロックテクスチャ（16x16 PNG）
  - `sprite`: アイテムスプライト（32x32 PNG）
- **サブカテゴリ**（modelの場合）: item/machine/conveyor
- **--iterate**: 自動改善ループを有効化（modelのみ）
- **--all**: 全アセットを再生成

---

## クイックコマンド

### 全アセット再生成
```bash
# 全アセット一括生成
./tools/generate_all_assets.sh

# 個別実行
python3 tools/voxel_generator.py all          # VOXファイル生成
./tools/convert_all_vox.sh                    # GLB変換
DISPLAY=:10 blender --background --python tools/generate_textures.py      # テクスチャ
DISPLAY=:10 blender --background --python tools/generate_item_sprites.py  # スプライト
```

### 部分的な生成
```bash
# アイテムモデルのみ
./tools/convert_all_vox.sh --items

# 機械モデルのみ
./tools/convert_all_vox.sh --machines

# コンベアモデルのみ
./tools/convert_all_vox.sh --conveyors

# 単一ファイル
./tools/convert_all_vox.sh assets/models/machines/miner.vox
```

---

## テクスチャ生成（16x16 PNG）

### 対応テクスチャ
| ファイル | 説明 |
|----------|------|
| grass_top | 草ブロック上面 |
| grass_side | 草ブロック側面 |
| dirt | 土ブロック |
| stone | 石ブロック |
| iron_ore | 鉄鉱石 |
| copper_ore | 銅鉱石 |
| coal_ore | 石炭鉱石 |
| sand | 砂ブロック |
| bedrock | 岩盤 |

### 新規テクスチャ追加
```python
# tools/generate_textures.py の textures に追加
textures = {
    "new_block": ([(R, G, B)], "pattern"),  # pattern: noise/solid/ore/bedrock/grass_side
}
```

### 出力先
`assets/textures/blocks/{name}.png`

---

## スプライト生成（32x32 PNG）

### 対応スプライト
| ファイル | 形状タイプ |
|----------|-----------|
| iron_ore, copper_ore, coal, stone | rock |
| iron_ingot, copper_ingot | ingot |
| miner, crusher, assembler | machine |
| conveyor | belt |
| furnace | furnace |
| storage | crate |
| generator | generator |
| inserter | arm |

### 新規スプライト追加
```python
# tools/generate_item_sprites.py の ITEMS に追加
ITEMS = {
    "new_item": ((R, G, B), "shape_type"),  # shape_type: rock/ingot/machine/belt/furnace/crate/generator/arm
}
```

### 出力先
`assets/textures/items/{name}.png`

---

## 3Dモデル生成（VOX → GLB）

### 自動改善ワークフロー

```
┌─────────────────────────────────────────────────────────┐
│  1. モデル生成（voxel_generator.py）                      │
│     ↓                                                    │
│  2. 自動評価（voxel_evaluator.py）                        │
│     ↓                                                    │
│  3. スコア確認                                            │
│     ├── ≥7.0 → 成功 → glb変換 → 完了                      │
│     └── <7.0 → 改善提案を適用 → 1に戻る（最大3回）          │
└─────────────────────────────────────────────────────────┘
```

---

## 実行手順

### Step 1: モデル定義を作成

```python
# tools/voxel_generator.py に関数を追加
def create_furnace() -> VoxelModel:
    model = VoxelModel(16, 16, 16)

    # ベース
    model.fill_box(1, 1, 0, 14, 14, 2, "stone")

    # 本体
    model.fill_box(2, 2, 2, 13, 13, 12, "furnace_body")

    # 開口部（前面）
    model.fill_box(5, 14, 3, 10, 15, 8, "dark_steel")

    # 煙突
    model.fill_box(6, 6, 12, 9, 9, 15, "iron")

    # 発光部
    model.fill_box(6, 14, 4, 9, 14, 7, "furnace_glow")

    # インジケータ
    model.fill_box(3, 2, 10, 4, 2, 11, "active")

    return model
```

### Step 2: 評価を実行

```python
from tools.voxel_generator import VoxelModel
from tools.voxel_evaluator import evaluate_model, format_evaluation

model = create_furnace()
result = evaluate_model(model.voxels, (16, 16, 16), "furnace", "machine")
print(format_evaluation(result))
```

### Step 3: 改善提案に従って修正

評価結果の`suggestions`に従ってコードを修正し、再評価。

### Step 4: 合格したらglb変換

```bash
python3 -c "from tools.voxel_generator import ...; model.save('furnace.vox')"
blender --background --python tools/vox_to_gltf.py -- furnace.vox furnace.glb
```

---

## 評価基準

| 基準 | 重み | 説明 | 合格ライン |
|------|------|------|-----------|
| symmetry | 15% | 左右対称性 | ≥7 |
| fill_ratio | 20% | 充填率（カテゴリ別） | ≥7 |
| color_variety | 15% | 色の多様性 | ≥7 |
| feature_presence | 25% | 特徴要素の有無 | ≥7 |
| proportions | 25% | 比率の適切さ | ≥7 |

**総合スコア ≥7.0 で合格**

---

## カテゴリ別の期待値

### item（ツール、インゴット等）
```
サイズ: 8x8x16
充填率: 15-40%
色数: 2-4色
特徴: ハンドル、ヘッド
比率: 縦長
```

### machine（採掘機、精錬炉等）
```
サイズ: 16x16x16
充填率: 30-70%
色数: 3-6色
特徴: ボディ、出力口、インジケータ
比率: 立方体に近い
```

### conveyor（コンベア各種）
```
サイズ: 16x16x4
充填率: 40-80%
色数: 3-5色
特徴: フレーム、ベルト、矢印
比率: 平たい
```

---

## マテリアルパレット

### 基本マテリアル
| 名前 | 用途 |
|------|------|
| iron | 鉄製パーツ |
| copper | 銅製パーツ |
| brass | 真鍮、ギア |
| dark_steel | 重機、ベース |
| wood | ハンドル |
| stone | 基礎、炉 |

### 機械用
| 名前 | 用途 |
|------|------|
| frame | フレーム |
| belt | ベルト |
| roller | ローラー |
| arrow | 方向矢印 |
| furnace_body | 精錬炉本体 |
| furnace_glow | 発光部分 |
| crusher_body | 粉砕機本体 |
| miner_body | 採掘機本体 |

### アクセント（特徴要素に使用）
| 名前 | 用途 |
|------|------|
| danger | 危険表示（赤） |
| warning | 警告表示（黄） |
| power | 電力表示（青） |
| active | 稼働表示（緑） |

---

## VoxelModel API

```python
model = VoxelModel(size_x, size_y, size_z)

# 塗りつぶし
model.fill_box(x1, y1, z1, x2, y2, z2, "material")
model.fill_box_hollow(x1, y1, z1, x2, y2, z2, "material", thickness=1)
model.fill_cylinder(cx, cy, z1, z2, radius, "material")

# 単一ボクセル
model.set_voxel_named(x, y, z, "material")
model.remove_voxel(x, y, z)

# ヘルパー
model.draw_arrow(x, y, z, "+y", "arrow")
model.draw_line(x1, y1, z1, x2, y2, z2, "material")

# 出力
model.save("path/to/file.vox")
stats = model.get_stats()
```

---

## 自動改善の例

### 初回評価
```
=== furnace (machine) ===
Total Score: 5.8/10 ✗ FAIL

Scores:
  symmetry             [██████░░░░] 6.0 ✗
  fill_ratio           [████████░░] 8.0 ✓
  color_variety        [████░░░░░░] 4.0 ✗
  feature_presence     [██████░░░░] 6.0 ✗
  proportions          [████████░░] 8.0 ✓

Issues:
  - 対称性が低い (65%)
  - 色が少なすぎる (2 < 3)
  - 特徴が不足: indicator, output

Suggestions:
  → 左右対称になるようボクセルを配置
  → アクセント色（warning, active）を追加
  → 特徴的な要素（出力口、インジケータ等）を追加
```

### 修正後
```python
# 修正: インジケータとアクセント色を追加
model.fill_box(3, 2, 10, 4, 2, 11, "active")      # インジケータ
model.fill_box(12, 2, 10, 13, 2, 11, "active")    # 対称に配置
model.fill_box(2, 2, 5, 13, 2, 6, "warning")      # アクセントライン
```

### 再評価
```
=== furnace (machine) ===
Total Score: 8.2/10 ✓ PASS
```

---

## 学習データベース

評価結果は `tools/model_learning.json` に蓄積され、将来の生成に活用。

```python
from tools.voxel_evaluator import get_guidance

guidance = get_guidance("machine")
print(guidance["target_scores"])    # 成功モデルの平均スコア
print(guidance["avoid_issues"])     # よくある問題
```

---

## 座標系

```
    +Z (上)
     |
     |
     +---- +Y (奥/出力方向)
    /
   /
  +X (右)
```

- **原点**: モデル中央下部
- **単位**: 16ボクセル = 1ゲームブロック

---

## 出力先

| カテゴリ | パス |
|----------|------|
| item | assets/models/items/{name}.glb |
| machine | assets/models/machines/{name}.glb |
| conveyor | assets/models/machines/conveyor/{name}.glb |

---

## 完全な生成フロー（コピペ用）

```python
import sys
sys.path.insert(0, 'tools')
from voxel_generator import VoxelModel
from voxel_evaluator import evaluate_model, format_evaluation, record_evaluation

def create_MODEL_NAME():
    model = VoxelModel(16, 16, 16)

    # === ここにボクセル定義 ===
    model.fill_box(...)

    return model

# 生成と評価
model = create_MODEL_NAME()
result = evaluate_model(model.voxels, (16, 16, 16), "MODEL_NAME", "machine")
print(format_evaluation(result))

# 合格したら保存
if result.passed:
    model.save("assets/models/machines/MODEL_NAME.vox")
    record_evaluation(result)
    print("Saved! Now convert to glb:")
    print("  blender --background --python tools/vox_to_gltf.py -- *.vox *.glb")
else:
    print("Fix issues and try again.")
```

---

## トラブルシューティング

### スコアが上がらない
- `suggestions`に従って修正
- 対称性: ミラー配置を意識
- 色: アクセント色を必ず1つ以上追加
- 特徴: 出力口、インジケータを忘れずに

### glbで表示されない
- GlobalTransform, Visibilityコンポーネントを確認
- scene.spawnを使用しているか確認

### 向きが違う
- +Y方向が出力/前面
- Transform::from_rotation()で調整
