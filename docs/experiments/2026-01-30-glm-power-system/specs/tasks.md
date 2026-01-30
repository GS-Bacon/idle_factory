# Implementation Tasks: M3 Power System

**Feature**: M3電力システム
**Branch**: `001-power-system-spec`
**Date**: 2026-01-30

## Summary

5つのユーザーストーリーを37の実装タスクに分解しました。各ストーリーは独立してテスト可能で、ファイル競合を回避する順序で実行します。

## Implementation Strategy

**MVP Scope**: User Story 1 (基本電力ネットワーク) - 単一のグリッドで発電機と機械を動作させる

**Incremental Delivery**: 各ユーザーストーリーを優先順位（P1→P5）で実装し、毎回完了時にテスト可能な状態を維持

**Parallel Opportunities**: Phase 2-6の各フェーズ内で、ファイルが被らないタスクは並列実行可能

---

## Task Count Summary

- **Phase 1**: 2タスク（セットアップ）
- **Phase 2**: 7タスク（US1: 基盤）
- **Phase 3**: 3タスク（US1: グリッド計算）
- **Phase 4**: 4タスク（US1: 発電機/消費機械統合）
- **Phase 5**: 4タスク（US2: UIフィードバック）
- **Phase 6**: 4タスク（US3: 複数グリッド）
- **Phase 7**: 4タスク（US4: 燃料ベース）
- **Phase 8**: 3タスク（US5: 動的更新）
- **Phase 9**: 4タスク（仕様通りの完了確認）
- **Phase 10**: 2タスク（統合テスト）
- **Total**: 37タスク

---

## Dependency Graph

```
Phase 1 (セットアップ)
    ↓
Phase 2 (基盤コンポーネント)
    ↓
Phase 3 (グリッド計算) → Phase 4 (機械統合) → Phase 5 (UI)
    ↓                              ↓
Phase 6 (複数グリッド) → Phase 7 (燃料ベース)
    ↓
Phase 8 (動的更新)
    ↓
Phase 9 (完了確認)
    ↓
Phase 10 (統合テスト)
```

---

## Phase 1: プロジェクトセットアップ

### Phase Goal

電力システムの実装に必要なモジュール構造とテンプレートを作成

### Tasks

- [ ] T001 Create power module directory structure in src/components/power.rs
- [ ] T002 Create power systems directory in src/systems/power/mod.rs

---

## Phase 2: User Story 1 - Basic Power Network (基盤)

### Phase Goal

発電機、電線、消費機械のComponentとResourceを作成し、基本データ構造を確立する

### Independent Test Criteria

- 電線ブロックを配置して、発電機を配置して、機械を配置できる
- グリッドResourceが初期化されている
- Componentが正しく登録されている

### Tasks

- [ ] T006a [US1] Create power wire block definition in mods/base/items.toml
- [ ] T006b [US1] Create water wheel generator definition in mods/base/machines.toml
- [ ] T006c [US1] Create coal generator definition in mods/base/machines.toml
- [ ] T006d [US1] Create coal item definition in mods/base/items.toml
- [ ] T003 [US1] Implement PowerProducer component in src/components/power.rs
- [ ] T004 [US1] Implement PowerWire component in src/components/power.rs
- [ ] T005 [US1] Implement PowerGrids resource and PowerGrid struct in src/components/power.rs
- [ ] T006 [P] Export power components in src/components/mod.rs

---

## Phase 3: User Story 1 - Basic Power Network (グリッド計算)

### Phase Goal

Union-Findアルゴリズムで電力ネットワークを検出し、グリッドごとの発電量・消費量を計算するシステムを実装する

### Independent Test Criteria

- 発電機と機械を電線で接続した時、正しく1つのグリッドとして検出される
- 発電量が消費量以上の場合、全機械がis_powered=trueになる
- 発電量が消費量未満の場合、全機械がis_powered=falseになる
- グリッド計算が50ms以内に完了する

### Tasks

- [ ] T007 [US1] Implement Union-Find data structure in src/systems/power/grid_calc.rs
- [ ] T008 [US1] Implement grid calculation system using Union-Find in src/systems/power/grid_calc.rs
- [ ] T009 [US1] Add PowerGridChanged event to src/events/game_events.rs
- [ ] T010 [P] Register power grid calculation system in PowerSystemPlugin (FixedUpdate)

---

## Phase 4: User Story 1 - Basic Power Network (機械統合)

### Phase Goal

既存のgeneric_machine_tickシステムに電力チェックを追加し、無電力時に機械が停止するようにする

### Independent Test Criteria

- 電力がない機械は、progressが進まず、出力が生成されない
- 電力がある機械は、既存通りに処理される

### Tasks

- [ ] T011a [US1] Verify PowerConsumer component exists in src/components/machines/ports.rs
- [ ] T011 [US1] Add power check to generic_machine_tick in src/machines/generic/tick.rs
- [ ] T012 [US1] Add PowerConsumer component to PowerConsumer query in src/machines/generic/tick.rs
- [ ] T013 [P] Update generic_machine_tick to return early when unpowered in src/machines/generic/tick.rs

---

## Phase 5: User Story 2 - Power UI Feedback

### Phase Goal

機械UIで電力状態を視覚的に表示し、グリッド統計パネルを実装する

### Independent Test Criteria

- 機械UIを開いた時、電力状態（稼働中/停止中）が表示される
- グリッド統計パネルで、総発電量、総消費量、余剰/不足が表示される
- 無電力機械は「停止中」と赤色で表示される

### Tasks

- [ ] T014 [US2] Add power status section to machine UI in src/systems/ui/machine_ui.rs
- [ ] T015 [US2] Add grid statistics panel display in src/systems/ui/machine_ui.rs
- [ ] T016 [P] Update machine UI to fetch and display PowerConsumer data

---

## Phase 6: User Story 3 - Multiple Independent Power Grids

### Phase Goal

Union-Findの実装を検証し、グリッド分割・統合が正しく動作することを確認する

### Independent Test Criteria

- 電線を1本抜くと、グリッドが2つに分割される
- 分割されたグリッドは互いに独立し、片方を停止しても他方が影響されない
- グリッドIDが正しく更新される

### Tasks

- [ ] T017 [US3] Implement grid split detection in Union-Find when wires removed in src/systems/power/grid_calc.rs
- [ ] T018 [US3] Implement grid merge detection in Union-Find when wires connect in src/systems/power/grid_calc.rs
- [ ] T019 [US3] Update PowerGridChanged events to emit GridSplit and GridMerged types in src/systems/power/grid_calc.rs

---

## Phase 7: User Story 4 - Fuel-Based Generators

### Phase Goal

MachineSpecに電力関連フィールドを追加し、石炭発電機の燃料消費ロジックを実装する

### Independent Test Criteria

- 石炭発電機に燃料を追加すると、起動遅延後に発電が開始する
- 燃料が尽きると、発電が停止し、接続機械が停止する
- 燃料消費率が正しく動作する

### Tasks

- [ ] T020a [US4] Add power_output and power_consumption fields to MachineSpec struct in src/game_spec/machines.rs
- [ ] T020b [US4] Add fuel_consumption_rate and startup_delay fields to MachineSpec struct in src/game_spec/machines.rs
- [ ] T020c [US4] Update existing machine specs to include power fields in src/game_spec/machines.rs
- [ ] T021 [US4] Implement generator tick system for fuel consumption in src/systems/power/generator_tick.rs
- [ ] T022 [US4] Add startup delay timer logic to generator tick system in src/systems/power/generator_tick.rs
- [ ] T023 [P] Register generator tick system in PowerSystemPlugin (FixedUpdate)

---

## Phase 8: User Story 5 - Dynamic Power Grid Updates

### Phase Goal

グリッド計算を変更検出時のみ実行するようにし、パフォーマンスを最適化する

### Independent Test Criteria

- 電線を追加/削除した時、グリッドが1チック（50ms）以内に更新される
- 高頻度の更新でも、ゲームがフリーズしない

### Tasks

- [ ] T024 [US5] Implement Observer pattern for grid recalculation triggers in src/systems/power/grid_calc.rs
- [ ] T025 [US5] Optimize grid calculation to skip unnecessary recalculations in src/systems/power/grid_calc.rs
- [ ] T026 [US5] Implement PowerGridChanged event filtering for high-frequency events in src/systems/power/grid_calc.rs
- [ ] T027 [P] Set up PowerGridChanged to exclude external Mod notifications (high-frequency) in src/systems/power/grid_calc.rs

---

## Phase 9: 仕様通りの完了確認

### Phase Goal

全ての機能要件と成功基準を満たしていることを確認する

### Tasks

- [ ] T028 Verify FR-001: Automatic power network detection works correctly
- [ ] T029 Verify FR-006: Grid calculations complete within 50ms
- [ ] T030 Verify FR-008: Power state visual indicators are displayed
- [ ] T031 Verify SC-001: Players can create functional power network within 5 minutes

---

## Phase 10: 統合テスト

### Phase Goal

全システムを統合し、シナリオテストで正常動作を確認する

### Independent Test Criteria

- 基本的な電力ネットワーク（発電機→電線→機械）が正しく動作する
- 複数の独立グリッドが正しく動作する
- 燃料ベース発電機が正しく動作する
- 無電力時の機械停止が正しく動作する
- グリッド統計が正しく表示される

### Tasks

- [ ] T032 [P] Run cargo build and fix any compilation errors
- [ ] T033 [P] Run cargo test and fix any test failures
- [ ] T034 [P] Run cargo clippy and fix any warnings
- [ ] T035 Create basic power network scenario test in tests/scenarios/power_system.toml
- [ ] T036 [P] Run scenario test with node scripts/run-scenario.js power_system.toml

---

## Parallel Execution Examples

### Phase 2 Parallel Tasks

以下のタスクは並列実行可能（ファイルが被らないため）:

```
T006a [US1]: Create power wire block definition
T006b [US1]: Create water wheel generator definition
T006c [US1]: Create coal generator definition
T006d [US1]: Create coal item definition

T003 [US1]: Implement PowerProducer component
T004 [US1]: Implement PowerWire component
T005 [US1]: Implement PowerGrids resource
```

これらはそれぞれ別のファイルに実装するが、独立して実装可能（items.tomlとmachines.toml、src/components/power.rs）。

### Phase 5 Parallel Tasks

以下のタスクは並列実行可能（異なる関数）:

```
T014 [US2]: Add power status section
T015 [US2]: Add grid statistics panel
```

これらは同じファイル（src/systems/ui/machine_ui.rs）に実装するが、異なるUI要素なので独立して実装可能。

---

## File Competition Risk Matrix

| Task | 競合するタスク | 推奨実行順序 |
|-------|------------------|----------------|
| T006a-T006c | T003-T005（Component） | T006a→T006b→T006c→T006d→T003→T004→T005 |
| T006d | T006（exports） | T003-T004-T005完了後 |
| T007-T009 | T010（registration） | T007→T008→T009→T010 |
| T011-T013 | Phase 3タスク | Phase 3完了後 |
| T014-T015 | T016（update） | T014→T015→T016 |
| T020a-T020c | T021-T022（implementation） | T020a→T020b→T020c→T021→T022 |
| T020-T023 | Registration（plugin） | T020a-T020b-T020c完了後 |

---

## Independent Test Criteria Summary

| Phase | ユーザーストーリー | テスト可能な状態 |
|--------|---------------------|------------------|
| Phase 2 | US1 (Basic) | ComponentとResourceが作成済み |
| Phase 3 | US1 (Basic) | グリッド計算が動作、発電量・消費量が計算済み |
| Phase 4 | US1 (Basic) | 無電力時機械が停止する |
| Phase 5 | US2 (UI) | 電力状態と統計がUIに表示される |
| Phase 6 | US3 (Grids) | グリッド分割・統合が正しく動作する |
| Phase 7 | US4 (Fuel) | 燃料消費と起動遅延が動作する |
| Phase 8 | US5 (Dynamic) | 変更時のみグリッド計算、高頻度イベント除外 |
| Phase 9 | 全て | 全ての要件と成功基準を満たす |
| Phase 10 | 全て | システム全体が統合され、シナリオテストが通る |

---

## Implementation Notes

**重要な設計決定**:
1. 既存の`Machine.fuel`フィールドを活用し、新規`PowerFuelSlot` Componentは実装しない
2. `MachineSpec`に電力関連フィールド（`power_output`, `power_consumption`, `fuel_consumption_rate`, `startup_delay`）を追加
3. Union-Findアルゴリズムを使用してグリッド計算の効率化を図る
4. 高頻度イベント（`PowerGridChanged`）は外部Mod通知をOFFにする

**ファイル競合回避**:
- Component実装（T003-T005）はT006（exports）より先に実行
- System実装（T007-T009）はT010（registration）より先に実行
- UI実装（T014-T015）はT016（update）より先に実行

---

## References

- `.claude/architecture.md` - 電力システム設計骨格
- `specs/001-power-system-spec/spec.md` - 機能仕様
- `specs/001-power-system-spec/plan.md` - 実装計画
- `specs/001-power-system-spec/data-model.md` - データモデル
- `specs/001-power-system-spec/research.md` - 研究決定
- `specs/001-power-system-spec/contracts/systems.md` - システム契約
- `specs/001-power-system-spec/quickstart.md` - クイックスタート
