[日本語版はこちら](レシピ)

# Recipes

Overview of the recipe system in Idle Factory.

---

## How Recipes Work

1. Machine receives input items
2. Machine checks for matching recipe
3. If fuel required, consumes fuel
4. After craft_time seconds, outputs result

---

## Recipe Types

### Smelting

Furnace recipes convert raw materials to refined products.

| Input | Output | Time |
|-------|--------|------|
| Iron Ore | Iron Ingot | 2.0s |
| Copper Ore | Copper Ingot | 2.0s |
| Iron Dust | Iron Ingot | 1.0s |

### Crushing

Crusher recipes break down ores for bonus yield.

| Input | Output | Bonus |
|-------|--------|-------|
| Iron Ore | Iron Dust x2 | +100% |
| Copper Ore | Copper Dust x2 | +100% |

### Assembly

Assembler recipes combine multiple inputs.

---

## Fuel

Some machines require fuel to operate.

| Fuel | Burn Time |
|------|-----------|
| Coal | 8.0s |
| Wood | 3.0s |

---

## Efficiency Tips

### Crushing Before Smelting

Crush ore → Smelt dust for 2x output.

```
Iron Ore → Crusher → Iron Dust x2 → Furnace → Iron Ingot x2
```

### Dust Smelts Faster

Dust takes less time to smelt than raw ore.

---

## See Also

- **[Machines](Machines)** - Machine types
- **[Conveyor System](Conveyor-System)** - Automating recipes
- **[Data Mod Guide](Data-Mod-Guide)** - Adding custom recipes
