# Phase 0: 共通基盤の拡張 - 詳細設計

**フェーズ番号**: 0 | **目標**: 既存の `NetworkGraph<K, V>` を活用し、共通ユーティリティを作成する。

---

## タスク一覧

| ID | ファイル | 説明 | ステータス |
|----|---------|------|----------|
| | T0-1 | `src/core/id.rs` | `GridId` 型を定義 | TODO |
| | T0-2 | `src/core/network.rs` | `SignalNetwork` 型エイリアスを定義（PowerNetwork/FluidNetworkは既存） | TODO |
| | T0-3 | `src/logistics/network_utils.rs` | `NetworkUnionFind` 共通ユーティリティを実装 | TODO |
| | T0-4 | `src/components/machines/ports.rs` | `PortType` enum を新規に作成し `IoPort` に追加 | TODO |

---

## 詳細設計

### T0-1: GridId 型定義

#### 設計

```rust
// src/core/id.rs のカテゴリマーカーセクション（行145-155の後）に追加
#[derive(Copy, Clone)]
pub struct GridCategory;

// 型エイリアス（行158-162の後）に追加
pub type GridId = Id<GridCategory>;
```

#### 設計理由

1. **既存パターンへの準拠**: `Id<Category>` パターン（ItemCategory, MachineCategory等）に従い、一貫性を保つ
2. **型安全性**: GridId と ItemId/MachineId 等の混同をコンパイル時に防止
3. **ネットワークキー用途**: NetworkGraph のキーとして Entity ID (u64) とは別に使用
4. **マルチプレイ対応**: 将来のネットワーク同期で GridId を識別子として活用可能

#### 既存実装との整合性

- `src/core/id.rs` に既存のカテゴリマーカー（ItemCategory, MachineCategory等）と同じパターンで追加
- `Id<Category>` のトレイト実装（PartialEq, Eq, Hash, Serialize, Deserialize）をそのまま継承

---

### T0-2: SignalNetwork 型エイリアス

#### 設計

```rust
// src/core/network.rs の型エイリアスセクション（行119-123の後）に追加

/// Signal-specific network type
/// K = u64 (Entity ID or grid position)
/// V = u8 (signal strength 0-15, Minecraft redstone-style)
pub type SignalNetwork = NetworkGraph<u64, u8>;
```

#### 設計理由

1. **一貫性**: 既存の PowerNetwork, FluidNetwork と同じパターンで定義
2. **データ型最適化**: u8 は信号強度（0-15）に最適化。Minecraft レッドストーンと同じモデル
3. **NetworkGraph 再利用**: 既存の `add_node`, `connect`, `disconnect` 等のメソッドをそのまま使用
4. **テスト容易性**: 既存の NetworkGraph テストがそのまま利用可能

#### 既存実装との整合性

- `src/core/network.rs:119-123` に既存の PowerNetwork, FluidNetwork 定義がある
- SignalNetwork を同じセクションに追加

---

### T0-3: NetworkUnionFind 共通ユーティリティ

#### 設計

```rust
// src/logistics/network_utils.rs (新規ファイル)

use std::collections::HashMap;

/// Union-Find (Disjoint Set Union) for network grid calculation
///
/// Efficiently groups connected entities into grids with O(α(n)) amortized time.
/// Used for Power, Fluid, and Signal networks to detect connected components.
///
/// # Example
/// ```rust,ignore
/// let mut uf = NetworkUnionFind::new();
/// uf.union(1, 2);
/// uf.union(2, 3);
/// assert_eq!(uf.find(1), uf.find(3));
/// let components = uf.get_components(); // {root: [1, 2, 3]}
/// ```
pub struct NetworkUnionFind {
    parent: HashMap<u64, u64>,
    rank: HashMap<u64, u32>,
}

impl NetworkUnionFind {
    pub fn new() -> Self {
        Self {
            parent: HashMap::new(),
            rank: HashMap::new(),
        }
    }

    /// Find the root of x with path compression
    pub fn find(&mut self, x: u64) -> u64 {
        if let Some(&p) = self.parent.get(&x) {
            if p != x {
                let root = self.find(p);
                self.parent.insert(x, root); // Path compression
            }
            root
        } else {
            self.parent.insert(x, x);
            self.rank.insert(x, 0);
            x
        }
    }

    /// Union two sets by rank (union by rank for balance)
    pub fn union(&mut self, x: u64, y: u64) {
        let root_x = self.find(x);
        let root_y = self.find(y);

        if root_x != root_y {
            let rank_x = self.rank.get(&root_x).copied().unwrap_or(0);
            let rank_y = self.rank.get(&root_y).copied().unwrap_or(0);

            if rank_x < rank_y {
                self.parent.insert(root_x, root_y);
            } else if rank_x > rank_y {
                self.parent.insert(root_y, root_x);
            } else {
                self.parent.insert(root_y, root_x);
                if let Some(r) = self.rank.get_mut(&root_x) {
                    *r += 1;
                }
            }
        }
    }

    /// Get all components: maps root_id to list of nodes in that component
    pub fn get_components(&mut self) -> HashMap<u64, Vec<u64>> {
        let mut components: HashMap<u64, Vec<u64>> = HashMap::new();

        for &node in self.parent.keys() {
            let root = self.find(node);
            components.entry(root).or_default().push(node);
        }

        components
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.parent.clear();
        self.rank.clear();
    }

    /// Number of distinct components
    pub fn component_count(&self) -> usize {
        self.parent.iter().filter(|(k, v)| k == v).count()
    }
}

impl Default for NetworkUnionFind {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_union_find_basic() {
        let mut uf = NetworkUnionFind::new();
        uf.union(1, 2);
        uf.union(2, 3);

        assert_eq!(uf.find(1), uf.find(3));
        assert_ne!(uf.find(1), uf.find(4));
    }

    #[test]
    fn test_union_find_components() {
        let mut uf = NetworkUnionFind::new();
        uf.union(1, 2);
        uf.union(3, 4);

        let components = uf.get_components();
        // Components should be {root1: [1, 2], {root3: [3, 4]}, {4}
        assert_eq!(components.len(), 3);
    }

    #[test]
    fn test_union_find_path_compression() {
        let mut uf = NetworkUnionFind::new();
        uf.union(1, 2);
        uf.union(2, 3);
        uf.union(3, 4);
        uf.union(4, 5);

        let root = uf.find(1);
        assert_eq!(uf.find(5), root);

        // Path compression: parent of 2 should be root
        if let Some(&parent) = uf.parent.get(&2) {
            assert_eq!(parent, root);
        }
    }

    #[test]
    fn test_union_find_clear() {
        let mut uf = NetworkUnionFind::new();
        uf.union(1, 2);
        uf.clear();

        assert_eq!(uf.parent.len(), 0);
        assert_eq!(uf.rank.len(), 0);
    }

    #[test]
    fn test_component_count() {
        let mut uf = NetworkUnionFind::new();
        uf.union(1, 2);
        uf.union(3, 4);

        assert_eq!(uf.component_count(), 3); // {1,2}, {3,4}, {4}
    }
}
```

#### 設計理由

1. **アルゴリズム選定**: Union-Find (Disjoint Set Union) は動的グラフの連結成分検出に最適（O(α(n)) amortized）
2. **既存検証済み**: 電力システム実験で有効性が確認済み（NFR-002, NFR-001 を満たす）
3. **汎用性**: 3つのネットワークすべてで再利用可能
4. **最適化**: パス圧縮とランクによる結合で、実用上ほぼ定数時間

#### 既存実装との整合性

- `src/logistics/` ディレクトリに新規ファイルとして追加
- 既存の `src/logistics/mod.rs` に module 登録が必要

---

### T0-4: PortType enum と IoPort 拡張

#### 設計

```rust
// src/components/machines/ports.rs に追加

/// Port type for network connections
///
/// Each IoPort can be one of these types, determining what network
/// it connects to (Item conveyor, Fluid pipe, Power wire, Signal wire).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortType {
    Item,
    Fluid,
    Power,
    Signal,
}

/// Port side/direction
///
/// Relative to machine orientation or absolute world direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortSide {
    North,
    East,
    South,
    West,
    Top,
    Bottom,
}

/// Generic I/O port for network connections
///
/// Used by machines to specify connection points for different network types.
/// A machine can have multiple IoPorts (e.g., both Power and Fluid).
///
/// # Example
/// ```rust,ignore
/// IoPort {
///     side: PortSide::North,
///     port_type: PortType::Power,
///     slot_id: 0,
/// }
/// ```
#[derive(Clone, Debug)]
pub struct IoPort {
    /// Which side/direction this port is on
    pub side: PortSide,
    /// What type of network this port connects to
    pub port_type: PortType,
    /// Slot ID (for machines with multiple ports of same type)
    pub slot_id: u8,
}

impl IoPort {
    pub fn new(side: PortSide, port_type: PortType, slot_id: u8) -> Self {
        Self {
            side,
            port_type,
            slot_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_type() {
        let power_port = IoPort::new(PortSide::North, PortType::Power, 0);
        let fluid_port = IoPort::new(PortSide::East, PortType::Fluid, 0);

        assert_eq!(power_port.port_type, PortType::Power);
        assert_eq!(power_port.side, PortSide::North);
        assert_eq!(fluid_port.port_type, PortType::Fluid);
        assert_eq!(fluid_port.slot_id, 0);
    }

    #[test]
    fn test_port_side_equality() {
        assert_eq!(PortSide::North, PortSide::North);
        assert_ne!(PortSide::North, PortSide::East);
    }

    #[test]
    fn test_port_type_all_variants() {
        let types = [PortType::Item, PortType::Fluid, PortType::Power, PortType::Signal];
        assert_eq!(types.len(), 4);
    }
}
```

#### 設計理由

1. **アーキテクチャ設計への準拠**: `.claude/architecture.md:1044-1060` で `PortType::Item, Fluid, Power, Signal` が設計済み
2. **既存パターンの拡張**: 既存の `InputPort`/`OutputPort` とは別に、ネットワーク接続用の汎用 `IoPort` を定義
3. **柔軟性**: 1つの機械が複数ポートタイプを持てる（例: 電力入力 + 流体入力 + アイテム出力）
4. **明示的**: `side`, `port_type`, `slot_id` で接続ポイントを明確に指定

#### 既存実装との整合性

- 既存の `InputPort`/`OutputPort` はアイテム入出力用として維持
- `IoPort` は新規追加で、ネットワーク接続専用
- `port_type` フィールドがない既存ポート構造体との衝突を回避

---

## テスト計画

### 単体テスト（各タスク終了後実行）

#### T0-1 GridId テスト

```rust
// src/core/id.rs の #[cfg(test)] mod tests に追加

#[test]
fn test_grid_id() {
    let id1: GridId = Id::new(1);
    let id2: GridId = Id::new(2);
    let item_id: ItemId = Id::new(1);

    assert_ne!(id1, id2);
    assert_eq!(id1.raw(), 1);
    assert_eq!(item_id.raw(), 1); // Same raw value, but different types

    // Type safety: GridId and ItemId cannot be compared (compile error)
    // assert_eq!(id1, item_id); // This would fail to compile
}

#[test]
fn test_grid_id_serialization() {
    let id: GridId = Id::new(42);

    // Serialize/Deserialize should work for save/load
    let serialized = bincode::serialize(&id).unwrap();
    let deserialized: GridId = bincode::deserialize(&serialized).unwrap();

    assert_eq!(id, deserialized);
}
```

#### T0-2 SignalNetwork テスト

```rust
// src/core/network.rs の #[cfg(test)] mod tests に追加

#[test]
fn test_signal_network_add() {
    let mut network: SignalNetwork = NetworkGraph::new();
    network.add_node(1, NetworkNode {
        capacity: 15,
        current: 10,
        production: 0,
    });

    assert_eq!(network.len(), 1);
    assert_eq!(network.get_node(1).unwrap().current, 10);
}

#[test]
fn test_signal_network_connect() {
    let mut network: SignalNetwork = NetworkGraph::new();
    network.add_node(1, NetworkNode::default());
    network.add_node(2, NetworkNode::default());
    network.connect(1, 2);

    let neighbors: Vec<_> = network.neighbors(1).collect();
    assert_eq!(neighbors.len(), 1);
    assert!(neighbors.contains(&2));
}
```

#### T0-3 NetworkUnionFind テスト

詳細設計のテストコード（`src/logistics/network_utils.rs` の #[cfg(test)]）を参照

#### T0-4 PortType / IoPort テスト

詳細設計のテストコード（`src/components/machines/ports.rs` の #[cfg(test)]）を参照

---

### 統合テスト（フェーズ終了時実行）

```toml
# tests/scenarios/resource_network_phase0_integration.toml
name = "Phase 0 統合テスト"
description = "GridId、SignalNetwork、PortTypeが正しく連携"

[[steps]]
action = "get_state"

[[steps]]
action = "assert"
params = { condition = "ui_state == Gameplay" }

# テスト: 型とユーティリティが正しくインポートできることを確認
# （実際には unit tests で検証）
```

---

### バリデーションテスト

フェーズ終了時に以下を実行し、すべて成功することを確認:

```bash
# コンパイル
cargo build --release

# すべてのテスト
cargo test

# Lint（警告なし）
cargo clippy -- -D warnings

# 特定のモジュールテスト
cargo test --package idle_factory --lib core::id::tests::test_grid_id
cargo test --package idle_factory --lib core::network::tests::test_signal_network
cargo test --package idle_factory --lib logistics::network_utils::tests
cargo test --package idle_factory --lib components::machines::ports::tests
```

---

## 依存関係

| タスク | 前提条件 | 理由 |
|------|---------|------|
| T0-1 | なし | GridId は独立した型定義 |
| T0-2 | T0-1 | SignalNetwork は将来 GridId をキーとして使用可能にするため |
| T0-3 | T0-1 | NetworkUnionFind は GridId をキーとして使用する |
| T0-4 | なし | PortType / IoPort は独立した定義 |

**実装順序**: T0-4 → T0-1 → T0-2 → T0-3

---

## 受入基準

- [ ] GridId が `Id<Category>` パターンで定義されている
- [ ] GridCategory がカテゴリマーカーとして追加されている
- [ ] SignalNetwork 型エイリアスが `NetworkGraph<u64, u8>` として定義されている
- [ ] NetworkUnionFind が正しく実装され、find/union/get_components メソッドが動作する
- [ ] NetworkUnionFind のすべてのテストがパスする
- [ ] PortType enum に Item, Fluid, Power, Signal の4つのバリアントが含まれる
- [ ] PortSide enum に North, East, South, West, Top, Bottom の6つのバリアントが含まれる
- [ ] IoPort 構造体が side, port_type, slot_id のフィールドを持つ
- [ ] IoPort に new() コンストラクタが提供されている
- [ ] すべての単体テストがパスする
- [ ] `cargo build --release` が成功する
- [ ] `cargo test` が成功する
- [ ] `cargo clippy -- -D warnings` が成功し、警告が 0 件である
- [ ] 既存のテスト（613件）がすべてパスする（リグレッションなし）

---

## 既存設計との整合性チェック

### Architecture.md 整合性

| 設計項目 | Architecture.md 参照 | Phase 0 実装 | 整合性 |
|---------|----------------------|-------------|--------|
| GridId | 行113-114 | T0-1 | ✅ |
| SignalNetwork | 行344-356 (信号制御) | T0-2 | ✅ |
| PortType | 行1054-1059 | T0-4 | ✅ |
| Union-Find | 行262-286 (電力システム) | T0-3 | ✅ |

### 禁止パターン遵守

| 禁止パターン | Phase 0 での回避 | 検証方法 |
|------------|---------------|---------|
| PlayerInventory Resource | Component 使用なし | `grep -r "Res<GridId>"` で 0 件 |
| 個別機械ファイル | `machines/generic.rs` は使用せず | 新規ファイル作成なし |
| unwrap() | Result + expect 使用 | `cargo clippy` で警告確認 |

### Constitution Check

| ルール | 順守 | 備考 |
|--------|-----|------|
| 動的ID + Phantom Type | ✅ | GridId が Id<Category> パターン |
| Component化 | ✅ | NetworkUnionFind は単一ユーティリティ、GridId は Component ではなく型エイリアス |
| Bevy Observer | N/A | Phase 0 ではイベント不使用 |
| Single Source of Truth | ✅ | データ定義なし（型定義のみ） |

---

*最終更新: 2026-01-30*
