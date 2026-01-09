# Machines

Overview of the machine system in Idle Factory.

Idle Factoryの機械システムの概要。

---

## Machine Types

### Processing Machines

Machines that transform input items into output items.

入力アイテムを出力アイテムに変換する機械。

| Machine | Function | Fuel |
|---------|----------|------|
| Furnace | Smelts ores to ingots | Yes |
| Crusher | Crushes ores to dust (2x yield) | No |
| Assembler | Crafts complex items | No |

| 機械 | 機能 | 燃料 |
|-----|------|-----|
| 精錬炉 | 鉱石をインゴットに精錬 | 必要 |
| 粉砕機 | 鉱石を粉に粉砕（2倍収量） | 不要 |
| 組立機 | 複雑なアイテムをクラフト | 不要 |

### Generation Machines

Machines that produce items without input.

入力なしでアイテムを生産する機械。

| Machine | Output |
|---------|--------|
| Miner | Ore based on biome |

| 機械 | 出力 |
|-----|------|
| 採掘機 | バイオームに基づく鉱石 |

---

## Ports and Conveyors

### Input Ports

Where conveyors deliver items to machines.

コンベアが機械にアイテムを配送する場所。

### Output Ports

Where machines output processed items.

機械が加工済みアイテムを出力する場所。

### Port Directions

| Direction | Relative to |
|-----------|-------------|
| `front` | Machine facing direction |
| `back` | Behind machine |
| `left` | Left side |
| `right` | Right side |
| `top` | Above |
| `bottom` | Below |

| 方向 | 基準 |
|-----|------|
| `front` | 機械の向き |
| `back` | 後ろ |
| `left` | 左側 |
| `right` | 右側 |
| `top` | 上 |
| `bottom` | 下 |

---

## Machine UI

Right-click on a machine to open its UI.

機械を右クリックしてUIを開きます。

- **Input slots**: Where raw materials go

  入力スロット: 原材料を入れる場所

- **Output slots**: Where processed items appear

  出力スロット: 加工済みアイテムが出る場所

- **Fuel slot** (if applicable): Where fuel goes

  燃料スロット（該当する場合）: 燃料を入れる場所

- **Progress bar**: Shows processing progress

  進捗バー: 加工の進行状況を表示

---

## Processing Speed

Processing time is defined per recipe. Machines process items at a fixed rate.

加工時間はレシピごとに定義されています。機械は一定の速度でアイテムを加工します。

---

## See Also

- **[Recipes](Recipes)** - Recipe system

  レシピシステム

- **[Conveyor System](Conveyor-System)** - Item transport

  アイテム輸送

- **[Data Mod Guide](Data-Mod-Guide)** - Adding custom machines

  カスタム機械の追加
