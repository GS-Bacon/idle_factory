# 並列実行計画テンプレート

このテンプレートを使って並列実行可能な計画を立てる。

## フォーマット

```markdown
# 計画: [タスク名]

## 概要
1-2行で何をするか

## Phase 1: 調査（並列、worktree不要）

| ID | 調査内容 | 担当エージェント |
|----|----------|------------------|
| R1 | [調査内容] | Explore |
| R2 | [調査内容] | Explore |
| R3 | [調査内容] | Explore |

## Phase 2: 実装（並列、worktree内）

### 依存グラフ
```
I1 ─┬─→ I2 ─┬─→ I5
    └─→ I3 ─┤
I4 ────────→┘
```

### タスク詳細

| ID | ファイル | 作業内容 | 依存 | 並列グループ |
|----|----------|----------|------|--------------|
| I1 | src/xxx.rs | 新規: 型定義 | - | A |
| I2 | src/yyy.rs | 修正: 実装追加 | I1 | B |
| I3 | src/zzz.rs | 修正: 統合 | I1 | B |
| I4 | tests/xxx.rs | 新規: テスト | - | A |
| I5 | src/mod.rs | 修正: mod追加 | I2,I3 | C |

### 並列グループ実行順序

1. **Group A** (依存なし): I1, I4 を同時実行
2. **Group B** (A完了後): I2, I3 を同時実行
3. **Group C** (B完了後): I5 を実行

## Phase 3: 検証（直列）

- [ ] cargo build
- [ ] cargo test
- [ ] cargo clippy

## エラー時の対応

- ビルドエラー: 担当ファイルのエージェントが修正
- テスト失敗: 関連ファイルのエージェントが修正
- 型不整合: I1担当が修正後、依存タスク再実行
```

## ルール

### 調査フェーズ (Phase 1)

- **worktree不要**: masterを読むだけ
- **全て並列実行可能**: 依存関係なし
- **Task(Explore)を使用**: 読み取り専用
- **結果を集約してから実装フェーズへ**

### 実装フェーズ (Phase 2)

- **1つのworktree内で並列編集**
- **同一ファイルは1エージェント限定**
- **依存関係を明示**: 型定義→使用の順
- **ビルドチェックはPhase 3で**: サブエージェントはビルドしない

### 依存関係の決め方

| パターン | 依存関係 |
|----------|----------|
| 新規型を定義 → 使う | 定義が先 |
| mod.rs にexport追加 | 実装が先、mod.rsは最後 |
| テスト追加 | 実装と並列可能 |
| 独立したファイル | 並列可能 |

### 並列グループの作り方

1. 依存なしタスクを **Group A** に
2. Group A に依存するタスクを **Group B** に
3. 以下繰り返し
4. **同一グループ内は全て並列実行**

## 例: UIリファクタ

```markdown
# 計画: UI表示制御の統一

## 概要
UI表示/非表示のロジックを一箇所に集約する

## Phase 1: 調査（並列）

| ID | 調査内容 | 担当 |
|----|----------|------|
| R1 | 現在のUI表示ロジックを調査 | Explore |
| R2 | InputStateの使われ方を調査 | Explore |
| R3 | 機械UIの表示パターンを調査 | Explore |

## Phase 2: 実装（並列）

### 依存グラフ
```
I1 ─┬─→ I2 ─┬─→ I5
    └─→ I3 ─┘
I4 ────────→ I5
```

### タスク詳細

| ID | ファイル | 作業内容 | 依存 | Group |
|----|----------|----------|------|-------|
| I1 | src/ui/visibility.rs | 新規: UiVisibility型 | - | A |
| I2 | src/systems/ui_visibility.rs | 修正: 表示システム | I1 | B |
| I3 | src/ui/machine_ui.rs | 修正: 機械UIを統合 | I1 | B |
| I4 | tests/ui_visibility_test.rs | 新規: テスト | - | A |
| I5 | src/ui/mod.rs | 修正: mod追加 | I2,I3,I4 | C |

### 実行順序

1. Group A: I1, I4 (並列)
2. Group B: I2, I3 (並列)
3. Group C: I5 (直列)

## Phase 3: 検証

- [ ] cargo build
- [ ] cargo test
- [ ] cargo clippy
```

## JSON形式（スクリプト連携用）

計画が完成したら `.claude/current-plan.json` に変換:

```json
{
  "name": "UI表示制御の統一",
  "worktree": "refactor-ui-visibility",
  "phases": [
    {
      "name": "調査",
      "type": "investigate",
      "parallel": true,
      "tasks": [
        {"id": "R1", "prompt": "現在のUI表示ロジックを調査", "agent": "Explore"},
        {"id": "R2", "prompt": "InputStateの使われ方を調査", "agent": "Explore"},
        {"id": "R3", "prompt": "機械UIの表示パターンを調査", "agent": "Explore"}
      ]
    },
    {
      "name": "実装",
      "type": "implement",
      "parallel": true,
      "groups": [
        {
          "id": "A",
          "tasks": [
            {"id": "I1", "file": "src/ui/visibility.rs", "action": "create", "prompt": "UiVisibility型を定義"},
            {"id": "I4", "file": "tests/ui_visibility_test.rs", "action": "create", "prompt": "テストを作成"}
          ]
        },
        {
          "id": "B",
          "depends_on": ["A"],
          "tasks": [
            {"id": "I2", "file": "src/systems/ui_visibility.rs", "action": "modify", "prompt": "表示システムを実装"},
            {"id": "I3", "file": "src/ui/machine_ui.rs", "action": "modify", "prompt": "機械UIを統合"}
          ]
        },
        {
          "id": "C",
          "depends_on": ["B"],
          "tasks": [
            {"id": "I5", "file": "src/ui/mod.rs", "action": "modify", "prompt": "mod追加"}
          ]
        }
      ]
    },
    {
      "name": "検証",
      "type": "verify",
      "parallel": false,
      "commands": ["cargo build", "cargo test", "cargo clippy"]
    }
  ]
}
```
