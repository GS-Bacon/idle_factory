# modeling-rules (VOX方式)

## 概要

ボクセルベースで3Dモデルを生成。グリーディメッシングで自動最適化。

## スタイル

- **形状**: Minecraft/Unturned風ブロック感
- **カラー**: テクスチャレス、マテリアルカラーのみ（Astroneer風）
- **単位**: 16ボクセル = 1ゲームブロック

## ツール

| ファイル | 役割 |
|----------|------|
| `tools/voxel_generator.py` | .voxファイル生成 |
| `tools/vox_to_gltf.py` | .vox → .glb変換（グリーディメッシング） |

## パレット

```
基本: iron, copper, brass, dark_steel, wood, stone
コンベア: frame, belt, roller, arrow
機械: furnace_body, furnace_glow, crusher_body, miner_body
アクセント: danger, warning, power, active
```

## サイズガイド

| カテゴリ | サイズ |
|----------|--------|
| item | 8x8x16 |
| machine | 16x16x16 |
| conveyor | 16x16x4 |
| structure | 32x32x32 |

## 出力先

```
assets/models/
├── items/{name}.glb
├── machines/{name}.glb
└── machines/conveyor/{shape}.glb
```

## 完成済みモデル

### コンベア（VOX方式）
- straight.glb (8.9KB, 232頂点)
- corner_left.glb (8.2KB, 208頂点)
- corner_right.glb (8.2KB, 208頂点)
- t_junction.glb (8.5KB, 220頂点)
- splitter.glb (13.4KB, 404頂点)

### Blender方式（バックアップ）
- *_blender.glb として保存

## ワークフロー

```
1. voxel_generator.pyでモデル定義
2. .voxファイル生成
3. vox_to_gltf.pyでglb変換
4. Blenderでプレビュー確認
5. ゲームで使用
```

## 方向の定義（コード基準）

| 用語 | 定義 | 回転 |
|------|------|------|
| Left | 反時計回り（N→W→S→E→N） | +90° |
| Right | 時計回り（N→E→S→W→N） | -90° |

コンベア曲がり（アイテムの視点）：
- **CornerLeft**: アイテムが**左**へ曲がる（モデル: corner_left.glb）
- **CornerRight**: アイテムが**右**へ曲がる（モデル: corner_right.glb）

重要: モデル名とシェイプ名は一致している必要がある

## コマンド

```bash
# vox生成
python3 tools/voxel_generator.py

# glb変換
blender --background --python tools/vox_to_gltf.py -- input.vox output.glb

# 一括変換
for vox in assets/models/**/*.vox; do
    blender --background --python tools/vox_to_gltf.py -- "$vox" "${vox%.vox}.glb"
done
```

## 配置時の座標

VOXモデルの原点は**底面中央**（XY中央、Z=0から開始）。

配置時の座標計算：
```rust
// VOXモデル使用時 - 底面が原点なのでY+0.5不要
let model_transform = Transform::from_translation(Vec3::new(
    place_pos.x as f32 * BLOCK_SIZE + 0.5,
    place_pos.y as f32 * BLOCK_SIZE,        // ← Yオフセットなし
    place_pos.z as f32 * BLOCK_SIZE + 0.5,
));

// フォールバック（キューブメッシュ）使用時 - 中心が原点なのでY+0.5必要
let cube_transform = Transform::from_translation(Vec3::new(
    place_pos.x as f32 * BLOCK_SIZE + 0.5,
    place_pos.y as f32 * BLOCK_SIZE + 0.5,  // ← Yオフセットあり
    place_pos.z as f32 * BLOCK_SIZE + 0.5,
));
```
