# 3Dモデル制作ガイドライン

このドキュメントでは、ゲーム内で使用する3Dモデルの技術仕様を定義します。

---

## 1. ファイル形式

| 用途 | 形式 | 備考 |
|------|------|------|
| 3Dモデル | glTF 2.0 (.gltf/.glb) | Bevy標準対応 |
| テクスチャ | PNG / KTX2 | KTX2推奨（GPU圧縮対応） |
| アニメーション | glTF埋め込み | 別ファイル不可 |

---

## 2. スケール

**1ユニット = 1ブロック = 1メートル**

| カテゴリ | サイズ目安 |
|----------|-----------|
| 1ブロック機械 | 1.0 x 1.0 x 1.0 |
| 手持ちアイテム | 0.3 x 0.3 x 0.3 |
| 地面アイテム（ドロップ） | 0.4 x 0.4 x 0.4 |
| プレイヤー | 0.6 x 1.8 x 0.6 |

---

## 3. 原点・ピボット

| カテゴリ | 原点位置 | 向き |
|----------|---------|------|
| 機械・ブロック | 底面中央 (0, 0, 0) | +Z = 前面 |
| アイテム | 中心 | +Z = 前面 |
| マルチブロック | マスターブロックの底面中央 | +Z = 前面 |

```
      +Y (上)
       |
       |
       +-----> +X (右)
      /
     /
   +Z (前面/正面)
```

---

## 4. ポリゴン数（トライアングル）

| カテゴリ | 推奨 | 上限 |
|----------|------|------|
| 手持ちアイテム | 50-200 | 500 |
| 地面アイテム | 20-100 | 200 |
| 1ブロック機械 | 200-800 | 1,500 |
| マルチブロック（小） | 500-2,000 | 4,000 |
| マルチブロック（大） | 2,000-5,000 | 10,000 |

### LOD用メッシュ

| LODレベル | ポリゴン比率 | 使用距離 |
|-----------|-------------|---------|
| LOD0 (Full) | 100% | 0-50m |
| LOD1 (Medium) | 50% | 50-150m |
| LOD2 (Low) | 25% | 150-300m |

---

## 5. UV展開

- **0-1範囲内**: UVは必ず0-1の範囲に収める
- **パディング**: アトラス化のため、UV島間に最低4px余白
- **方向統一**: テキスト・ロゴは+Y方向が上

### テクスチャ解像度

| 用途 | 解像度 |
|------|--------|
| アイテムアイコン（UI） | 64x64 / 128x128 |
| 機械テクスチャ | 256x256 / 512x512 |
| 大型構造物 | 512x512 / 1024x1024 |

---

## 6. マテリアル

### PBR設定

| プロパティ | 対応 |
|------------|------|
| Base Color | ○ |
| Metallic | ○ |
| Roughness | ○ |
| Normal Map | ○ |
| Emissive | ○ |
| Occlusion | ○ |

### 推奨値

| 素材タイプ | Metallic | Roughness |
|------------|----------|-----------|
| 金属（鉄） | 1.0 | 0.4-0.6 |
| 金属（銅） | 1.0 | 0.3-0.5 |
| 木材 | 0.0 | 0.7-0.9 |
| 石材 | 0.0 | 0.6-0.8 |
| プラスチック | 0.0 | 0.3-0.5 |
| ガラス | 0.0 | 0.1-0.2 |

### 命名規則

```
{モデル名}_{テクスチャ種別}.png

例:
mechanical_press_basecolor.png
mechanical_press_normal.png
mechanical_press_roughness.png
mechanical_press_metallic.png
mechanical_press_emissive.png
```

---

## 7. ボーン・アーマチュア

### 命名規則

```
{機能}_{部位}_{番号}

例:
press_head
crusher_wheel
mixer_blade_01
conveyor_roller_01
conveyor_roller_02
```

### 制約

- ボーン数上限: 30本/モデル
- ルートボーンは必須（`root` または `armature`）
- スケールアニメーション非推奨（回転・移動のみ）

---

## 8. アニメーション

### 命名規則

```
{動作名}_cycle  # ループアニメーション
{動作名}_once   # 単発アニメーション

例:
press_cycle
crush_cycle
saw_spin
door_open_once
```

### フレームレート

- エクスポート: 30 FPS
- ゲーム内再生: 可変（MachineAnimationで制御）

---

## 9. エクスポート設定（Blender）

### glTF 2.0 エクスポート

| 設定 | 値 |
|------|-----|
| Format | glTF Separate (.gltf + .bin + textures) |
| Include > Selected Objects | 必要に応じて |
| Transform > +Y Up | ✓ |
| Mesh > Apply Modifiers | ✓ |
| Mesh > UVs | ✓ |
| Mesh > Normals | ✓ |
| Mesh > Tangents | ✓ |
| Mesh > Vertex Colors | 必要に応じて |
| Material > Materials | Export |
| Animation > Animations | ✓（アニメーションがある場合） |
| Animation > Sampling Rate | 30 |

### チェックリスト

- [ ] スケールが適用されている（Ctrl+A > Scale）
- [ ] 回転が適用されている（Ctrl+A > Rotation）
- [ ] 原点位置が正しい
- [ ] UV展開が0-1範囲内
- [ ] マテリアルが設定されている
- [ ] 不要な頂点・面が削除されている

---

## 10. ファイル配置

```
assets/
├── models/
│   ├── machines/
│   │   ├── mechanical_press.gltf
│   │   ├── mechanical_press.bin
│   │   └── mechanical_press/
│   │       ├── basecolor.png
│   │       └── normal.png
│   ├── items/
│   │   ├── iron_ingot.gltf
│   │   └── copper_wire.gltf
│   └── structures/
│       └── blast_furnace.gltf
└── textures/
    └── machines/
        └── (2Dアニメーション用フレーム)
```

---

## 11. 検証ツール

### glTF Validator
https://github.khronos.org/glTF-Validator/

エクスポート後に必ず検証を実行し、エラーがないことを確認する。

---

*このガイドラインは技術仕様のみを定義しています。デザイン・アートスタイルは別途定義されます。*
