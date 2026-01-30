# Implementation Plan: M3 Power System

**Branch**: `001-power-system-spec` | **Date**: 2026-01-30 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-power-system-spec/spec.md`

## Summary

M3電力システムは、発電機（水車・石炭発電機）から電線を通じて機械に電力を供給するシステムを実装する。複数の独立した電力グリッドをサポートし、電力不足時に機械が停止する。電力状態をUIで視覚的に表示し、グリッド統計（総発電量、消費量、余剰/不足）を提供する。

**重要な設計決定**:
- 既存の`MachineSlots.fuel`を活用し、新規`PowerFuelSlot` Componentは実装しない
- `MachineSpec`に電力関連フィールド（`power_output`, `power_consumption`, `fuel_consumption_rate`, `startup_delay`）を追加

## Technical Context

**Language/Version**: Rust 1.75+
**Primary Dependencies**: Bevy 0.14+, Wasmtime (既に使用)
**Storage**: Bevy ECS components + セーブファイル（V2形式）
**Testing**: cargo test（既存のシナリオテスト）
**Target Platform**: Linux, Windows, macOS
**Project Type**: Desktop game (Bevy ECS)
**Performance Goals**: 20 tick/秒（FixedUpdate）、1グリッド100台以上の機械でグリッド計算を50ms以内に完了
**Constraints**: 電力伝播は即時、グリッド計算は1チック（50ms）以内
**Scale/Scope**: 1グリッド100台以上、複数独立グリッド対応

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**参照**: `.specify/memory/constitution.md` (Version 1.0.0)

### 原則チェック

| 原則 | チェック項目 | パス基準 |
|------|-------------|----------|
| ゲーム進行優先 | この機能はゲームが遊べるようになるか？ | YES（電力システムは機械動作に必須） |
| 事前調査義務 | ベストプラクティス調査完了済みか？ | YES（Phase 0で実施：Union-Find、機械統合、イベント統合、UI実装） |
| ライブラリ導入チェック | 新規ライブラリのメンテナンス・セキュリティチェック済みか？ | YES（新規ライブラリは追加しない） |

### バグ修正の場合

N/A（機能追加）

### アーキテクチャ整合性

| チェック項目 | パス基準 |
|-------------|----------|
| 設計参照 | `.claude/architecture.md` を参照したか？ | YES（電力システム設計を確認） |
| 矛盾チェック | 既存アーキテクチャと矛盾していないか？ | YES（Component化、イベントシステム、FixedUpdateに準拠） |

### 憲法違反がある場合

なし

## Project Structure

### Documentation (this feature)

```text
specs/001-power-system-spec/
├── spec.md              # Feature specification (completed)
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (created below)
├── data-model.md        # Phase 1 output (created below)
├── quickstart.md        # Phase 1 output (created below)
├── contracts/           # Phase 1 output (created below)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── components/
│   ├── machines/mod.rs         # 既存
│   └── power.rs               # 新規：PowerProducer, PowerWire, PowerGrids resource
├── logistics/
│   └── power_grid.rs          # 新規：電力グリッド計算システム
├── events/
│   └── game_events.rs         # 更新：電力関連イベント追加
├── systems/
│   ├── power/                 # 新規：電力関連システム
│   │   ├── mod.rs
│   │   ├── grid_calc.rs       # グリッド計算
│   │   ├── generator_tick.rs   # 発電機tick処理
│   │   └── consumer_tick.rs    # 消費機械tick処理
│   └── ui/machine_ui.rs       # 更新：電力UI表示追加
└── game_spec/
    └── machines.rs            # 更新：発電機定義追加

mods/base/
├── items.toml                # 更新：電線ブロック、燃料アイテム
└── machines.toml             # 更新：水車、石炭発電機定義

tests/scenarios/
└── power_system.toml          # 新規：電力システムシナリオテスト
```

**Structure Decision**: 既存の`src/components/`、`src/logistics/`、`src/systems/`構造に従う。電力システムは独立モジュールとして実装し、既存の機械システムとイベント経由で統合する。

## Complexity Tracking

なし（憲法違反なし）
