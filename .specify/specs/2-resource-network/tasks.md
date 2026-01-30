# Tasks: Resource Networks

**Branch**: `2-resource-network`
**Date**: 2026-01-30
**Spec**: [spec.md](./spec.md)
**Plan**: [plan.md](./plan.md)

## Task Overview

このタスクリストは、リソースネットワーク機能（電力・流体・信号）の実装進捗を追跡するために使用されます。

---

## Phase 0: 共通基盤の拡張

### T0-1: GridId 型の定義
- [ ] `src/core/id.rs` に `GridId` 型を定義
- [ ] Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize derive
- [ ] テスト: GridId の生成と比較

**担当**: TBD
**見積もり**: 0.5 時間

---

### T0-2: NetworkGraph 型エイリアスの定義
- [ ] `src/core/network.rs` に `PowerNetwork`, `FluidNetwork`, `SignalNetwork` を定義
- [ ] 既存の `NetworkGraph<K, V>` を活用

**担当**: TBD
**見積もり**: 0.5 時間

---

### T0-3: NetworkUnionFind 共通ユーティリティの実装
- [ ] `src/logistics/network_utils.rs` を新規作成
- [ ] `NetworkUnionFind` 構造体を実装
- [ ] `find`, `union`, `get_networks` メソッドを実装
- [ ] テスト: Union-Find の基本的な操作

**担当**: TBD
**見積もり**: 2 時間

---

### T0-4: IoPort 拡張
- [ ] `src/components/machines/ports.rs` の `PortType` enum を更新
- [ ] `Fluid`, `Power`, `Signal` を追加

**担当**: TBD
**見積もり**: 0.5 時間

---

## Phase 1: 電力ネットワーク実装

### T1-1: 電力 Component の定義
- [ ] `src/components/power.rs` を新規作成
- [ ] `PowerProducer` Component を定義
- [ ] `PowerConsumer` Component を定義
- [ ] `PowerWire` Component を定義

**担当**: TBD
**見積もり**: 2 時間

---

### T1-2: 電力グリッドシステムの実装
- [ ] `src/logistics/power_grid.rs` を新規作成
- [ ] `PowerNetworks` Resource を定義
- [ ] `recalculate_power_grids` システムを実装（Union-Find 使用）
- [ ] `distribute_power` システムを実装

**担当**: TBD
**見積もり**: 4 時間

---

### T1-3: MachineSpec の電力フィールド追加
- [ ] `src/game_spec/machines.rs` を更新
- [ ] `power_consumption`, `power_output`, `fuel_consumption_rate`, `startup_delay` を追加

**担当**: TBD
**見積もり**: 1 時間

---

### T1-4: generic_machine_tick() の電力チェック追加
- [ ] `src/machines/generic/tick.rs` を更新
- [ ] tick の冒頭で `PowerConsumer` をチェック
- [ ] `!is_powered` の場合、早期リターン

**担当**: TBD
**見積もり**: 1 時間

---

### T1-5: 機械 UI の電力状態表示追加
- [ ] `src/machines/generic/ui.rs` を更新
- [ ] 電力状態（⚡ Powered / ❌ No Power）を表示

**担当**: TBD
**見積もり**: 1 時間

---

## Phase 2: 流体ネットワーク実装

### T2-1: FluidSpec の定義
- [ ] `src/game_spec/fluids.rs` を新規作成
- [ ] `FluidSpec` 構造体を定義
- [ ] 定義済み流体（水, 溶岩, 蒸気）を追加
- [ ] `get_fluid_spec` ユーティリティ関数

**担当**: TBD
**見積もり**: 2 時間

---

### T2-2: 流体 Component の定義
- [ ] `src/components/fluids.rs` を新規作成
- [ ] `FluidSource` Component を定義
- [ ] `FluidDrain` Component を定義
- [ ] `Pipe` Component を定義
- [ ] `Tank` Component を定義

**担当**: TBD
**見積もり**: 2 時間

---

### T2-3: 流体ネットワークシステムの実装
- [ ] `src/logistics/pipe.rs` を新規作成
- [ ] `FluidNetworks` Resource を定義
- [ ] `update_fluid_networks` システムを実装
- [ ] 流体転送ロジックを実装（圧力/粘度考慮）

**担当**: TBD
**見積もり**: 4 時間

---

### T2-4: MachineSpec の流体フィールド追加
- [ ] `src/game_spec/machines.rs` を更新
- [ ] `fluid_consumption`, `required_fluid`, `required_temperature` を追加

**担当**: TBD
**見積もり**: 1 時間

---

### T2-5: generic_machine_tick() の流体チェック追加
- [ ] `src/machines/generic/tick.rs` を更新
- [ ] `FluidDrain` をチェック
- [ ] 流体なしの場合、早期リターン

**担当**: TBD
**見積もり**: 1 時間

---

### T2-6: 機械 UI の流体状態表示追加
- [ ] `src/machines/generic/ui.rs` を更新
- [ ] 流体タイプ、量、温度を表示

**担当**: TBD
**見積もり**: 1 時間

---

## Phase 3: 信号ネットワーク実装

### T3-1: 信号 Component の定義
- [ ] `src/components/signals.rs` を新規作成
- [ ] `SignalEmitter` Component を定義
- [ ] `SignalReceiver` Component を定義
- [ ] `SignalWire` Component を定義
- [ ] `LogicGate` enum を定義
- [ ] `SignalInput` Component を定義
- [ ] `SignalCondition` enum を定義

**担当**: TBD
**見積もり**: 3 時間

---

### T3-2: 信号ネットワークシステムの実装
- [ ] `src/logistics/signals.rs` を新規作成
- [ ] `SignalNetworks` Resource を定義
- [ ] `update_signal_networks` システムを実装
- [ ] 信号伝播ロジックを実装（減衰モデル）
- [ ] 論理ゲート処理を実装

**担当**: TBD
**見積もり**: 4 時間

---

### T3-3: MachineSpec の信号フィールド追加
- [ ] `src/game_spec/machines.rs` を更新
- [ ] `signal_input` フィールド追加

**担当**: TBD
**見積もり**: 0.5 時間

---

### T3-4: generic_machine_tick() の信号チェック追加
- [ ] `src/machines/generic/tick.rs` を更新
- [ ] `SignalInput` をチェック
- [ ] 信号OFFの場合、早期リターン

**担当**: TBD
**見積もり**: 0.5 時間

---

### T3-5: 機械 UI の信号状態表示追加
- [ ] `src/machines/generic/ui.rs` を更新
- [ ] 信号状態（🔴 Signal ON / ⚫ Signal OFF）を表示

**担当**: TBD
**見積もり**: 0.5 時間

---

## Phase 4: UIと統合

### T4-1: ネットワーク統計UI の実装
- [ ] `src/ui/network_stats.rs` を新規作成
- [ ] 電力統計表示（発電量、消費量、余剰/不足）
- [ ] 流体統計表示（流量、タンク容量）
- [ ] 信号統計表示（アクティブな信号）

**担当**: TBD
**見積もり**: 3 時間

---

### T4-2: UI システムへの登録
- [ ] `src/plugins/ui.rs` を更新
- [ ] ネットワーク統計UIをシステムに登録

**担当**: TBD
**見積もり**: 0.5 時間

---

## Phase 5: イベントとセーブ/ロード

### T5-1: ゲームイベントの追加
- [ ] `src/events/game_events.rs` を更新
- [ ] `PowerStateChanged`, `FluidFlowChanged`, `SignalActivated`, `SignalDeactivated` を追加

**担当**: TBD
**見積もり**: 1 時間

---

### T5-2: セーブ/ロードフォーマットの定義
- [ ] `src/save/format/v2/networks.rs` を新規作成
- [ ] `PowerNetworkData`, `FluidNetworkData`, `SignalNetworkData` を定義

**担当**: TBD
**見積もり**: 2 時間

---

### T5-3: セーブ/ロードシステムの更新
- [ ] `src/save/systems.rs` を更新
- [ ] ネットワークデータのシリアライズ
- [ ] ネットワークデータのデシリアライズ

**担当**: TBD
**見積もり**: 3 時間

---

## Phase 6: テストと完了

### T6-1: 電力ネットワークの単体テスト
- [ ] `src/logistics/power_grid/tests.rs` を新規作成
- [ ] 単一発電機のテスト
- [ ] 複数発電機のテスト
- [ ] ネットワーク分割のテスト

**担当**: TBD
**見積もり**: 2 時間

---

### T6-2: 流体ネットワークの単体テスト
- [ ] `src/logistics/pipe/tests.rs` を新規作成
- [ ] 基本的な流体転送のテスト
- [ ] 流体プロパティのテスト
- [ ] タンク容量のテスト

**担当**: TBD
**見積もり**: 2 時間

---

### T6-3: 信号ネットワークの単体テスト
- [ ] `src/logistics/signals/tests.rs` を新規作成
- [ ] 信号伝播のテスト
- [ ] 減衰モデルのテスト
- [ ] 論理ゲートのテスト

**担当**: TBD
**見積もり**: 2 時間

---

### T6-4: シナリオテストの作成
- [ ] `tests/scenarios/power_basic.toml` を作成
- [ ] `tests/scenarios/fluid_basic.toml` を作成
- [ ] `tests/scenarios/signal_basic.toml` を作成
- [ ] `tests/scenarios/integrated_networks.toml` を作成

**担当**: TBD
**見積もり**: 4 時間

---

### T6-5: 統合テスト
- [ ] 3つのネットワークが独立して動作することを確認
- [ ] パフォーマンステスト（100ノード以上のネットワーク）
- [ ] セーブ/ロードの一貫性テスト

**担当**: TBD
**見積もり**: 3 時間

---

## タスクサマリー

| Phase | タスク数 | 総見積もり | 進捗 |
|--------|----------|-------------|--------|
| Phase 0 | 4 | 3.5 時間 | 0% |
| Phase 1 | 5 | 9 時間 | 0% |
| Phase 2 | 6 | 11 時間 | 0% |
| Phase 3 | 5 | 9.5 時間 | 0% |
| Phase 4 | 2 | 3.5 時間 | 0% |
| Phase 5 | 3 | 6 時間 | 0% |
| Phase 6 | 5 | 13 時間 | 0% |
| **合計** | **30** | **55.5 時間** | **0%** |

---

*最終更新: 2026-01-30*
