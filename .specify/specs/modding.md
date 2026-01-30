# Mod仕様書

## 禁止事項（MUST NOT）

| 禁止 | 理由 |
|------|------|
| ファイルシステムアクセス | セキュリティ |
| ネットワーク直接アクセス | セキュリティ |
| 任意メモリアクセス | WASMサンドボックス |
| シェルコマンド実行 | セキュリティ |
| 他Modのメモリ直接アクセス | 隔離 |

---

## 概要

3層のMod構造でFactorioレベルの拡張性を目指す。

| レイヤー | 形式 | 用途 | 状態 |
|---------|------|------|------|
| **Data Mod** | TOML | アイテム/機械/レシピ定義 | ✅ 実装済み |
| **Script Mod** | WebSocket | イベント監視、外部連携 | ✅ 基盤あり |
| **Core Mod** | WASM | 新ロジック追加 | ✅ M2完了 |

## できること / できないこと

| 機能 | Data | Script | Core |
|------|:----:|:------:|:----:|
| アイテム/機械/レシピ追加 | ✅ | - | ✅ |
| イベント監視 | - | ✅ | ✅ |
| 機械状態変更 | - | ❌ | ✅ |
| ブロック配置/削除 | - | ❌ | ✅ |
| 新ロジック（電力等） | - | - | ✅ |
| UI追加 | - | - | ✅ |
| Entity/Mob生成 | - | - | ✅ |

## ディレクトリ構造

```
mods/
├── base/                  # Data Mod（TOML）
│   ├── items.toml
│   ├── machines.toml
│   └── recipes.toml
└── sample_power_mod/      # Core Mod（WASM）
    ├── mod.toml
    ├── Cargo.toml
    └── src/lib.rs
```

### mod.toml形式

```toml
[mod]
id = "power_system"
name = "Power System"
version = "1.0.0"
api_version = 1

[dependencies]
base = ">=1.0.0"
```

## API安定性

> 製品版まで公式Modのみ対応。後方互換性は気にしない。

| 方針 | 詳細 |
|------|------|
| 破壊的変更 | ✅ OK |
| 関数削除 | ✅ OK |
| バージョニング | 不要 |

## 詳細

- WASM API詳細: [modding-api.md](modding-api.md)
- サンプルMod: `mods/sample_power_mod/`

---

*最終更新: 2026-01-30*
