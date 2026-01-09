# Save Format

Technical documentation of the save file format.

セーブファイル形式の技術ドキュメント。

---

## Overview

Saves are stored in JSON format in the `saves/` directory.

セーブは `saves/` ディレクトリにJSON形式で保存されます。

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

現在のフォーマット: **V2**（文字列ベースのItemId）

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

Modアイテムを使用するセーブは以下の場合に互換性あり：

- Mod is still installed

  Modがまだインストールされている

- Item IDs haven't changed

  アイテムIDが変更されていない

Unknown items are logged and skipped on load.

不明なアイテムはロード時にログに記録されスキップされます。

---

## See Also

- **[Data Mod Guide](Data-Mod-Guide)** - Mod item IDs

  ModアイテムID
