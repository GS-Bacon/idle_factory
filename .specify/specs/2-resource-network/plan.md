# Implementation Plan: Resource Networks

**Branch**: `2-resource-network` | **Date**: 2026-01-30 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specify/specs/2-resource-network/spec.md`

## Summary

リソースネットワーク機能は、電力（発電機→電線→機械）、流体（ポンプ→パイプ→タンク→機械）、信号（センサー→信号線→論理ゲート→レシーバー）の3つのネットワークシステムを実装する。各ネットワークは共通の `NetworkGraph<K, V>` 基盤を活用し、独立して動作する。

**重要な設計決定**:
- Union-Find アルゴリズムを全3ネットワークに使用
- IoPort 拡張で `PortType::Fluid`, `Power`, `Signal` を追加
- MachineSpec に電力・流体・信号関連フィールドを追加

## Technical Context

**Language/Version**: Rust 1.75+
**Primary Dependencies**: Bevy 0.15+, Wasmtime (既に使用)
**Storage**: Bevy ECS components + セーブファイル（V2形式）
**Testing**: cargo test（既存のシナリオテスト）
**Target Platform**: Linux, Windows, macOS
**Project Type**: Desktop game (Bevy ECS)
**Performance Goals**: 20 tick/秒（FixedUpdate）、1グリッド100ノード以上で計算を50ms以内に完了
**Constraints**: 電力・流体・信号伝播は即時、グリッド計算は1チック（50ms）以内
**Scale/Scope**: 1グリッド100ノード以上、複数独立グリッド対応

## Project Structure

### Documentation (this feature)

 ```
 .specify/specs/2-resource-network/
 ├── spec.md              # Feature specification (completed)
 ├── plan.md              # This file
 ├── research.md          # Research results
 ├── data-model.md        # Data model design
 ├── tasks/              # Task lists by phase
 │   ├── phase0.md       # Phase 0: 共通基盤の拡張 (38 tasks)
 │   ├── phase1.md       # Phase 1: 電力ネットワーク実装 (TBD)
 │   ├── phase2.md       # Phase 2: 流体ネットワーク実装 (TBD)
 │   ├── phase3.md       # Phase 3: 信号ネットワーク実装 (TBD)
 │   ├── phase4.md       # Phase 4: UIと統合 (TBD)
 │   ├── phase5.md       # Phase 5: イベントとセーブ/ロード (TBD)
 │   └── phase6.md       # Phase 6: テストと完了 (TBD)
 ├── quickstart.md        # Quickstart guide
 ├── contracts/           # Component contracts
 └── checklists/
     └── requirements.md # Specification quality checklist
 ```

### Source Code (repository root)

```
src/
├── components/
│   ├── machines/
│   │   └── ports.rs           # 更新：IoPort拡張
│   ├── power.rs                # 新規：PowerProducer, PowerConsumer, PowerWire
│   ├── fluids.rs               # 新規：FluidSource, FluidDrain, Pipe, Tank
│   └── signals.rs              # 新規：SignalEmitter, SignalReceiver, LogicGate, SignalInput
├── core/
│   ├── id.rs                  # 更新：GridId追加
│   └── network.rs             # 更新：PowerNetwork, FluidNetwork, SignalNetwork type aliases
├── logistics/
│   ├── power_grid.rs          # 新規：電力グリッド計算システム
│   ├── pipe.rs                # 新規：流体ネットワークシステム
│   ├── signals.rs             # 新規：信号ネットワークシステム
│   └── network_utils.rs       # 新規：Union-Find 共通ユーティリティ
├── game_spec/
│   ├── machines.rs             # 更新：電力・流体・信号フィールド追加
│   └── fluids.rs              # 新規：FluidSpec定義
├── machines/
│   └── generic/
│       ├── tick.rs             # 更新：ネットワーク状態チェック追加
│       └── ui.rs               # 更新：ネットワーク状態表示追加
├── save/
│   └── format/
│       └── v2/
│           └── networks.rs     # 新規：セーブ/ロードフォーマット
├── ui/
│   └── network_stats.rs       # 新規：ネットワーク統計UI
└── events/
    └── game_events.rs         # 更新：電力・流体・信号イベント追加
```

---

## Implementation Phases

### Phase 0: 共通基盤の拡張

**Goal**: 既存の `NetworkGraph<K, V>` を活用し、共通ユーティリティを作成する。

**詳細設計**: [phase0.md](./phase0.md)

| タスク | ファイル | 説明 |
|------|---------|------|
| T0-1 | `src/core/id.rs` | `GridId` 型を定義 |
| T0-2 | `src/core/network.rs` | `SignalNetwork` 型エイリアスを定義（PowerNetwork/FluidNetworkは既存） |
| T0-3 | `src/logistics/network_utils.rs` | `NetworkUnionFind` 共通ユーティリティを実装 |
| T0-4 | `src/components/machines/ports.rs` | `PortType` enum を新規に作成し `IoPort` に追加 |

---

### Phase 1: 電力ネットワーク実装

**Goal**: 発電機→電線→機械の電力供給システムを実装する。

**詳細設計**: [phase1.md](./phase1.md)

| タスク | ファイル | 説明 |
|------|---------|------|
| T1-1 | `src/components/power.rs` | `PowerProducer`, `PowerConsumer`, `PowerWire` Component を定義 |
| T1-2 | `src/logistics/power_grid.rs` | `PowerNetworks` Resource とグリッド計算システムを実装 |
| T1-3 | `src/game_spec/machines.rs` | `power_consumption`, `power_output`, `fuel_consumption_rate` フィールド追加 |
| T1-4 | `src/machines/generic/tick.rs` | `generic_machine_tick()` の冒頭で電力チェック追加 |
| T1-5 | `src/machines/generic/ui.rs` | 電力状態（powered/unpowered）表示追加 |

---

### Phase 2: 流体ネットワーク実装

**Goal**: ポンプ→パイプ→タンク→機械の流体供給システムを実装する。

**詳細設計**: [phase2.md](./phase2.md)

| タスク | ファイル | 説明 |
|------|---------|------|
| T2-1 | `src/game_spec/fluids.rs` | `FluidSpec` と定義済み流体（水, 溶岩, 蒸気）を定義 |
| T2-2 | `src/components/fluids.rs` | `FluidSource`, `FluidDrain`, `Pipe`, `Tank` Component を定義 |
| T2-3 | `src/logistics/pipe.rs` | `FluidNetworks` Resource と流体転送システムを実装 |
| T2-4 | `src/game_spec/machines.rs` | `fluid_consumption`, `required_fluid`, `required_temperature` フィールド追加 |
| T2-5 | `src/machines/generic/tick.rs` | 流体チェック追加 |
| T2-6 | `src/machines/generic/ui.rs` | 流体状態（量、温度）表示追加 |

---

### Phase 3: 信号ネットワーク実装

**Goal**: センサー→信号線→論理ゲート→レシーバーの信号制御システムを実装する。

**詳細設計**: [phase3.md](./phase3.md)

| タスク | ファイル | 説明 |
|------|---------|------|
| T3-1 | `src/components/signals.rs` | `SignalEmitter`, `SignalReceiver`, `SignalWire`, `LogicGate`, `SignalInput` Component を定義 |
| T3-2 | `src/logistics/signals.rs` | `SignalNetworks` Resource と信号伝播システムを実装 |
| T3-3 | `src/game_spec/machines.rs` | `signal_input` フィールド追加 |
| T3-4 | `src/machines/generic/tick.rs` | 信号入力チェック追加 |
| T3-5 | `src/machines/generic/ui.rs` | 信号状態（ON/OFF）表示追加 |

---

### Phase 4: UIと統合

**Goal**: ネットワーク状態をUIで視覚化し、プレイヤーにフィードバックを提供する。

**詳細設計**: [phase4.md](./phase4.md)

| タスク | ファイル | 説明 |
|------|---------|------|
| T4-1 | `src/ui/network_stats.rs` | ネットワーク統計UI（発電量、消費量、余剰/不足、流体流量、信号状態）を実装 |
| T4-2 | `src/plugins/ui.rs` | ネットワーク統計UIをシステムに登録 |

---

### Phase 5: イベントとセーブ/ロード

**Goal**: ネットワークの状態変化をイベントで通知し、セーブ/ロードをサポートする。

**詳細設計**: [phase5.md](./phase5.md)

| タスク | ファイル | 説明 |
|------|---------|------|
| T5-1 | `src/events/game_events.rs` | 電力・流体・信号関連イベントを追加 |
| T5-2 | `src/save/format/v2/networks.rs` | セーブ/ロードフォーマットを定義 |
| T5-3 | `src/save/systems.rs` | セーブ/ロードシステムにネットワークデータ処理を追加 |

---

### Phase 6: テストと完了

**Goal**: 単体テスト・統合テスト・シナリオテストで機能を検証する。

**詳細設計**: [phase6.md](./phase6.md)

| タスク | ファイル | 説明 |
|------|---------|------|
| T6-1 | `src/logistics/power_grid/tests.rs` | 電力ネットワークの単体テスト |
| T6-2 | `src/logistics/pipe/tests.rs` | 流体ネットワークの単体テスト |
| T6-3 | `src/logistics/signals/tests.rs` | 信号ネットワークの単体テスト |
| T6-4 | `tests/scenarios/` | シナリオテストを作成（power_basic, fluid_basic, signal_basic, integrated_networks） |
| T6-5 | すべて | 統合テスト（3つのネットワークが独立して動作することを確認） |

---

## Dependencies

### 外部依存

なし（既存の依存関係のみ）

### 内部依存

- **Phase 0**: 既存の `NetworkGraph<K, V>`
- **Phase 1**: Phase 0, 既存の `Machine` Component
- **Phase 2**: Phase 0, Phase 1, 既存の `Machine` Component
- **Phase 3**: Phase 0, 既存の `Machine` Component
- **Phase 4**: Phase 1-3, 既存の UI システム
- **Phase 5**: Phase 1-3, 既存のイベントシステム、セーブシステム
- **Phase 6**: Phase 0-5

---

## Risk Management

| リスク | 影響 | 軽減策 |
|--------|--------|----------|
| Union-Find で複雑なグラフ分割/統合が正しく動作しない | 高 | 電力システムの実験で既に検証済み。シナリオテストで徹底的にテスト |
| 3つのネットワークが相互干渉する | 中 | 独立した `NetworkGraph` インスタンスを使用。統合テストで分離性を確認 |
| パフォーマンスが50ms/tickを超える | 中 | ベンチマークを実施。必要に応じて最適化（空間分割など） |
| セーブ/ロードで不明IDが発生する | 低 | 不明IDは警告＋デフォルト値でフォールバック。テストで検証 |

---

## Acceptance Criteria

- [ ] **Phase 0**: 共通基盤が拡張され、型エイリアスとユーティリティが使用可能
- [ ] **Phase 1**: 電力ネットワークが動作し、発電機→電線→機械の電力供給が可能
- [ ] **Phase 2**: 流体ネットワークが動作し、ポンプ→パイプ→タンク→機械の流体供給が可能
- [ ] **Phase 3**: 信号ネットワークが動作し、センサー→信号線→論理ゲート→レシーバーの制御が可能
- [ ] **Phase 4**: UI でネットワーク状態が視覚化され、プレイヤーにフィードバックが提供される
- [ ] **Phase 5**: ネットワークの状態変化がイベントで通知され、セーブ/ロードが正常に動作する
- [ ] **Phase 6**: 全てのテストがパスし、3つのネットワークが独立して動作することが確認される
- [ ] **パフォーマンス**: 1グリッド100ノード以上で計算が50ms以内に完了する
- [ ] **品質**: `cargo build && cargo test && cargo clippy` が成功し、警告0件

---

## References

- [Specification](./spec.md) - 機能仕様書
- [Research](./research.md) - 調査結果
- [Data Model](./data-model.md) - データモデル
- [Architecture Design](../../.claude/architecture.md) - システムアーキテクチャ
- [Core Network Implementation](../../src/core/network.rs) - NetworkGraph 実装
- [Task Lists](./tasks/) - 各フェーズのタスクリスト
  - [Phase 0: 共通基盤の拡張](./tasks/phase0.md) - 38タスク (4.0時間)
  - [Phase 1: 電力ネットワーク実装](./tasks/phase1.md) - 計画中
  - [Phase 2: 流体ネットワーク実装](./tasks/phase2.md) - 計画中
  - [Phase 3: 信号ネットワーク実装](./tasks/phase3.md) - 計画中
  - [Phase 4: UIと統合](./tasks/phase4.md) - 計画中
  - [Phase 5: イベントとセーブ/ロード](./tasks/phase5.md) - 計画中
  - [Phase 6: テストと完了](./tasks/phase6.md) - 計画中

---

*最終更新: 2026-01-30*
