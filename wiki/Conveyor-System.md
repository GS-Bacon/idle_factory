[日本語版はこちら](コンベアシステム)

# Conveyor System

Overview of the conveyor belt system for automated item transport.

---

## Basics

Conveyors move items from one location to another automatically.

### Placement

1. Select conveyor from hotbar
2. Click to place
3. Conveyors connect automatically

### Direction

Conveyors have a direction indicated by arrows.

Press R to rotate before placing.

---

## Conveyor Shapes

| Shape | Description |
|-------|-------------|
| Straight | Direct path |
| Corner (L/R) | 90 degree turn |
| T-Junction | Split/merge point |
| Splitter | 1 input to 2 outputs |

---

## Connecting to Machines

### Input Connection

Place conveyor touching machine's input port.

```
[Conveyor] → [Machine Input Port]
```

### Output Connection

Place conveyor at machine's output port.

```
[Machine Output Port] → [Conveyor]
```

---

## Item Flow

Items move along conveyors at a constant speed.

### Blocking

If destination is full, items stop moving.

### Priority

When merging, items alternate between inputs.

---

## Automation Example

A basic ore processing line:

```
[Miner] → [Conveyor] → [Crusher] → [Conveyor] → [Furnace] → [Conveyor] → [Storage]
```

---

## Tips

### Keep Lines Short

Shorter conveyor lines mean faster throughput.

### Avoid Loops

Loops can cause items to circulate forever.

### Use Splitters

Distribute items to multiple machines for parallel processing.

---

## See Also

- **[Machines](Machines)** - Machine ports
- **[Recipes](Recipes)** - Processing recipes
