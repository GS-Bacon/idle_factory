# 仕様インデックス

## ゲーム概要

- **ジャンル**: 3Dボクセル工場自動化ゲーム
- **技術**: Rust + Bevy 0.15, YAML (データ), Lua 5.4 (スクリプト)
- **コンセプト**: ストレスフリー、戦闘なし、自動化特化

## コア仕様

| 項目 | 内容 |
|------|------|
| プレイヤー | HP/空腹なし、落下ダメージなし、インベントリ40スロット |
| ワールド | 無限XY、高さ±256、チャンク32³ |
| 電力 | 電気のみ、過負荷で減速（壊れない） |
| 目標 | 宇宙ステーション建設 |

## 仕様ファイル

| ファイル | 内容 |
|----------|------|
| [core-concept.md](core-concept.md) | ゲームコンセプト |
| [editor.md](editor.md) | エディタ仕様（確定） |
| [ui.md](ui.md) | UI仕様 |
| [first-30-minutes.md](first-30-minutes.md) | 序盤体験 |
| mechanics/ | 個別メカニクス仕様 |

## メカニクス仕様

| ファイル | 内容 |
|----------|------|
| player.md | プレイヤー |
| conveyor.md | コンベア |
| power.md | 電力システム |
| fluid.md | 液体・パイプ |
| signal.md | 信号・回路 |
| quest.md | クエスト |
| worldgen.md | ワールド生成 |
| interaction.md | インタラクション |
| miner.md | 採掘機 |
| smelter.md | 精錬炉 |
| multiblock.md | マルチブロック |
| blueprint.md | ブループリント |
| enchant.md | エンチャント |
| progression.md | 進行システム |
| platform.md | 納品プラットフォーム |

## 将来機能

→ [../roadmap.md](../roadmap.md) 参照
