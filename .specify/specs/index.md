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

### 設計原則

| ファイル | 内容 |
|----------|------|
| [../memory/constitution.md](../memory/constitution.md) | **プロジェクト憲章**（根本原則） |

### ゲーム仕様

| ファイル | 内容 | 状態 |
|----------|------|------|
| [core-concept.md](core-concept.md) | ゲームコンセプト・差別化 | 参照用 |
| [first-30-minutes.md](first-30-minutes.md) | 序盤体験フロー | 参照用 |
| [ui.md](ui.md) | UI画面一覧・HUD | 参照用 |

### 開発ガイド

| ファイル | 内容 |
|----------|------|
| [../memory/input-rules.md](../memory/input-rules.md) | 入力マトリクス |
| [../memory/modeling-rules.md](../memory/modeling-rules.md) | 3Dモデル作成ルール |
| [../memory/ui-design-rules.md](../memory/ui-design-rules.md) | UIデザインルール |

## 将来設計

| ファイル | 内容 |
|----------|------|
| [../../.claude/architecture-future.md](../../.claude/architecture-future.md) | 将来アーキテクチャ（17機能の骨格設計） |
| [../roadmap.md](../roadmap.md) | 実装ロードマップ |

## 現在のフェーズ

**Phase C 完了** → **Phase D 準備中**

| Phase | 内容 | 状態 |
|-------|------|------|
| A | v0.2完成（チュートリアル、バイオームUI） | ✅ 完了 |
| B | アーキテクチャ再設計 | ✅ 完了 |
| C | データ駆動設計 | ✅ 完了 |
| D | 基盤強化（マルチ準備、イベント、動的ID） | ⚠️ 次 |

---

*最終更新: 2026-01-07*
