# 仕様インデックス

## ゲーム概要

| 項目 | 内容 |
|------|------|
| **ジャンル** | 3Dボクセル工場自動化ゲーム |
| **技術** | Rust + Bevy 0.15 |
| **コンセプト** | ストレスフリー、戦闘なし、自動化特化 |
| **目標** | 宇宙ステーション建設 |

## コア仕様

| 項目 | 内容 |
|------|------|
| プレイヤー | HP/空腹なし、落下ダメージなし、インベントリ40スロット |
| ワールド | 無限XY、高さ±256、チャンク32³ |
| 電力 | 過負荷で減速（壊れない） |
| 操作 | WASD + マウス（Minecraft風） |

## データ定義（Single Source of Truth）

**仕様はコードで定義。ドキュメントは参照用。**

| 種別 | 定義場所 |
|------|----------|
| アイテム | `src/game_spec/registry.rs` |
| 機械 | `src/game_spec/machines.rs` |
| レシピ | `src/game_spec/recipes.rs` |
| クエスト | `src/game_spec/quests.rs` |

## 仕様ファイル

| ファイル | 内容 |
|----------|------|
| [../memory/constitution.md](../memory/constitution.md) | **プロジェクト憲章**（根本原則・禁止事項） |
| [core-concept.md](core-concept.md) | ゲームコンセプト・差別化 |
| [first-30-minutes.md](first-30-minutes.md) | 序盤体験フロー |
| [ui.md](ui.md) | UI画面一覧・HUD |
| [modding.md](modding.md) | Mod概要（できること/できないこと） |
| [modding-api.md](modding-api.md) | WASM APIリファレンス |

## 開発ガイド

| ファイル | 内容 |
|----------|------|
| [../memory/input-rules.md](../memory/input-rules.md) | 入力マトリクス |
| [../memory/modeling-rules.md](../memory/modeling-rules.md) | 3Dモデル作成ルール |
| [../memory/ui-design-rules.md](../memory/ui-design-rules.md) | UIデザインルール |

## 将来設計・ロードマップ

| ファイル | 内容 |
|----------|------|
| [../../.claude/architecture.md](../../.claude/architecture.md) | 将来アーキテクチャ |
| [../roadmap.md](../roadmap.md) | 実装ロードマップ |

## 現在の状態

**M2完了** → **M3準備中**（電力システム）

詳細は [roadmap.md](../roadmap.md) 参照

---

*最終更新: 2026-01-30*
