# Conveyor System

Overview of the conveyor belt system for automated item transport.

自動アイテム輸送のためのコンベアベルトシステムの概要。

---

## Basics

Conveyors move items from one location to another automatically.

コンベアはアイテムを自動的にある場所から別の場所に移動します。

### Placement

1. Select conveyor from hotbar
2. Click to place
3. Conveyors connect automatically

設置方法：
1. ホットバーからコンベアを選択
2. クリックして設置
3. コンベアは自動的に接続

### Direction

Conveyors have a direction indicated by arrows.

コンベアには矢印で示される方向があります。

Press R to rotate before placing.

設置前にRキーで回転。

---

## Conveyor Shapes

| Shape | Description |
|-------|-------------|
| Straight | Direct path |
| Corner (L/R) | 90° turn |
| T-Junction | Split/merge point |
| Splitter | 1 input → 2 outputs |

| 形状 | 説明 |
|-----|------|
| 直線 | 直接パス |
| コーナー（左/右） | 90°ターン |
| T字路 | 分岐/合流点 |
| スプリッター | 1入力 → 2出力 |

---

## Connecting to Machines

### Input Connection

Place conveyor touching machine's input port.

コンベアを機械の入力ポートに接触させて設置。

```
[Conveyor] → [Machine Input Port]
```

### Output Connection

Place conveyor at machine's output port.

コンベアを機械の出力ポートに設置。

```
[Machine Output Port] → [Conveyor]
```

---

## Item Flow

Items move along conveyors at a constant speed.

アイテムはコンベア上を一定速度で移動します。

### Blocking

If destination is full, items stop moving.

目的地がいっぱいの場合、アイテムは停止します。

### Priority

When merging, items alternate between inputs.

合流時、アイテムは入力間で交互になります。

---

## Automation Example

A basic ore processing line:

基本的な鉱石加工ライン：

```
[Miner] → [Conveyor] → [Crusher] → [Conveyor] → [Furnace] → [Conveyor] → [Storage]
```

---

## Tips

### Keep Lines Short

Shorter conveyor lines mean faster throughput.

短いコンベアラインはスループットが速い。

### Avoid Loops

Loops can cause items to circulate forever.

ループはアイテムが永遠に循環する原因になる。

### Use Splitters

Distribute items to multiple machines for parallel processing.

スプリッターでアイテムを複数の機械に分配して並列処理。

---

## See Also

- **[Machines](Machines)** - Machine ports

  機械ポート

- **[Recipes](Recipes)** - Processing recipes

  加工レシピ
