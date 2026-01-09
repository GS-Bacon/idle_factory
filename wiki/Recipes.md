# Recipes

Overview of the recipe system in Idle Factory.

Idle Factoryのレシピシステムの概要。

---

## How Recipes Work

1. Machine receives input items
2. Machine checks for matching recipe
3. If fuel required, consumes fuel
4. After craft_time seconds, outputs result

レシピの動作：
1. 機械が入力アイテムを受け取る
2. 機械が一致するレシピを確認
3. 燃料が必要な場合、燃料を消費
4. craft_time秒後、結果を出力

---

## Recipe Types

### Smelting

Furnace recipes convert raw materials to refined products.

精錬炉レシピは原材料を精製品に変換します。

| Input | Output | Time |
|-------|--------|------|
| Iron Ore | Iron Ingot | 2.0s |
| Copper Ore | Copper Ingot | 2.0s |
| Iron Dust | Iron Ingot | 1.0s |

| 入力 | 出力 | 時間 |
|-----|------|------|
| 鉄鉱石 | 鉄インゴット | 2.0秒 |
| 銅鉱石 | 銅インゴット | 2.0秒 |
| 鉄ダスト | 鉄インゴット | 1.0秒 |

### Crushing

Crusher recipes break down ores for bonus yield.

粉砕機レシピは鉱石を分解してボーナス収量を得ます。

| Input | Output | Bonus |
|-------|--------|-------|
| Iron Ore | Iron Dust x2 | +100% |
| Copper Ore | Copper Dust x2 | +100% |

| 入力 | 出力 | ボーナス |
|-----|------|---------|
| 鉄鉱石 | 鉄ダスト x2 | +100% |
| 銅鉱石 | 銅ダスト x2 | +100% |

### Assembly

Assembler recipes combine multiple inputs.

組立機レシピは複数の入力を組み合わせます。

---

## Fuel

Some machines require fuel to operate.

一部の機械は動作に燃料が必要です。

| Fuel | Burn Time |
|------|-----------|
| Coal | 8.0s |
| Wood | 3.0s |

| 燃料 | 燃焼時間 |
|-----|---------|
| 石炭 | 8.0秒 |
| 木材 | 3.0秒 |

---

## Efficiency Tips

### Crushing Before Smelting

Crush ore → Smelt dust for 2x output.

鉱石を粉砕 → ダストを精錬で2倍の出力。

```
Iron Ore → Crusher → Iron Dust x2 → Furnace → Iron Ingot x2
```

### Dust Smelts Faster

Dust takes less time to smelt than raw ore.

ダストは生の鉱石より精錬時間が短い。

---

## See Also

- **[Machines](Machines)** - Machine types

  機械タイプ

- **[Conveyor System](Conveyor-System)** - Automating recipes

  レシピの自動化

- **[Data Mod Guide](Data-Mod-Guide)** - Adding custom recipes

  カスタムレシピの追加
