# シナリオテストガイド

このドキュメントでは、idle_factory プロジェクトにおけるシナリオテストの書き方と実行方法を説明します。

## 概要

シナリオテストは、ゲームの状態をプログラム的に検証するためのTOMLベースのテストフレームワークです。WebSocket API（port 9877）を通じてゲームと通信し、入力の注入や状態の検証を行います。

## 実行方法

```bash
node scripts/run-scenario.js tests/scenarios/bug_xxx.toml
```

## テストファイルの構造

```toml
# tests/scenarios/bug_xxx.toml
name = "バグ再現: XXXの問題"
description = "XXXするとYYYになる問題"

[[steps]]
action = "get_state"

[[steps]]
action = "send_input"
params = { action = "ToggleInventory" }

[[steps]]
action = "wait"
params = { ms = 100 }

[[steps]]
action = "assert"
params = { condition = "ui_state == Inventory" }
```

## 利用可能なアクション

| アクション | 説明 | パラメータ |
|-----------|------|-----------|
| `get_state` | ゲーム状態取得 | なし |
| `send_input` | 入力注入 | `action`: GameAction名 |
| `assert` | 状態検証 | `condition`: "field == value" |
| `wait` | 待機 | `ms`: ミリ秒 |

## 利用可能なGameAction

以下のアクションを `send_input` で使用できます：

- **移動**: `MoveForward`, `MoveBackward`, `MoveLeft`, `MoveRight`, `Jump`
- **UI操作**: `ToggleInventory`, `TogglePause`, `ToggleQuest`, `OpenCommand`, `CloseUI`
- **アクション**: `PrimaryAction`, `SecondaryAction`, `RotateBlock`
- **ホットバー**: `Hotbar1`, `Hotbar2`, `Hotbar3`, `Hotbar4`, `Hotbar5`, `Hotbar6`, `Hotbar7`, `Hotbar8`, `Hotbar9`

## 検証可能なフィールド

| フィールド | 型 | 例 |
|-----------|-----|-----|
| `ui_state` | String | `Gameplay`, `Inventory`, `PauseMenu` |
| `cursor_locked` | bool | `true`, `false` |
| `player_position` | [f32;3] | `[0.0, 10.0, 0.0]` |

## 例: バグ再現テスト

```toml
name = "バグ再現: インベントリが開かない"
description = "Eキーを押してもインベントリが開かない問題"

[[steps]]
action = "assert"
params = { condition = "ui_state == Gameplay" }

[[steps]]
action = "send_input"
params = { action = "ToggleInventory" }

[[steps]]
action = "wait"
params = { ms = 100 }

[[steps]]
action = "assert"
params = { condition = "ui_state == Inventory" }
```

## 例: 機能テスト

```toml
name = "機能テスト: インベントリ開閉"
description = "Eキーでインベントリが開閉できることを確認"

[[steps]]
action = "assert"
params = { condition = "ui_state == Gameplay" }

[[steps]]
action = "send_input"
params = { action = "ToggleInventory" }

[[steps]]
action = "wait"
params = { ms = 100 }

[[steps]]
action = "assert"
params = { condition = "ui_state == Inventory" }

[[steps]]
action = "send_input"
params = { action = "ToggleInventory" }

[[steps]]
action = "wait"
params = { ms = 100 }

[[steps]]
action = "assert"
params = { condition = "ui_state == Gameplay" }
```

## ベストプラクティス

1. **各テストは独立して実行可能にする** - 他のテストの結果に依存しない
2. **waitは必要最小限に** - 100msで十分な場合が多い
3. **失敗条件を明確に** - 何が失敗したか分かるようにassertを書く
4. **再現性を確保** - 初期状態をassertで確認してからテストを開始する
