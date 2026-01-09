[日本語版はこちら](機械)

# Machines

Overview of the machine system in Idle Factory.

---

## Machine Types

### Processing Machines

Machines that transform input items into output items.

| Machine | Function | Fuel |
|---------|----------|------|
| Furnace | Smelts ores to ingots | Yes |
| Crusher | Crushes ores to dust (2x yield) | No |
| Assembler | Crafts complex items | No |

### Generation Machines

Machines that produce items without input.

| Machine | Output |
|---------|--------|
| Miner | Ore based on biome |

---

## Ports and Conveyors

### Input Ports

Where conveyors deliver items to machines.

### Output Ports

Where machines output processed items.

### Port Directions

| Direction | Relative to |
|-----------|-------------|
| `front` | Machine facing direction |
| `back` | Behind machine |
| `left` | Left side |
| `right` | Right side |
| `top` | Above |
| `bottom` | Below |

---

## Machine UI

Right-click on a machine to open its UI.

- **Input slots**: Where raw materials go
- **Output slots**: Where processed items appear
- **Fuel slot** (if applicable): Where fuel goes
- **Progress bar**: Shows processing progress

---

## Processing Speed

Processing time is defined per recipe. Machines process items at a fixed rate.

---

## See Also

- **[Recipes](Recipes)** - Recipe system
- **[Conveyor System](Conveyor-System)** - Item transport
- **[Data Mod Guide](Data-Mod-Guide)** - Adding custom machines
