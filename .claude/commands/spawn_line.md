# 機械ライン一括配置

ゲーム内の `/spawn_line` コマンドを使用して、テスト用の機械ラインを一発で配置する。

## 引数
$ARGUMENTS

## 概要

このスキルは、テストやデバッグのために、事前定義された機械ラインを一括で配置します。手動で機械を1つずつ置く手間を省きます。

## 使用方法

### 1. ゲームを起動

```bash
./run.sh
```

### 2. コマンド入力

ゲーム内で `T` または `/` キーを押してコマンドバーを開き、以下を入力:

```
/spawn_line <preset> [x] [y] [z]
```

## 利用可能なプリセット

| プリセット | 内容 | 機械数 |
|-----------|------|--------|
| `iron_basic` | Miner(鉄鉱石) → Conveyor×5 → Furnace | 7 |
| `iron_crusher` | Miner → Crusher → Furnace（粉砕ルート） | 8 |
| `copper_basic` | Miner(銅鉱石) → Conveyor×5 → Furnace | 7 |
| `dual_line` | 鉄+銅の2ライン並列 | 14 |
| `l_shape` | L字コンベアテスト用 | 5 |
| `t_junction` | T字分岐テスト用 | 7 |

## パラメータ

| パラメータ | 説明 | デフォルト |
|-----------|------|-----------|
| `x`, `y`, `z` | 配置開始座標 | プレイヤー前方5ブロック |

## 出力

```
[SPAWN] Created iron_basic line at (10, 5, 10)
  - Miner at (10, 5, 10)
  - Conveyor at (11, 5, 10)
  - Conveyor at (12, 5, 10)
  - Conveyor at (13, 5, 10)
  - Conveyor at (14, 5, 10)
  - Conveyor at (15, 5, 10)
  - Furnace at (16, 5, 10)
```

## 実装状況

**現在**: 未実装（タスク #15）

実装後の期待動作:
1. プリセット定義をロード
2. 指定座標から順に機械を配置
3. コンベアの向きを自動設定
4. 結果をログ出力

## 関連ファイル

- `src/systems/command_ui.rs` - コマンド処理
- `src/systems/block_operations.rs` - ブロック配置
- `.claude/refactoring-tasks.md` - タスク #15

## 備考

- 配置先に既存ブロックがある場合は警告して中断
- クリエイティブモードでのみ使用可能（サバイバルでは無効）
- `/test` コマンドと組み合わせて使用
