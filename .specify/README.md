# .specify

仕様書・開発ガイド格納フォルダ

## 構造

```
.specify/
├── templates/
│   ├── spec-template.md       # 仕様テンプレート
│   ├── plan-template.md       # 実装計画テンプレート
│   ├── checklist-template.md  # チェックリストテンプレート
│   └── tasks-template.md      # タスク管理テンプレート
├── specs/                     # ゲーム仕様
│   ├── index.md               # ← 入り口
│   ├── core-concept.md        # ゲームコンセプト
│   ├── first-30-minutes.md    # 序盤体験
│   ├── ui.md                  # UI仕様
│   ├── modding.md             # Mod概要
│   └── modding-api.md         # WASM APIリファレンス
├── memory/                    # 開発ガイド・原則
│   ├── constitution.md        # ★ プロジェクト憲章（禁止事項・根本原則）
│   ├── input-rules.md         # 入力マトリクス
│   ├── modeling-rules.md      # 3Dモデル作成ルール
│   ├── magicavoxel-guide.md   # MagicaVoxel使い方
│   └── ui-design-rules.md     # UIデザインルール
└── roadmap.md                 # 実装ロードマップ
```

## 読み方

| 目的 | 参照先 |
|------|--------|
| プロジェクトの根本原則・禁止事項 | `memory/constitution.md` |
| ゲーム仕様を知りたい | `specs/index.md` |
| Mod仕様 | `specs/modding.md` → `specs/modding-api.md` |
| 実装ガイド | `memory/` 内の該当ファイル |
| 将来計画 | `roadmap.md` |
| 詳細な将来設計 | `.claude/architecture.md` |

## テンプレート一覧

| テンプレート | 用途 | 説明 |
|-------------|------|------|
| `spec-template.md` | 仕様書 | 機能の要件・スコープ定義 |
| `plan-template.md` | 実装計画 | フェーズ分割・ファイル一覧 |
| `checklist-template.md` | チェックリスト | 実装完了確認用 |
| `tasks-template.md` | タスク管理 | 進捗追跡用 |

### GLM対応のポイント

1. **禁止事項は最上部に配置** - AIが見落としやすいため `⛔` マーク付きで冒頭に
2. **検証コマンド付き** - `grep` で確認可能な形式で記載
3. **フェーズ分割** - 一括実装ではなく段階的に検証
4. **継承** - plan/checklist/tasksでも禁止事項を参照するよう明記

### テンプレート使用フロー

```
1. spec-template.md → 仕様定義（禁止事項・要件）
2. plan-template.md → 実装計画（フェーズ分割）
3. tasks-template.md → タスク分解（進捗管理）
4. checklist-template.md → 完了確認（最終チェック）
```

## Single Source of Truth

**仕様はコードで定義。ドキュメントは参照用。**

| 種別 | 定義場所 |
|------|----------|
| アイテム | `src/game_spec/registry.rs` |
| 機械 | `src/game_spec/machines.rs` |
| レシピ | `src/game_spec/recipes.rs` |
| クエスト | `src/game_spec/quests.rs` |
