# 3Dモデル生成コマンド

指定されたモデルのBlender Pythonスクリプトを生成します。

## 引数
$ARGUMENTS

## 指示

以下のスタイルガイドに従って、指定されたモデルのBlender Pythonスクリプトを生成してください。

### スタイルガイド参照
`docs/style-guide.json` を読み込んで仕様を確認してください。

### 生成するスクリプトの要件

1. **Blender 4.0+対応**のPythonスクリプト
2. スタイルガイドの以下を厳守:
   - `geometry.primitives`: 許可された形状のみ使用（円・球・円柱は八角形で代替）
   - `geometry.grid.snap_unit`: 0.0625単位でスナップ
   - `geometry.chamfer`: エッジに45度面取り
   - `scale.categories`: カテゴリに応じたサイズ
   - `origin`: 正しい原点位置
   - `polygon_budget`: ポリゴン上限を守る
   - `materials.presets`: 素材プリセットを使用
   - `mechanical_parts`: 機械パーツの形状仕様に従う
   - `bones`: ボーン命名規則に従う
   - `animation`: アニメーション命名規則に従う

3. スクリプトに含める機能:
   - メッシュ生成
   - マテリアル設定（PBR）
   - 頂点カラーでのエッジ暗化（オプション）
   - ボーン設定（アニメーションが必要な場合）
   - glTFエクスポート設定

4. 出力先: `tools/blender_scripts/{model_name}.py`

### 出力フォーマット

```python
"""
{モデル名} - Blender生成スクリプト
スタイルガイド: Industrial Lowpoly Style Guide v1.0.0
カテゴリ: {カテゴリ}
推奨ポリゴン: {推奨範囲}
"""

import bpy
import bmesh
from mathutils import Vector

# ... スクリプト本体
```

### 実行

生成後、ユーザーにBlenderでの実行方法を案内してください:
1. Blenderを開く
2. Scripting タブに切り替え
3. 生成されたスクリプトを貼り付けて実行
4. `assets/models/{category}/` にエクスポート
