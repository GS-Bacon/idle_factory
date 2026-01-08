# 実装計画

> **移行状況確認**: `./scripts/migration-status.sh`
> **将来設計**: `.claude/architecture-future.md`

## 現状サマリー (2026-01-08)

| 項目 | 値 |
|------|-----|
| コード行数 | **36,413行** |
| テスト | **397件** 通過 |
| Clippy警告 | **0件** |
| バージョン | **0.3.103** |

---

## 完了済みタスク

| タスク | 状態 | 確認方法 |
|--------|------|----------|
| LocalPlayer Entity化 | ✅ | 47箇所で使用 |
| PlayerInventory Component化 | ✅ | `Res<PlayerInventory>` 0件 |
| MachineBundle使用 | ✅ | 23箇所で使用 |
| NetworkId定義 | ✅ | `components/network.rs` |
| GuardedEventWriter使用 | ✅ | 16箇所で使用 |
| WebSocket API (port 9877) | ✅ | 18メソッド実装済み |
| InteractingMachine統合 | ✅ | 旧Interacting* 0件 |
| レガシー機械削除 | ✅ | 旧struct 0件 |
| パニック防止 (P.0-P.4) | ✅ | フォールバック実装済み |
| セーブV2形式 | ✅ | 文字列ID化完了 |
| 固定Tick導入 | ✅ | FixedUpdate(20Hz) |
| 各Pluginモジュール化 | ✅ | D.6-D.14全て登録済み |

---

## 残りの移行タスク

### M.1: BlockType→ItemId移行

**ステータス**: 🔄 移行中

| カテゴリ | 箇所数 | 対応 |
|----------|--------|------|
| 定義ファイル (`block_type.rs`, `id.rs`) | ~190 | **維持** |
| ワールド・描画層 | ~60 | **維持** (ブロックはBlockType) |
| ゲーム仕様 (`game_spec/`) | ~150 | **移行対象** |
| コンポーネント・システム | ~150 | **移行対象** |
| その他 | ~120 | **移行対象** |

**実質的な移行対象: 約420箇所**

**完了条件**:
- [ ] `game_spec/*.rs` がItemIdベース
- [ ] `core/inventory.rs` がItemIdベース
- [ ] `craft/mod.rs` がItemIdベース

**移行優先度**:
```
高: game_spec/recipes.rs - レシピ定義
高: core/inventory.rs - インベントリ
中: craft/mod.rs - クラフトシステム
低: その他 - 新機能追加時に順次
```

---

## 次のステップ（優先度順）

### 1. 新ゲーム機能追加（ユーザー判断）

| 機能 | 内容 | 参照 |
|------|------|------|
| 電力システム | 発電機→電線→機械 | D.15 |
| 液体・気体 | パイプ、タンク | D.16 |
| 信号制御 | 論理回路 | D.17 |

### 2. BlockType移行（段階的）

新機能実装時に関連箇所をItemIdに移行。
一括移行は非推奨（リスク高）。

### 3. 本体コンテンツMod化

| 項目 | 状態 |
|------|------|
| TOML定義 | ✅ 16アイテム |
| GameRegistry統合 | ❌ 未統合 |

---

## Phase D.15-D.20: 高度機能（将来）

| # | タスク | 内容 | 状態 |
|---|--------|------|------|
| D.15 | 電力 | 電力網、発電機 | ❌ |
| D.16 | 液体・気体 | パイプ、タンク | ❌ |
| D.17 | 信号制御 | ワイヤー、ゲート | ❌ |
| D.18 | 線路 | レール、列車 | ❌ |
| D.19 | Mob | NPC、敵 | ❌ |
| D.20 | マルチプレイ | P2P同期 | ❌ |

---

## 新コンテンツ追加フロー

### 現在（Rustハードコード）
```
1. BlockType enumに追加
2. game_spec/registry.rs にItemDescriptor追加（8行）
3. game_spec/machines.rs にMachineSpec追加（20行）
4. game_spec/recipes.rs にレシピ追加（5行）
5. assets/models/ に3Dモデル配置
```

### 目標（TOML駆動）
```
1. mods/base/items.toml に追加（3行）
2. mods/base/machines.toml に追加（10行）
3. mods/base/recipes.toml に追加（3行）
4. assets/models/ に3Dモデル配置
5. 完了（Rustコード変更なし）
```

---

*最終更新: 2026-01-08*
