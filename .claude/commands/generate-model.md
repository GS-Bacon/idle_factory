# 3Dモデル生成

Blenderでローポリ3Dモデルを生成する。

## 引数
$ARGUMENTS

## 引数の解析

引数から以下を抽出:
- **モデル名**: 必須（例: "pickaxe", "hammer", "conveyor"）
- **カテゴリ**: item/machine/structure（デフォルト: item）
- **色指定**: オプション（例: "赤", "青", "緑", "#FF5500", "copper"）

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
    # プリセット
    "iron": (0.29, 0.29, 0.29), "copper": (0.72, 0.45, 0.20),
    "brass": (0.79, 0.64, 0.15), "dark_steel": (0.18, 0.18, 0.18),
    "wood": (0.55, 0.41, 0.08), "stone": (0.41, 0.41, 0.41),
}
```

---

## Minecraft/Unturnedスタイルガイド

> 出典: [Blockbench Minecraft Style Guide](https://www.blockbench.net/wiki/guides/minecraft-style-guide/)

### 基本原則

| 原則 | 説明 |
|------|------|
| **シンプルさ最優先** | 形状はモデルで、ディテールはテクスチャで表現 |
| **要素数最小化** | 認識性を保ちながら可能な限り少ないキューブで構成 |
| **認識性 > 正確さ** | 小さいオブジェクトはスケール正確さより認識しやすさ優先 |
| **単一要素で表現** | 球や円柱は1つのキューブで表現（樽、丸太、カボチャ等） |

### スケールと寸法

```
1ブロック = 16×16×16ピクセル = 1m³
1ピクセル = 6.25cm

ゲーム内換算:
- 1.0 unit = 1ブロック = 1m
- 0.0625 unit = 1ピクセル
- ツール全長 0.2-0.25 = 約3-4ピクセル高
```

### 禁止事項

| NG | 理由 | 代替案 |
|----|------|--------|
| **階段状カーブ** | Minecraft美学に反する | 回転で斜めを表現 |
| **連続回転カーブ** | 曲線を作るための連続回転はNG | 単一キューブ |
| **Mixels** | 1px未満の要素や拡大要素の混在 | 統一スケール |
| **頂点操作** | 複雑なメッシュ編集 | プリミティブ組合せ |
| **過剰ディテール** | 多すぎる小要素 | 透明ピクセルで表現 |

### シェーディングルール

```
光源: 上と前から
明るさ階層: 上面 > 前面 > 側面 > 背面 > 底面

避けるべきシェーディング:
- Banding: ピクセルが一列に並ぶ
- Pillow Shading: 中心が明るく外側が暗い同心円状
- Pancake Shading: 単純に片側明/片側暗
```

### Unturned追加ルール

```
スケール: Blender 1 unit = 1m → Unity Scale Factor 1.5
テクスチャ: 256×256（または数ピクセルの単色パレット）
最小コライダー: 0.2×0.2×0.2
複雑度削減: Decimateで簡略化
```

---

## 実行手順

### 1. リファレンス読み込み
`.specify/memory/modeling-compact.md` を読む

### 2. スクリプト作成

**パーツ接続の鉄則**:
```python
# ❌ 浮いたパーツ
head_z = 0.17
handle_top = 0.15  # 隙間あり

# ✅ 相対位置で接続
head_z = 0.17
handle_top = head_z + 0.01  # 貫通接続
handle_length = handle_top - handle_bottom
```

**テンプレート**:
```python
exec(open("tools/blender_scripts/_base.py").read())
clear_scene()
parts = []

# === 色指定（オプション） ===
MAIN_COLOR = (0.8, 0.1, 0.1)  # 赤色。プリセット使用時は apply_preset_material()

# === Minecraft/Unturnedスタイル設計 ===
# 1. 最小キューブ数で形状を表現
# 2. 球/円柱は単一キューブで代用
# 3. 斜めは回転で表現（階段状NG）
# 4. 詳細はテクスチャに任せる

# === パーツ生成 ===
# 基準パーツの位置決め
main_z = 0.17

# 接続パーツは相対位置で計算
sub_top = main_z + overlap
sub_length = sub_top - sub_bottom
sub_center = sub_bottom + sub_length / 2

main = create_chamfered_cube(size, location=(0, 0, main_z), name="Main")
sub = create_octagonal_prism(radius, sub_length, location=(0, 0, sub_center), name="Sub")
parts.extend([main, sub])

# === 結合・マテリアル ===
result = join_all_meshes(parts, "ModelName")

# マテリアル適用
mat = bpy.data.materials.new("Mat")
mat.use_nodes = True
bsdf = mat.node_tree.nodes.get("Principled BSDF")
if bsdf:
    bsdf.inputs["Base Color"].default_value = (*MAIN_COLOR, 1)
    bsdf.inputs["Roughness"].default_value = 0.8  # マット仕上げ
result.data.materials.append(mat)

# === 仕上げ ===
bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
result.location = (0, 0, 0)
export_gltf("assets/models/items/name.gltf")
```

### 3. 生成実行
```bash
DISPLAY=:10 blender --background --python tools/blender_scripts/{name}.py
```

### 4. スクリーンショット検証
```bash
DISPLAY=:10 f3d --camera-azimuth-angle=45 --camera-elevation-angle=30 --output screenshots/{name}_angle.png assets/models/{cat}s/{name}.gltf &
DISPLAY=:10 f3d --camera-azimuth-angle=0 --camera-elevation-angle=0 --output screenshots/{name}_front.png assets/models/{cat}s/{name}.gltf &
DISPLAY=:10 f3d --camera-azimuth-angle=90 --camera-elevation-angle=0 --output screenshots/{name}_side.png assets/models/{cat}s/{name}.gltf &
wait
```

**チェックリスト**:
- [ ] パーツ間に隙間なし
- [ ] キューブ数は最小限か
- [ ] 階段状カーブがないか
- [ ] シルエットで認識可能か

---

## カテゴリ別仕様

| カテゴリ | サイズ | 三角形 | 原点 | キューブ目安 |
|---------|--------|--------|------|-------------|
| item | 0.2-0.3 | 50-300 | center | 3-8個 |
| machine | 0.9-1.0 | 200-800 | bottom_center | 10-25個 |
| structure | 1.0+ | 300-1500 | bottom_center | 15-40個 |

## マテリアル

| プリセット | HEX | 用途 |
|-----------|-----|------|
| iron | #4A4A4A | 鉄製パーツ |
| copper | #B87333 | 配線、熱交換 |
| brass | #C9A227 | ギア、装飾 |
| dark_steel | #2D2D2D | 重機、産業 |
| wood | #8B6914 | ハンドル、支柱 |
| stone | #696969 | 基礎、炉 |

## 出力先
- glTF: `assets/models/{category}s/{name}.gltf`
- スクリプト: `tools/blender_scripts/{name}.py`

## 使用例
- `/generate-model 赤いピッケル` → 赤色のピッケル
- `/generate-model blue sword` → 青い剣
- `/generate-model copper conveyor` → 銅色コンベア
- `/generate-model #FF5500 ハンマー` → オレンジ色ハンマー
