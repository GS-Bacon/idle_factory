# 新アーキテクチャ移行タスク一覧

> **方針**: 全タスクを15分以内で完了できる粒度に分割
> **実行前**: Geminiと設計合見積を取る（コア変更のみ）
> **並列度**: 最大4並列（worktree制限）

---

## タスク進捗サマリー

| カテゴリ | 完了 | 残 | 状態 |
|----------|------|-----|------|
| 1. レガシー削除 | 0/9 | 9 | ⏳ 未着手 |
| 2. プラグイン登録 | 0/10 | 10 | ⏳ 未着手 |
| 3. イベント送信 | 1/8 | 7 | ⏳ 進行中 |
| 4. 動的ID移行 | 0/12 | 12 | ⏳ Gemini合見積必要 |
| 5. セーブ形式 | 0/6 | 6 | ⏳ D.2後 |
| 6. 本体Mod化 | 0/7 | 7 | ⏳ D.2後 |
| **合計** | **1/52** | **51** | |

---

## 1. レガシー削除（独立・並列可）

**前提条件**: なし
**Gemini合見積**: 不要（単純削除）

| ID | タスク | ファイル | 行数 | 状態 |
|----|--------|----------|------|------|
| 1-A | Miner struct削除 | components/machines.rs:453 | ~25行 | ⏳ |
| 1-B | Furnace struct削除 | components/machines.rs:480 | ~50行 | ⏳ |
| 1-C | Crusher struct削除 | components/machines.rs:531 | ~40行 | ⏳ |
| 1-D | MinerSaveData削除 | save/format.rs:251 | ~8行 | ⏳ |
| 1-E | FurnaceSaveData削除 | save/format.rs:278 | ~10行 | ⏳ |
| 1-F | CrusherSaveData削除 | save/format.rs:288 | ~10行 | ⏳ |
| 1-G | MinerDump削除 | debug/state_dump.rs:56 | ~7行 | ⏳ |
| 1-H | FurnaceDump削除 | debug/state_dump.rs:71 | ~9行 | ⏳ |
| 1-I | CrusherDump削除 | debug/state_dump.rs:80 | ~8行 | ⏳ |

**実行コマンド**:
```bash
./scripts/parallel-run.sh start legacy-cleanup
# サブエージェントに: "1-A~1-C を削除、テスト確認"
./scripts/parallel-run.sh finish legacy-cleanup
```

---

## 2. プラグイン登録（独立・並列可）

**前提条件**: なし
**Gemini合見積**: 不要（単純追加）

| ID | タスク | ファイル | 状態 |
|----|--------|----------|------|
| 2-A | MapPlugin登録 | main.rs + src/map/mod.rs | ⏳ |
| 2-B | BlueprintPlugin登録 | main.rs + src/blueprint/mod.rs | ⏳ |
| 2-C | CraftPlugin登録 | main.rs + src/craft/mod.rs | ⏳ |
| 2-D | StoragePlugin登録 | main.rs + src/storage/mod.rs | ⏳ |
| 2-E | StatisticsPlugin登録 | main.rs + src/statistics/mod.rs | ⏳ |
| 2-F | AudioPlugin定義+登録 | main.rs + src/audio/mod.rs | ⏳ |
| 2-G | AchievementPlugin登録 | main.rs + src/achievements/mod.rs | ⏳ |
| 2-H | SkinPlugin登録 | main.rs + src/skin/mod.rs | ⏳ |
| 2-I | RobotPlugin登録 | main.rs + src/robot/mod.rs | ⏳ |
| 2-J | ModdingPlugin登録 | main.rs + src/modding/mod.rs | ⏳ |

**注意**: 各Pluginが存在しない場合は空Pluginを作成

---

## 3. イベント送信追加（独立・並列可）

**前提条件**: なし（基盤済み）
**Gemini合見積**: 不要（パターン確立済み）

| ID | イベント | 送信箇所 | 状態 |
|----|----------|----------|------|
| 3-0 | BlockPlaced | placement.rs:581 | ✅ 完了 |
| 3-A | BlockBroken | 破壊システム | ⏳ |
| 3-B | MachineSpawned | 機械生成時 | ⏳ |
| 3-C | MachineStarted | generic_machine_tick | ⏳ |
| 3-D | MachineCompleted | generic_machine_tick | ⏳ |
| 3-E | InventoryChanged | インベントリ操作全箇所 | ⏳ |
| 3-F | ConveyorTransfer | conveyor.rs | ⏳ |
| 3-G | ItemDelivered | 納品システム | ⏳ |

**パターン**:
```rust
events.send(MachineCompleted { entity, recipe_id, outputs });
```

---

## 4. 動的ID移行（依存あり・Gemini合見積必須）

**前提条件**: Phase 4-0（設計確定）完了後
**Gemini合見積**: 必須（影響範囲大）

### Phase 4-0: 設計確定（Gemini合見積）

| ID | 質問 | 状態 |
|----|------|------|
| 4-0a | 移行順序の最適化 | ⏳ |
| 4-0b | 互換性レイヤーの設計 | ⏳ |
| 4-0c | テスト戦略 | ⏳ |

### Phase 4-1: 定義元移行

| ID | ファイル | 箇所数 | 状態 |
|----|----------|--------|------|
| 4-1a | game_spec/mod.rs | 57 | ⏳ |
| 4-1b | game_spec/registry.rs | 50 | ⏳ |
| 4-1c | game_spec/recipes.rs | 46 | ⏳ |

### Phase 4-2: 利用側移行

| ID | ファイル | 箇所数 | 状態 |
|----|----------|--------|------|
| 4-2a | player/global_inventory.rs | 39 | ⏳ |
| 4-2b | player/inventory.rs | 33 | ⏳ |
| 4-2c | main.rs | 33 | ⏳ |
| 4-2d | world/mod.rs | 32 | ⏳ |
| 4-2e | craft/mod.rs | 32 | ⏳ |
| 4-2f | その他（~280箇所） | ~280 | ⏳ |

### Phase 4-3: enum削除

| ID | タスク | 状態 |
|----|--------|------|
| 4-3a | block_type.rs enum削除 | ⏳ |

---

## 5. セーブ形式移行（D.2完了後）

**前提条件**: Phase 4完了
**Gemini合見積**: 必要（互換性設計）

| ID | 構造体 | 変更内容 | 状態 |
|----|--------|----------|------|
| 5-A | BlockTypeSave | enum → String | ⏳ |
| 5-B | InventorySaveData | BlockTypeSave → String | ⏳ |
| 5-C | ConveyorSaveData | items: Vec<BlockTypeSave> → Vec<String> | ⏳ |
| 5-D | MinerSaveData | buffer → String | ⏳ |
| 5-E | FurnaceSaveData | input_type等 → String | ⏳ |
| 5-F | CrusherSaveData | input_type等 → String | ⏳ |

---

## 6. 本体Mod化（D.2完了後）

**前提条件**: Phase 4完了
**Gemini合見積**: 必要（TOML形式設計）

| ID | タスク | 状態 |
|----|--------|------|
| 6-A | mods/base/ ディレクトリ作成 | ⏳ |
| 6-B | mods/base/mod.toml 作成 | ⏳ |
| 6-C | mods/base/items.toml 作成（15アイテム） | ⏳ |
| 6-D | mods/base/machines.toml 作成（4機械） | ⏳ |
| 6-E | mods/base/recipes.toml 作成 | ⏳ |
| 6-F | 起動時ロードシステム | ⏳ |
| 6-G | ITEM_DESCRIPTORS 定数削除 | ⏳ |

---

## 並列実行プラン

### Wave 1（今すぐ・並列4）

```
┌────────────────┐  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐
│ legacy-1       │  │ legacy-2       │  │ plugins-1      │  │ events-1       │
│ 1-A,1-B,1-C    │  │ 1-D,1-E,1-F    │  │ 2-A,2-B,2-C    │  │ 3-A,3-B        │
│ (struct削除)   │  │ (SaveData削除) │  │ (Plugin登録)   │  │ (イベント送信) │
└────────────────┘  └────────────────┘  └────────────────┘  └────────────────┘
```

### Wave 2（Wave 1完了後・並列4）

```
┌────────────────┐  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐
│ legacy-3       │  │ plugins-2      │  │ plugins-3      │  │ events-2       │
│ 1-G,1-H,1-I    │  │ 2-D,2-E,2-F    │  │ 2-G,2-H,2-I,2-J│  │ 3-C,3-D        │
│ (Dump削除)     │  │ (Plugin登録)   │  │ (Plugin登録)   │  │ (イベント送信) │
└────────────────┘  └────────────────┘  └────────────────┘  └────────────────┘
```

### Wave 3（Wave 2完了後）

```
┌────────────────┐  ┌────────────────┐
│ events-3       │  │ gemini-design  │
│ 3-E,3-F,3-G    │  │ 4-0a,4-0b,4-0c │
│ (イベント送信) │  │ (設計合見積)   │
└────────────────┘  └────────────────┘
```

### Wave 4以降（Gemini合見積完了後）

Phase 4 → Phase 5 → Phase 6 を順次実行

---

## サブエージェント指示テンプレート

### レガシー削除用

```
タスク: レガシーコード削除（{ID}）
ファイル: {ファイルパス}
作業内容:
1. 該当structを削除
2. 参照箇所があればエラーを確認
3. 参照がなければ削除完了
4. `cargo test` で確認
5. `cargo clippy` で警告確認

完了条件:
- [ ] struct削除
- [ ] テスト通過
- [ ] Clippy警告0
```

### プラグイン登録用

```
タスク: プラグイン登録（{ID}）
ファイル: {ファイルパス}
作業内容:
1. src/{module}/mod.rs にPluginが存在するか確認
2. なければ空Pluginを作成
3. main.rs の app.add_plugins() に追加
4. `cargo build` で確認
5. `cargo test` で確認

完了条件:
- [ ] Plugin定義存在
- [ ] main.rs登録
- [ ] ビルド成功
- [ ] テスト通過
```

### イベント送信用

```
タスク: イベント送信追加（{ID}）
イベント: {イベント名}
送信箇所: {ファイル:行}
作業内容:
1. 該当箇所を特定
2. EventWriter<{イベント}> をシステム引数に追加
3. events.send({イベント} { ... }) を呼び出し
4. `cargo test` で確認

パターン:
```rust
fn system(
    mut events: EventWriter<{イベント}>,
) {
    events.send({イベント} { entity, ... });
}
```

完了条件:
- [ ] イベント送信追加
- [ ] テスト通過
```

---

## Gemini合見積プロンプト

### 4-0a: 移行順序の最適化

```
動的ID移行（BlockType → ItemId）について、783箇所の移行順序を最適化してください。

現状:
- Id<T> + StringInterner 基盤実装済み
- ItemId, MachineId, RecipeId 型定義済み

移行対象ファイル:
- game_spec/mod.rs (57箇所)
- game_spec/registry.rs (50箇所)
- save/format.rs (77箇所)
- ... (詳細は implementation-plan.md 参照)

質問:
1. どのファイルから始めるべきか？
2. 互換性レイヤーは必要か？
3. 一度に全置換 vs 段階的移行、どちらが安全か？
4. テストはどう書くべきか？
```

---

*最終更新: 2026-01-07*
