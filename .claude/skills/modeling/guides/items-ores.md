# 鉱石アイテム モデリングガイド

## 概要

コンベアやインベントリで表示される小さなアイテムモデル。

| アイテム | ID | 色 |
|----------|-----|-----|
| 鉄鉱石 | `iron_ore` | 茶赤 #8b4513 |
| 銅鉱石 | `copper_ore` | オレンジ茶 #cd7f32 |
| 石炭 | `coal` | 黒 #1a1a1a |
| 石 | `stone` | 灰色 #808080 |

## デザインコンセプト

- **不規則な岩の塊**
- ローポリ（6-12面）
- 特徴的な色で識別しやすく
- コンベア上で回転しても認識できる

## 構造

```
    ╱╲
   ╱  ╲
  ╱ Ore ╲     ← 不規則な多面体
  ╲    ╱
   ╲__╱
```

## サイズ

| 用途 | サイズ |
|------|--------|
| コンベア上 | 0.15 x 0.12 x 0.15 |
| インベントリアイコン | 正方形でレンダリング |

## パーツ詳細

### 鉄鉱石 (Iron Ore)

```json
{
  "shape": "irregular_rock",
  "faces": 8,
  "size": [0.15, 0.12, 0.15],
  "baseColor": "#8b4513",
  "spotColor": "#4a2a0a",
  "metallic": 0.3,
  "roughness": 0.8
}
```

### 銅鉱石 (Copper Ore)

```json
{
  "shape": "irregular_rock",
  "faces": 8,
  "size": [0.15, 0.12, 0.15],
  "baseColor": "#cd7f32",
  "spotColor": "#8b5a2b",
  "metallic": 0.4,
  "roughness": 0.7
}
```

### 石炭 (Coal)

```json
{
  "shape": "irregular_rock",
  "faces": 6,
  "size": [0.14, 0.1, 0.14],
  "baseColor": "#1a1a1a",
  "spotColor": "#333333",
  "metallic": 0.1,
  "roughness": 0.9
}
```

### 石 (Stone)

```json
{
  "shape": "irregular_rock",
  "faces": 8,
  "size": [0.15, 0.12, 0.15],
  "baseColor": "#808080",
  "spotColor": "#606060",
  "metallic": 0.0,
  "roughness": 0.9
}
```

## 検索キーワード

- `ore rock low poly`
- `mineral chunk 3D`
- `iron ore game asset`
- `voxel ore`
