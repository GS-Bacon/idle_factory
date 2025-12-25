# モデルレビューワークフロー

## 概要
3Dモデルを1つずつ確認し、問題があれば修正するワークフロー。

---

## ツール

### 1. プレビュー＆スクショ撮影
```bash
./tools/preview_model.sh <model_path>
```
- F3Dで斜めアングルからスクリーンショット撮影
- 保存先: `tools/model_screenshots/<model_name>.png`

### 2. デスクトップに表示
```bash
DISPLAY=:10 xdg-open tools/model_screenshots/<model_name>.png
```

### 3. 単一モデル再生成（テスト用）
```bash
blender --background --python tools/blender_scripts/test_single_model.py -- <model_name>
```

---

## ワークフロー

```
1. スクショ撮影
   ./tools/preview_model.sh assets/models/items/<model>.gltf

2. 画像確認（Claude経由 or デスクトップ）

3. 問題あり？
   ├─ Yes → スクリプト修正 → 再生成 → 1に戻る
   └─ No  → 次のモデルへ

4. 修正内容をMODEL_FIX_LOG.mdに記録
```

---

## ファイル構成

```
tools/
├── preview_model.sh          # スクショ撮影スクリプト
├── model_screenshots/        # スクショ保存先
│   └── *.png
└── blender_scripts/
    ├── _base.py              # 共通モジュール
    ├── test_single_model.py  # 単一モデルテスト
    ├── MODEL_FIX_LOG.md      # 修正ログ
    └── MODEL_REVIEW_WORKFLOW.md  # このファイル
```

---

## 作業記録

### 2025-12-25

| モデル | 状態 | 問題 | 対応 |
|--------|------|------|------|
| bronze_pickaxe | 完了 | パーツ分離、回転不正 | DESELECT追加、Z軸回転、transform_apply |

### 未確認モデル（優先度高）
- [ ] 他のピッケル（iron, steel, wooden, stone）
- [ ] 斧（axe系）
- [ ] ショベル（shovel系）
- [ ] 機械（machines/）

---

## 共通問題パターン

### 1. パーツが分離する
**原因**: join前にDESELECTしていない
```python
# 修正
bpy.ops.object.select_all(action='DESELECT')
```

### 2. join後の参照エラー
**原因**: 古い変数を使っている
```python
# 修正
result = bpy.context.active_object
```

### 3. 回転が適用されない
**原因**: transform_applyを呼んでいない
```python
# 修正
bpy.ops.object.transform_apply(rotation=True)
```

### 4. 位置がずれる
**原因**: 座標軸の勘違い（八角柱はZ軸が高さ）
```python
# 修正
head.location.z = 0.09  # Y軸ではなくZ軸
```
