# Phase 0: 共通基盤の拡張 - タスクリスト

**Feature**: 2-resource-network
**Phase**: 0 - 共通基盤の拡張
**Date**: 2026-01-30
**System**: 80 CPU cores, 30GB RAM (26GB available)
**Parallel Workers**: 6-8 workers recommended

---

## ⛔ 禁止事項（参照）

> `spec.md` の禁止事項セクションを確認すること
> - `PlayerInventory` Resource は禁止
> - `unwrap()` は禁止（Result + expect 使用）
> - 個別機械ファイルは禁止

---

## Phase 0: 共通基盤の拡張

**目標**: 既存の `NetworkGraph<K, V>` を活用し、共通ユーティリティを作成する。

**詳細設計**: [phase0.md](../phase0.md)

**完了条件**: `cargo build && cargo test && cargo clippy -- -D warnings` 通過

---

### Phase 0.1: GridId 型定義

**担当**: TBD
**並列実行**: 可能

| ID | タスク | 状態 | ファイル | 備考 |
|----|--------|------|---------|------|
| T001 | `GridCategory` マーカー構造体を定義 | ❌ | src/core/id.rs:146-155 | `#[derive(Copy, Clone)]` |
| T002 | `GridId` 型エイリアスを定義 | ❌ | src/core/id.rs:158-162 | `pub type GridId = Id<GridCategory>;` |
| T003 | 単体テスト `test_grid_id()` を実装 | ❌ | src/core/id.rs | 型生成、比較、raw() メソッド |
| T004 | 単体テスト `test_grid_id_serialization()` を実装 | ❌ | src/core/id.rs | bincode シリアライズ/デシリアライズ |
| T005 | GridId テストを実行して検証 | ❌ | - | `cargo test core::id::tests::test_grid_id` |

**完了条件**: GridId テストがすべてパス

---

### Phase 0.2: SignalNetwork 型エイリアス定義

**担当**: TBD
**並列実行**: 可能（T001完了後）

| ID | タスク | 状態 | ファイル | 備考 |
|----|--------|------|---------|------|
| T006 | `SignalNetwork` 型エイリアスを定義 | ❌ | src/core/network.rs:119-123 | `NetworkGraph<u64, u8>` |
| T007 | 単体テスト `test_signal_network_add()` を実装 | ❌ | src/core/network.rs | ノード追加、capacity/current検証 |
| T008 | 単体テスト `test_signal_network_connect()` を実装 | ❌ | src/core/network.rs | 接続、neighbors()検証 |
| T009 | SignalNetwork テストを実行して検証 | ❌ | - | `cargo test core::network::tests::test_signal_network` |

**完了条件**: SignalNetwork テストがすべてパス

---

### Phase 0.3: NetworkUnionFind 共通ユーティリティ実装

**担当**: TBD
**並列実行**: 可能（T002完了後）

| ID | タスク | 状態 | ファイル | 備考 |
|----|--------|------|---------|------|
| T010 | `src/logistics/network_utils.rs` 新規ファイル作成 | ❌ | 新規 | module宣言、use文 |
| T011 | `NetworkUnionFind` 構造体を定義 | ❌ | src/logistics/network_utils.rs | parent/rank HashMap フィールド |
| T012 | `new()` コンストラクタを実装 | ❌ | src/logistics/network_utils.rs | 空HashMapで初期化 |
| T013 | `find(&mut self, x: u64) -> u64` を実装 | ❌ | src/logistics/network_utils.rs | パス圧縮付き再帰実装 |
| T014 | `union(&mut self, x: u64, y: u64)` を実装 | ❌ | src/logistics/network_utils.rs | ランクによる結合 |
| T015 | `get_components(&mut self) -> HashMap<u64, Vec<u64>>` を実装 | ❌ | src/logistics/network_utils.rs | 連結成分を取得 |
| T016 | `clear(&mut self)` を実装 | ❌ | src/logistics/network_utils.rs | parent/rankクリア |
| T017 | `component_count(&self) -> usize` を実装 | ❌ | src/logistics/network_utils.rs | ルートノード数カウント |
| T018 | `Default` trait を実装 | ❌ | src/logistics/network_utils.rs | `new()` を呼ぶ |
| T019 | 単体テスト `test_union_find_basic()` を実装 | ❌ | src/logistics/network_utils.rs | union後にfindが一致 |
| T020 | 単体テスト `test_union_find_components()` を実装 | ❌ | src/logistics/network_utils.rs | 連結成分が正しく分離 |
| T021 | 単体テスト `test_union_find_path_compression()` を実装 | ❌ | src/logistics/network_utils.rs | 親ノードがルートに直接指す |
| T022 | 単体テスト `test_union_find_clear()` を実装 | ❌ | src/logistics/network_utils.rs | クリア後に長さ0 |
| T023 | 単体テスト `test_component_count()` を実装 | ❌ | src/logistics/network_utils.rs | コンポーネント数を正しくカウント |
| T024 | `src/logistics/mod.rs` に `pub mod network_utils;` を追加 | ❌ | src/logistics/mod.rs | モジュール公開 |
| T025 | NetworkUnionFind テストを実行して検証 | ❌ | - | `cargo test logistics::network_utils::tests` |

**完了条件**: NetworkUnionFind テストがすべてパス

---

### Phase 0.4: PortType enum と IoPort 構造体実装

**担当**: TBD
**並列実行**: 可能（他のタスクと完全独立）

| ID | タスク | 状態 | ファイル | 備考 |
|----|--------|------|---------|------|
| T026 | `PortType` enum を定義 | ❌ | src/components/machines/ports.rs | Item/Fluid/Power/Signal |
| T027 | `PortSide` enum を定義 | ❌ | src/components/machines/ports.rs | North/East/South/West/Top/Bottom |
| T028 | `IoPort` 構造体を定義 | ❌ | src/components/machines/ports.rs | side/port_type/slot_id |
| T029 | `new(side, port_type, slot_id)` コンストラクタを実装 | ❌ | src/components/machines/ports.rs | フィールド初期化 |
| T030 | 単体テスト `test_port_type()` を実装 | ❌ | src/components/machines/ports.rs | Power/Fluidポート作成検証 |
| T031 | 単体テスト `test_port_side_equality()` を実装 | ❌ | src/components/machines/ports.rs | PortSide比較検証 |
| T032 | 単体テスト `test_port_type_all_variants()` を実装 | ❌ | src/components/machines/ports.rs | 4バリアントすべて検証 |
| T033 | PortType/IoPort テストを実行して検証 | ❌ | - | `cargo test components::machines::ports::tests` |

**完了条件**: PortType/IoPort テストがすべてパス

---

### Phase 0.5: Phase 0 統合テスト

**担当**: TBD
**並列実行**: 不可（すべての実装完了後）

| ID | タスク | 状態 | ファイル | 備考 |
|----|--------|------|---------|------|
| T034 | `cargo build --release` を実行してビルド成功を確認 | ❌ | - | コンパイルエラーなし |
| T035 | `cargo test` を実行して全テスト成功を確認 | ❌ | - | 既存613件 + 新規テストすべてパス |
| T036 | `cargo clippy -- -D warnings` を実行して警告0件を確認 | ❌ | - | clippy警告なし |
| T037 | 既存テスト（613件）がリグレッションなしでパスすることを確認 | ❌ | - | ビルドとテストで確認 |
| T038 | Phase 0 受入基準チェックリストをすべて満たす | ❌ | - | phase0.md の受入基準参照 |

**完了条件**: すべてのテスト通過、警告0件、リグレッションなし

---

## 🚀 並列実行戦略

### Wave 1: 初期タスク（4並列）

```
T001 ── GridCategory マーカー定義
T002 ── GridId 型エイリアス定義
T026 ── PortType enum 定義
T027 ── PortSide enum 定義
```

**実行可能**: これらは異なるファイル内の独立した定義

---

### Wave 2: 中間タスク（6並列）

```
T003 ── test_grid_id 実装
T004 ── test_grid_id_serialization 実装
T006 ── SignalNetwork 型エイリアス（T001完了後）
T010 ── NetworkUnionFind 新規ファイル（T002完了後）
T028 ── IoPort 構造体定義
T029 ── IoPort::new 実装
```

---

### Wave 3: 詳細実装（8並列）

```
T005 ── GridId テスト実行
T007 ── test_signal_network_add 実装
T008 ── test_signal_network_connect 実装
T011 ── NetworkUnionFind::new 実装
T012 ── find メソッド実装
T013 ── union メソッド実装
T014 ── get_components メソッド実装
T015 ── clear メソッド実行
```

---

### Wave 4: テスト実装（8並列）

```
T009 ── SignalNetwork テスト実行
T016-T018 ── component_count, Default 実装
T019-T023 ── NetworkUnionFind テスト実装
T024 ── mod.rs 更新
T025 ── NetworkUnionFind テスト実行
T030-T032 ── PortType/IoPort テスト実装
T033 ── PortType/IoPort テスト実行
```

---

### Wave 5: 統合テスト（3並列）

```
T034 ── cargo build --release
T035 ── cargo test
T036 ── cargo clippy
```

---

## 📊 依存関係グラフ

```
T001 ──┬── T003 ─── T005 ───┐
       │                      ├── T034 ── T037 ── T038
T002 ──┼── T004 ─── T006 ───┤
       │     │         │      │
       │     │         ├── T007 ┤
       │     │         │      │
       │     │         ├── T008 ┤
       │     │         │      │
       │     └─────────┼── T009 ┘
       │               │
       │               ├── T010 ──┬─ T011-T018 ── T019-T023
       │                      │         │
       │                      └───── T024
       │                                │
       └─────────────────────────────────┴─ T025

T026 ── T028 ── T029 ── T030 ── T031 ── T032 ── T033
T027 ─┘

T036
```

---

## 📋 進捗サマリー

| Phase | タスク数 | 完了 | 進捗 | 推定時間 |
|-------|---------|------|--------|----------|
| 0.1 (GridId) | 5 | 0 | 0% | 0.5 時間 |
| 0.2 (SignalNetwork) | 4 | 0 | 0% | 0.5 時間 |
| 0.3 (NetworkUnionFind) | 16 | 0 | 0% | 2.0 時間 |
| 0.4 (PortType/IoPort) | 8 | 0 | 0% | 0.5 時間 |
| 0.5 (統合テスト) | 5 | 0 | 0% | 0.5 時間 |
| **合計** | **38** | **0** | **0%** | **4.0 時間** |

---

## 🧪 テスト戦略

### 各フェーズ完了後即時テスト

1. **T005**: GridId実装完了後 → `cargo test core::id::tests::test_grid_id`
2. **T009**: SignalNetwork実装完了後 → `cargo test core::network::tests::test_signal_network`
3. **T025**: NetworkUnionFind実装完了後 → `cargo test logistics::network_utils::tests`
4. **T033**: PortType/IoPort実装完了後 → `cargo test components::machines::ports::tests`
5. **T035**: すべての実装完了後 → `cargo test` （全テスト）

### 並列テスト実行コマンド例

```bash
# 4つのモジュールテストを並列実行
cargo test --package idle_factory --lib core::id::tests &
cargo test --package idle_factory --lib core::network::tests &
cargo test --package idle_factory --lib logistics::network_utils::tests &
cargo test --package idle_factory --lib components::machines::ports::tests &
wait
```

---

## 📝 SwarmTools 並列実行コマンド例

### Wave 1 実行（4並列）
```bash
cargo swarm execute T001 T002 T026 T027 --parallel 4
```

### Wave 2 実行（6並列）
```bash
cargo swarm execute T003 T004 T006 T010 T028 T029 --parallel 6
```

### Wave 3 実行（8並列）
```bash
cargo swarm execute T005 T007 T008 T011 T012 T013 T014 T015 --parallel 8
```

### Wave 4 実行（8並列）
```bash
cargo swarm execute T009 T016 T017 T018 T019 T020 T021 T022 --parallel 8
```

### Wave 5 実行（3並列）
```bash
cargo swarm execute T034 T035 T036 --parallel 3
```

---

## ⚠️ 注意事項

1. **unwrap() 禁止**: すべてのエラーハンドリングで `Result` + `expect` を使用
2. **型安全性**: GridId と ItemId の比較がコンパイルエラーになることを確認
3. **既存テスト**: 既存の613件のテストがすべてパスすることを確認
4. **並列実行**: 同じファイル内のタスクは慎重に並列実行（競合回避）
5. **テスト挟み込み**: 各実装フェーズ完了後にテストを実行し、早期発見

---

## ✅ 完了条件

- [ ] 全タスクが ✅
- [ ] `cargo build --release` 通過
- [ ] `cargo test` 通過（既存613件 + 新規テスト）
- [ ] `cargo clippy -- -D warnings` 通過（警告0件）
- [ ] Phase 0 受入基準チェックリストすべて満たす

---

## 状態凡例

| 記号 | 意味 |
|------|------|
| ❌ | 未着手 |
| 🔨 | 作業中 |
| ✅ | 完了 |
| ⏸️ | ブロック中（理由を備考に記載） |

---

*最終更新: 2026-01-30*
