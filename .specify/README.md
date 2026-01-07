# .specify

仕様書・開発ガイド格納フォルダ

## 構造

```
.specify/
├── specs/                  # ゲーム仕様
│   ├── index.md            # ← 入り口
│   ├── core-concept.md     # ゲームコンセプト
│   ├── first-30-minutes.md # 序盤体験
│   └── ui.md               # UI仕様
├── memory/                 # 開発ガイド・原則
│   ├── constitution.md     # ★ プロジェクト憲章（根本原則）
│   ├── input-rules.md      # 入力マトリクス
│   ├── modeling-rules.md   # 3Dモデル作成ルール
│   ├── magicavoxel-guide.md # MagicaVoxel使い方
│   └── ui-design-rules.md  # UIデザインルール
└── roadmap.md              # 実装ロードマップ
```

## 読み方

| 目的 | 参照先 |
|------|--------|
| プロジェクトの根本原則を知りたい | `memory/constitution.md` |
| ゲーム仕様を知りたい | `specs/index.md` |
| 実装ガイド | `memory/` 内の該当ファイル |
| 将来計画 | `roadmap.md` |
| 詳細な将来設計 | `.claude/architecture-future.md` |

## Single Source of Truth

**仕様はコードで定義。ドキュメントは参照用。**

| 種別 | 定義場所 |
|------|----------|
| アイテム | `src/game_spec/registry.rs` |
| 機械 | `src/game_spec/machines.rs` |
| レシピ | `src/game_spec/recipes.rs` |
| クエスト | `src/game_spec/quests.rs` |
