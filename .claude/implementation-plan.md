# 実装計画

> 詳細な将来設計は `.claude/architecture-future.md` 参照
> ロードマップは `.specify/roadmap.md` 参照

## 現状サマリー

| 項目 | 値 |
|------|-----|
| コード行数 | **~19,000行** |
| テスト | **140件** 通過 |
| Clippy警告 | **0件** |
| Phase | **D.0/D.5 完了** → **D.1 次** |

---

## Phase D: 基盤強化（最優先）

**これらは機能追加より先にやる。後からは困難。**

| # | タスク | 内容 | 状態 |
|---|--------|------|------|
| D.0 | **マルチ準備** | PlayerInventory Component化 + LocalPlayer + Query | ✅ 完了 |
| D.1 | **イベントシステム** | Bevy Observer + 全フック + GuardedEventWriter | 未着手 |
| D.2 | **動的ID** | Phantom Type + StringInterner + Registry | 未着手 |
| D.3 | **Mod API Server** | WebSocket/JSON-RPC + バージョニング | 未着手 |
| D.4 | **データ駆動Mod** | TOML/JSONでコンテンツ追加 | 未着手 |
| D.5 | **Blockbenchローダー** | .bbmodel直接読み込み + アニメーション | ✅ 完了 |

### D.0 完了内容 (2026-01-07)

- `LocalPlayer(Entity)` リソース導入
- `PlayerInventory` Component化（`Inventory` Resource削除）
- 全システムを `LocalPlayer + Query<&PlayerInventory>` パターンに移行
  - targeting/block_operations (5箇所)
  - machines/generic.rs, command/*.rs (3箇所)
  - hotbar.rs (4箇所)
  - inventory_ui.rs (5箇所)
  - save/systems.rs (2箇所)
- `sync_inventory_system` 削除
- **-413行** 削減

### D.5 完了内容 (2026-01-07)

- テクスチャ読み込み（base64 → Image）
- Bone階層構造パース
- Keyframe/Animation構造パース
- `load_bbmodel()` でアニメーション返却

---

## 完了済みPhase

### Phase C: データ駆動設計 ✅ (2026-01-07)

| 追加するもの | 以前 | 現在 |
|--------------|------|------|
| 新アイテム | 100行 | **8行** (ItemDescriptor) |
| 新機械 | 500行 | **20行** (MachineSpec) |
| 新レシピ | 5行 | 5行 |

**実装内容**:
- C.1/C.2: ItemDescriptor統合（hardness, drops, stack_size）
- C.3: MachineSpec + generic_machine_tick + setup_generic_machine_ui
- C.4: GameRegistry with O(1) lookup
- Legacy機械コード削除 (-629行)

### Phase B: アーキテクチャ再設計 ✅

- 物流分離: `logistics/conveyor.rs`
- 機械統合: `machines/generic.rs`
- UI統合: `UIState`, `UIAction`, `UIContext`
- セーブ統合: `save/format.rs`, `save/systems.rs`

### Phase A: v0.2完成 ✅

- UIテーマ刷新
- バイオーム表示UI
- チュートリアル

---

## 保留タスク

### 設定画面 (Phase D後)

| # | タスク | 状態 |
|---|--------|------|
| SET-1 | GameSettings リソース定義 | ✅ 定義済み |
| SET-2 | SettingsPlugin 実装 | ✅ 実装済み（**未登録**） |
| SET-3 | 設定画面UI | 未着手 |
| SET-4 | スライダー/トグルWidget | 未着手 |

### パフォーマンス

| # | タスク | 状態 | 効果 |
|---|--------|------|------|
| 10 | Greedy meshing | ✅ 完了 | 頂点数50%減 |
| 11 | ChunkData HashMap削除 | ✅ 完了 | メモリ50%減 |
| 12 | 差分メッシュ更新 | 未着手 | CPU負荷減 |
| 13 | LOD実装 | 未着手 | 遠距離軽量化 |

---

## 新コンテンツ追加フロー（現在）

```
1. game_spec/registry.rs にItemDescriptor追加（8行）
2. game_spec/machines.rs にMachineSpec追加（20行）
3. game_spec/recipes.rs にレシピ追加（5行）
4. assets/models/ に3Dモデル配置
5. 完了（UIもtickも自動生成）
```

---

*最終更新: 2026-01-07*
