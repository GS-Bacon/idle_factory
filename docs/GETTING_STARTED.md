# Getting Started

## Requirements

- Rust stable (1.75+)
- GPU with Vulkan support

## Quick Start

```bash
# Clone and run
git clone https://github.com/your-repo/idle_factory
cd idle_factory
cargo run --release
```

## Controls

| Key | Action |
|-----|--------|
| WASD | Move |
| Space | Jump / Fly up |
| Shift | Fly down |
| E | Open inventory |
| Q | Toggle build mode |
| 1-9 | Select hotbar slot |
| Left Click | Place / Interact |
| Right Click | Remove / Open container |
| F3 | Debug overlay |
| Esc | Menu |

## First Steps

1. **Select an item** from the hotbar (1-9)
2. **Enter build mode** (Q) to place machines
3. **Connect machines** with conveyors
4. **Open inventory** (E) to craft items

## Creating Mods

```
mods/
└── my-mod/
    ├── manifest.yaml    # Required: name, version, dependencies
    ├── data/            # YAML files (items, recipes, machines)
    └── scripts/         # Lua scripts
```

Minimal `manifest.yaml`:
```yaml
name: my-mod
version: 1.0.0
game_version: ">=0.1.0"
```

Add items in `data/items.yaml`:
```yaml
- id: my_item
  name: My Item
  stack_size: 64
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Black screen | Update GPU drivers |
| Low FPS | Use `--release` flag |
| Crash on start | Check Vulkan support |
