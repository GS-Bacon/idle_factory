# インゴット・粉末アイテム モデリングガイド

## 概要

精錬・粉砕で作られる加工品アイテム。

| アイテム | ID | 色 |
|----------|-----|-----|
| 鉄インゴット | `iron_ingot` | シルバー #c0c0c0 |
| 銅インゴット | `copper_ingot` | 銅色 #b87333 |
| 鉄粉 | `iron_dust` | 暗灰色 #4a4a4a |
| 銅粉 | `copper_dust` | 茶オレンジ #aa6622 |

## デザインコンセプト

### インゴット
- **台形の金属塊**
- 光沢あり
- 整った形状

### 粉末
- **小さな山/パイル形状**
- マットな質感
- 粒子感

## 構造

### インゴット
```
   ┌────────┐
  ╱        ╱│
 ┌────────┐ │    ← 台形ブロック
 │ Ingot  │╱
 └────────┘
```

### 粉末
```
     ╱╲
    ╱  ╲
   ╱Dust╲       ← 山形/パイル
  ╱──────╲
```

## サイズ

| 用途 | サイズ |
|------|--------|
| コンベア上 | 0.12 x 0.08 x 0.06 (インゴット) |
| コンベア上 | 0.1 x 0.06 x 0.1 (粉末) |

## パーツ詳細

### 鉄インゴット (Iron Ingot)

```json
{
  "shape": "trapezoid_block",
  "size": [0.12, 0.04, 0.06],
  "topSize": [0.1, 0.05],
  "color": "#c0c0c0",
  "metallic": 0.8,
  "roughness": 0.2
}
```

### 銅インゴット (Copper Ingot)

```json
{
  "shape": "trapezoid_block",
  "size": [0.12, 0.04, 0.06],
  "topSize": [0.1, 0.05],
  "color": "#b87333",
  "metallic": 0.7,
  "roughness": 0.3
}
```

### 鉄粉 (Iron Dust)

```json
{
  "shape": "pile_cone",
  "baseRadius": 0.05,
  "height": 0.04,
  "color": "#4a4a4a",
  "metallic": 0.1,
  "roughness": 0.95
}
```

### 銅粉 (Copper Dust)

```json
{
  "shape": "pile_cone",
  "baseRadius": 0.05,
  "height": 0.04,
  "color": "#aa6622",
  "metallic": 0.15,
  "roughness": 0.9
}
```

## 検索キーワード

- `metal ingot 3D`
- `iron bar low poly`
- `metal dust pile`
- `copper ingot game asset`
