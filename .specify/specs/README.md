# 仕様書インデックス

## コア

| ファイル | 内容 |
|----------|------|
| [core-concept.md](core-concept.md) | コアコンセプト、差別化、ターゲット |
| [first-30-minutes.md](first-30-minutes.md) | 最初の30分のプレイフロー |

## UI

| ファイル | 内容 |
|----------|------|
| [ui.md](ui.md) | 画面一覧、HUD、各UI画面 |

## メカニクス

| ファイル | 内容 |
|----------|------|
| [mechanics/player.md](mechanics/player.md) | プレイヤー（HP無し、移動手段、ツール） |
| [mechanics/interaction.md](mechanics/interaction.md) | クリック操作、レンチ機能 |
| [mechanics/machines-index.md](mechanics/machines-index.md) | 全機械一覧 |
| [mechanics/miner.md](mechanics/miner.md) | 採掘機（無限採掘） |
| [mechanics/smelter.md](mechanics/smelter.md) | 精錬炉 |
| [mechanics/conveyor.md](mechanics/conveyor.md) | コンベア（両側レーン、合流・分配） |
| [mechanics/platform.md](mechanics/platform.md) | 納品プラットフォーム |
| [mechanics/power.md](mechanics/power.md) | 電力システム（電気のみ） |
| [mechanics/fluid.md](mechanics/fluid.md) | パイプ・液体システム |
| [mechanics/signal.md](mechanics/signal.md) | 信号・Luaスクリプト |
| [mechanics/multiblock.md](mechanics/multiblock.md) | マルチブロック機械 |
| [mechanics/worldgen.md](mechanics/worldgen.md) | ワールド生成・バイオーム |
| [mechanics/quest.md](mechanics/quest.md) | クエストシステム |
| [mechanics/progression.md](mechanics/progression.md) | 進行・アンロック |

## エディタ

| ファイル | 内容 |
|----------|------|
| [editor.md](editor.md) | エディタ連携仕様 |

## アーカイブ

| ファイル | 内容 |
|----------|------|
| [refactoring-plan.md](refactoring-plan.md) | 仕様詰め直し計画・議論メモ |
| [index-compact.md](index-compact.md) | 旧仕様（AI圧縮版、参考用） |
| [ai-feedback-loop.md](ai-feedback-loop.md) | AIフィードバックループ仕様 |

## 更新履歴

- 2025-12-26: 全仕様を詳細化
  - クエスト、進行、パイプ、信号、マルチブロック、ワールド生成、エディタ追加
  - 運動エネルギー削除、電気のみに統一
  - ワイヤーをブロック単位配置に変更
- 2025-12-26: 仕様を分割して再整理
