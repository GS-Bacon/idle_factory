[日本語版はこちら](セーブ形式)

# Save Format

Technical documentation of the save file format.

---

## Overview

Saves are stored in JSON format in the `saves/` directory.

---

## File Location

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%/idle_factory/saves/` |
| Linux | `~/.local/share/idle_factory/saves/` |
| macOS | `~/Library/Application Support/idle_factory/saves/` |

---

## Format Version

Current format: **V2** (string-based ItemId)

---

## Structure

```json
{
  "version": 2,
  "player": {
    "position": [x, y, z],
    "inventory": [...]
  },
  "world": {
    "chunks": {...},
    "machines": [...]
  },
  "quests": {...}
}
```

---

## Modding Compatibility

Saves using mod items are compatible as long as:

- Mod is still installed
- Item IDs haven't changed

Unknown items are logged and skipped on load.

---

## See Also

- **[Data Mod Guide](Data-Mod-Guide)** - Mod item IDs
