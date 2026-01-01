# ゲーム状態アサーション

ゲーム内の `/assert` コマンドを使用して、インベントリや機械状態を検証する。

## 引数
$ARGUMENTS

## 概要

このスキルは、ゲーム内の状態が期待値と一致するかを検証し、自動テストやデバッグに使用します。

## 使用方法

### 1. ゲームを起動

```bash
./run.sh
```

### 2. コマンド入力

ゲーム内で `T` または `/` キーを押してコマンドバーを開き、以下を入力:

```
/assert <condition>
```

## 利用可能なアサーション

| コマンド | 内容 | 例 |
|----------|------|-----|
| `inventory_contains <item> <count>` | インベントリにアイテムがあるか | `/assert inventory_contains iron_ingot 10` |
| `inventory_empty` | インベントリが空か | `/assert inventory_empty` |
| `machine_working <x> <y> <z>` | 指定座標の機械が動作中か | `/assert machine_working 10 5 10` |
| `machine_idle <x> <y> <z>` | 指定座標の機械が待機中か | `/assert machine_idle 10 5 10` |
| `conveyor_has_item <x> <y> <z>` | コンベア上にアイテムがあるか | `/assert conveyor_has_item 10 5 11` |
| `quest_completed <id>` | クエストが完了しているか | `/assert quest_completed 1` |

## 出力

- **成功**: `[ASSERT OK] condition`
- **失敗**: `[ASSERT FAIL] condition - expected: X, actual: Y`

## 実装状況

**現在**: 未実装（タスク #15）

実装後の期待動作:
1. 条件をパース
2. ゲーム状態を取得
3. 比較して結果をログ出力
4. E2Eテストスクリプトと連携可能

## 関連ファイル

- `src/systems/command_ui.rs` - コマンド処理
- `src/player/inventory.rs` - インベントリ
- `.claude/refactoring-tasks.md` - タスク #15

## 使用例: E2Eテストとの連携

```bash
# シナリオ実行後にアサーション
/test miner_chain
# 30秒待機
/assert inventory_contains iron_ingot 5
```
