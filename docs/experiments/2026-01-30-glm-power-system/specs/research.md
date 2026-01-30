# Research: M3 Power System

**Feature**: M3電力システム
**Date**: 2026-01-30

## Overview

本機能では、発電機→電線→機械の電力供給システムを実装する。複数の独立した電力グリッドをサポートし、電力不足時に機械が停止する。

## Research Topics

### 1. 電力グリッド計算アルゴリズム

**問題**: 電力ネットワークを効率的に検出・管理する必要がある。ネットワークは動的に変更される（電線追加/削除、機械配置/破壊）。

**調査事項**:
- グリッド検出アルゴリズム（Union-Find vs BFS/DFS）
- 動的更新時の効率的な再計算方法
- 100台以上の機械を50ms以内に計算できるか

**決定**: Union-Find（Disjoint Set Union）を使用

**根拠**:
- **Union-Find**: O(α(n)) amortized、動的な接続追加/削除に最適
  - 初期化: O(n)
  - Union操作: O(α(n)) ≈ O(1)
  - Find操作: O(α(n)) ≈ O(1)
- **BFS/DFS**: 毎回のグリッド検出にO(V+E)、動的更新に非効率
- **現実的な負荷**: 100台の機械であれば、Union-Findは50ms以内に十分完了
- **実装の単純さ**: Union-Findは数10行で実装可能、メンテナンスが容易

**代替案（検討済み）**:
- **BFS/DFS**: 検討したが、動的更新のたびに全グラフを走査するため、頻繁なネットワーク変更に非効率
- **隣接リストのみ**: 実装が最も単純だが、ネットワーク分割/統合の検出が困難

**参考実装**:
```rust
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self { /* ... */ }
    fn find(&mut self, x: usize) -> usize { /* path compression */ }
    fn union(&mut self, x: usize, y: usize) { /* union by rank */ }
}
```

---

### 2. 既存の機械システムとの統合

**問題**: 既存の機械システム（`src/machines/generic/`）は、`Machine` Componentとレシピ処理を持っている。これに電力消費機能を統合する必要がある。

**調査事項**:
- 既存の`generic_machine_tick()`のどこで電力チェックを追加すべきか
- 無電力時の機械停止処理をどう実装するか
- `PowerConsumer` Componentを既存の機械にどう追加するか

**決定**: `generic_machine_tick()`の冒頭で電力チェックを追加、無電力時は早期リターン

**根拠**:
- **既存パターン**: `src/machines/generic/tick.rs`にレシピ処理ロジックが集約されている
- **責任の分離**: 電力チェックはシステム全体の前提条件であり、tick処理の冒頭でチェックするのが自然
- **コードの重複回避**: 既存の`Machine` Componentを変更せず、新しい`PowerConsumer` Componentを追加するだけで済む
- **テスト容易性**: 電力状態が明確な「停止/動作」バイナリとして扱える

**実装方針**:
```rust
fn generic_machine_tick(...) {
    let machine = machines.get(entity).unwrap();

    // 電力チェック（新規）
    if let Some(power_consumer) = power_consumers.get(entity) {
        if !power_consumer.is_powered {
            return; // 無電力時は早期リターン
        }
    }

    // 既存のレシピ処理
    // ...
}
```

**代替案（検討済み）**:
- **各機械tickで個別に電力チェック**: コード重複が増えるため却下
- **独立した電力管理システム**: 既存システムとの統合が複雑になるため却下

---

### 3. 既存のイベントシステムとの統合

**問題**: 電力グリッドの変更（発電機追加/削除、電線追加/削除）を検知し、適切なイベントを発行する必要がある。

**調査事項**:
- 既存のイベントシステム（`src/events/game_events.rs`）のパターン
- どのイベントを新規追加すべきか
- 高頻度イベント（電力グリッド計算）の扱い

**決定**: 既存のイベントパターンに従い、`PowerGridChanged`イベントを追加

**根拠**:
- **既存パターン**: `BlockPlaced`, `MachineSpawned`, `MachineCompleted`等、Bevy Observerパターンを使用
- **イベント駆動アーキテクチャ**: `.claude/architecture.md`の「イベントシステム」原則に従う
- **高頻度除外**: 既存の`ConveyorTransfer`は外部通知OFF（`GuardedEventWriter`）→電力グリッド計算も同様に扱う
- **イベント種別の明確化**: `PowerGridChangeType` enumで変更種別を分類（`GridCreated`, `GridSplit`, `GridMerged`等）

**実装方針**:
```rust
// events/game_events.rsに追加
#[derive(Message, Debug)]
pub struct PowerGridChanged {
    pub grid_id: u64,
    pub change_type: PowerGridChangeType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerGridChangeType {
    GridCreated,
    GridSplit { new_grid_ids: Vec<u64> },
    GridMerged { merged_into_id: u64 },
    GeneratorAdded,
    GeneratorRemoved,
    ConsumerAdded,
    ConsumerRemoved,
    WireAdded,
    WireRemoved,
}
```

**代替案（検討済み）**:
- **Resource直接監視**: イベント駆動アーキテクチャと整合しないため却下
- **System間直接呼び出し**: Modフック、デバッグが困難になるため却下

---

### 4. UI表示の実装方法

**問題**: プレイヤーに電力状態（有電力/無電力）を視覚的に表示し、グリッド統計を提供する必要がある。

**調査事項**:
- 既存のUIコンポーネント（`src/systems/ui/machine_ui.rs`）のパターン
- 機械UIでの電力情報表示方法
- グリッド統計UIの表示場所と実装方法

**決定**: 既存の`machine_ui.rs`のパターンに従い、機械UIに電力情報を追加。グリッド統計は独立したパネルとして実装。

**根拠**:
- **既存パターン**: `src/systems/ui/machine_ui.rs`で、機械のスロット、レシプション等を表示している
- **UI状態管理**: 既存の`UIContext`スタック型管理に従う
- **テキストヘルパー使用**: `.claude/AGENTS.md`の「UI実装ルール」に従い、`text_font()`ヘルパーを使用
- **設定画面への追加禁止**: `AGENTS.md`の「設定画面への機能追加」ルールに従い、ゲームプレイUIとして実装

**実装方針**:
```rust
// 機械UIに電力情報を追加
spawn_section_header(panel, font, "電力");
spawn_text(panel, font, &format!("状態: {}", if is_powered { "稼働中" } else { "停止中" });
spawn_text(panel, font, &format!("消費電力: {}W", required_power));

// グリッド統計パネル（新規）
spawn_grid_stats_panel(font, power_grid_resource);
```

**代替案（検討済み）**:
- **HUDオーバーレイ**: 実装が複雑、既存のUIパターンと整合しないため却下
- **3D空間に表示**: 既存のUIシステムと統合が困難なため却下

---

## Summary

| Topic | Decision | Rationale |
|-------|-----------|-----------|
| グリッド計算アルゴリズム | Union-Find | O(α(n)) amortized、動的更新に最適、実装が単純 |
| 機械システム統合 | `generic_machine_tick()`冒頭でチェック | 既存パターンに従う、責任の分離、コード重複回避 |
| イベントシステム統合 | `PowerGridChanged`イベント追加 | 既存パターンに従う、イベント駆動アーキテクチャ準拠 |
| UI表示 | 既存`machine_ui.rs`拡張 | 既存パターンに従う、UIルール準拠 |

## Outstanding Questions

なし（全て決定済み）

## References

- `.claude/architecture.md` - 電力システム設計骨格
- `.claude/AGENTS.md` - UI実装ルール
- `src/machines/generic/tick.rs` - 既存の機械tick処理
- `src/events/game_events.rs` - 既存のイベントシステム
