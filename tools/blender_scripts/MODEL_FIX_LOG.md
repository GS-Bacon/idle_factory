# モデル修正ログ

Blenderスクリプトで3Dモデルを生成する際の共通問題と修正方法を記録。

---

## 共通問題: オブジェクト結合時の参照エラー

### 症状
- `ReferenceError: StructRNA of type Object has been removed`
- モデルのパーツが分離している
- 意図しないオブジェクトが結合される

### 原因
`bpy.ops.object.join()`を呼ぶ前に、シーン内の他のオブジェクトも選択状態になっている。

### 解決策
**join前にDESELECT、join後はactive_objectを使用**

```python
# 悪い例
bpy.context.view_layer.objects.active = obj1
obj2.select_set(True)
bpy.ops.object.join()
return obj1  # ← 無効な参照になる可能性

# 良い例
bpy.ops.object.select_all(action='DESELECT')  # ← 追加
bpy.context.view_layer.objects.active = obj1
obj1.select_set(True)  # ← 明示的に選択
obj2.select_set(True)
bpy.ops.object.join()
result = bpy.context.active_object  # ← join後の有効な参照
return result
```

---

## 修正履歴

### 2025-12-25: bronze_pickaxe

**問題**: ハンドルとヘッドが分離、向きが不正

**修正前**:
```python
def create_pickaxe(name, material_preset):
    handle = create_handle(length=0.18, radius=0.01)
    handle.location.y = -0.09
    head = create_pickaxe_head(material_preset)
    head.location.y = 0.02
    head.rotation_euler.x = pi / 2  # 間違った回転軸

    bpy.context.view_layer.objects.active = handle
    head.select_set(True)
    handle.select_set(True)
    bpy.ops.object.join()
    handle.name = name  # 無効な参照
```

**修正後**:
```python
def create_pickaxe(name, material_preset):
    # ハンドル（縦向き）
    handle = create_handle(length=0.18, radius=0.012)

    # ヘッド（ハンドル上部に配置）
    head = create_pickaxe_head(material_preset)
    head.location.z = 0.09  # ハンドル上端に配置
    head.rotation_euler.y = pi / 2  # 正しい回転軸

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = handle
    head.select_set(True)
    handle.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object
    result.name = name
```

**変更点**:
1. `location.y` → `location.z` (八角柱はZ軸が高さ)
2. `rotation_euler.y` → `rotation_euler.z` (Z軸回転でT字に)
3. `transform_apply(rotation=True)` 追加（回転を確定）
4. DESELECT追加
5. join後に`bpy.context.active_object`を使用

**最終形コード**:
```python
def create_pickaxe(name, material_preset):
    clear_scene()
    handle = create_handle(length=0.18, radius=0.012)
    head = create_pickaxe_head(material_preset)
    head.location.z = 0.09
    head.rotation_euler.z = pi / 2  # Z軸回転

    # 回転を適用
    bpy.ops.object.select_all(action='DESELECT')
    head.select_set(True)
    bpy.context.view_layer.objects.active = head
    bpy.ops.object.transform_apply(rotation=True)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = handle
    head.select_set(True)
    handle.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object
    result.name = name
    finalize_model(result, category="item")
    return result
```

---

## ツールサイズガイドライン

| カテゴリ | サイズ | 原点 |
|---------|--------|------|
| handheld_item | 0.3×0.3×0.3 | 中央 |
| dropped_item | 0.4×0.4×0.4 | 中央 |

### ツールの構成
- ハンドル長: 0.15-0.20
- ハンドル半径: 0.01-0.015
- ヘッド配置: ハンドル上端（Z座標 = ハンドル長/2）

---

## プレビュー手順

```bash
# 1. スクショ撮影＆確認
./tools/preview_model.sh assets/models/items/<model_name>.gltf

# 2. デスクトップに表示
DISPLAY=:10 xdg-open tools/model_screenshots/<model_name>.png

# 3. モデル再生成（修正後）
blender --background --python tools/blender_scripts/test_single_model.py -- <model_name>
```

---

## レビュー進捗

| 日付 | モデル | 状態 | 備考 |
|------|--------|------|------|
| 2025-12-25 | bronze_pickaxe | 完了 | DESELECT, Z回転, transform_apply |

### 未確認
- tools_items.py: iron_pickaxe, steel_pickaxe, wooden_pickaxe, stone_pickaxe
- tools_items.py: axe系, shovel系
- machines.py: 全13種
