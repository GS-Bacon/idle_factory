# Phase 2: 流体ネットワーク実装 - 詳細設計

**フェーズ番号**: 2 | **目標**: ポンプ→パイプ→タンク→機械の流体供給システムを実装する。

---

## タスク一覧

| ID | ファイル | 説明 | ステータス |
|----|---------|------|----------|
| T2-1 | `src/game_spec/fluids.rs` (新規) | `FluidSpec` と定義済み流体（水, 溶岩, 蒸気）を定義 | TODO |
| T2-2 | `src/components/fluids.rs` (新規) | `FluidSource`, `FluidDrain`, `Pipe`, `Tank` Component を定義 | TODO |
| T2-3 | `src/logistics/pipe.rs` (新規) | `FluidNetworks` Resource と流体転送システムを実装 | TODO |
| T2-4 | `src/game_spec/machines.rs` | `fluid_consumption`, `required_fluid`, `required_temperature` フィールド追加 | TODO |
| T2-5 | `src/machines/generic/tick.rs` | 流体チェック追加 | TODO |
| T2-6 | `src/machines/generic/ui.rs` | 流体状態（量、温度）表示追加 | TODO |

---

## 詳細設計

### T2-1: FluidSpec 定義
[作業中に記述]

### T2-2: Fluid Components 定義
[作業中に記述]

### T2-3: Fluid Networks 転送システム
[作業中に記述]

### T2-4: MachineSpec に流体フィールド追加
[作業中に記述]

### T2-5: 機械 Tick で流体チェック
[作業中に記述]

### T2-6: UI で流体状態表示
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
