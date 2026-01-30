# Phase 1: 電力ネットワーク実装 - 詳細設計

**フェーズ番号**: 1 | **目標**: 発電機→電線→機械の電力供給システムを実装する。

---

## タスク一覧

| ID | ファイル | 説明 | ステータス |
|----|---------|------|----------|
| T1-1 | `src/components/power.rs` (新規) | `PowerProducer`, `PowerConsumer`, `PowerWire` Component を定義 | TODO |
| T1-2 | `src/logistics/power_grid.rs` (新規) | `PowerNetworks` Resource とグリッド計算システムを実装 | TODO |
| T1-3 | `src/game_spec/machines.rs` | `power_consumption`, `power_output`, `fuel_consumption_rate` フィールド追加 | TODO |
| T1-4 | `src/machines/generic/tick.rs` | `generic_machine_tick()` の冒頭で電力チェック追加 | TODO |
| T1-5 | `src/machines/generic/ui.rs` | 電力状態（powered/unpowered）表示追加 | TODO |

---

## 詳細設計

### T1-1: Power Components 定義
[作業中に記述]

### T1-2: Power Grid 計算システム
[作業中に記述]

### T1-3: MachineSpec に電力フィールド追加
[作業中に記述]

### T1-4: 機械 Tick で電力チェック
[作業中に記述]

### T1-5: UI で電力状態表示
[作業中に記述]

---

## テスト計画
[作業中に記述]

---

## 依存関係
[作業中に記述]

---

## 受入基準
[作業中に記述]

---

*最終更新: 2026-01-30*
