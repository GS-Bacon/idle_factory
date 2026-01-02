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
| `inventory <item> <min_count>` | インベントリにアイテムが最低N個あるか | `/assert inventory iron_ingot 10` |
| `slot <index> <item> <count>` | 特定スロットにアイテムがN個以上あるか | `/assert slot 0 coal 5` |
| `machine miner working` | マイナーが稼働中か確認 | `/assert machine miner working` |
| `machine conveyor items` | コンベアにアイテムがあるか確認 | `/assert machine conveyor items` |
| `machine <type> count <min>` | 機械の設置数を確認 | `/assert machine miner count 3` |

## 出力

- **成功**: `✓ PASS: <item> >= <count> (actual: <actual>)`
- **失敗**: `✗ FAIL: <item> < <count> (actual: <actual>)`

## 実装状況

**現在**: ✅ 実装済み

動作:
1. 条件をパース（inventory または slot）
2. ゲーム状態を取得
3. 比較して結果をログ出力（✓ PASS / ✗ FAIL）

## 関連ファイル

- `src/systems/command/executor.rs:262-317` - コマンド処理
- `src/player/inventory.rs` - インベントリ

## 使用例

```bash
# 生産ラインをセットアップ
/test production

# 30秒待機後、インベントリを検証
/assert inventory iron_ingot 5

# 特定スロットを検証
/assert slot 0 coal 10

# 機械の稼働状態を確認
/assert machine miner working
/assert machine conveyor items
/assert machine miner count 3
```
