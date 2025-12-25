# /evaluate - AI自動評価・改善フィードバックループ

AIがゲームをプレイして評価・改善提案を生成するスキル。

## 使用方法

- `/evaluate` - Casualペルソナで評価
- `/evaluate casual` - Casualペルソナで評価
- `/evaluate critic` - 批判的視点で評価
- `/evaluate optimizer` - 効率厨視点で評価
- `/evaluate newbie` - 完全初心者視点で評価
- `/evaluate gamer` - ジャンル経験者視点で評価
- `/evaluate speedrunner` - 最速クリア視点で評価
- `/evaluate builder` - 建築・見た目重視で評価
- `/evaluate explorer` - 全要素探索視点で評価
- `/evaluate all` - 全ペルソナで順次評価
- `/evaluate fix` - pending/の改善提案を実装
- `/evaluate report` - 最新の評価レポートを表示
- `/evaluate trends` - 傾向分析を表示
- `/evaluate meta` - メタ評価レポートを表示
- `/evaluate calibrate` - 基準時間を再調整

## コアコンセプト（これに反する提案は自動却下）

constitution.mdより:
- **creative-sandbox**: 戦闘なし、敵なし、空腹なし、落下ダメージなし
- **stress-free**: プレイヤーを急かさない、ペナルティを与えない
- **data-driven**: YAML+Luaによる拡張性
- **player-empower**: プレイヤーに力を与える（制限しない）

## 引数: $ARGUMENTS

## 実行手順

### 引数が空または特定のペルソナ名の場合

1. **ペルソナ選択**
   - 引数が空なら `casual` を使用
   - 引数がペルソナ名なら該当ペルソナを使用
   - 有効なペルソナ: newbie, casual, gamer, optimizer, critic, speedrunner, builder, explorer

2. **ゲーム起動とプレイ**
   ```bash
   # E2Eテストを実行
   DISPLAY=:10 timeout 120 cargo run -- --e2e-test
   ```

3. **結果収集**
   - `screenshots/test_report.txt` を読む
   - `screenshots/ui_dump_*.txt` を読む
   - 問題があれば該当スクリーンショットを確認

4. **ペルソナ視点で評価**
   以下の評価プロンプトを使用:
   ```
   あなたは{persona}タイプのゲーマーです。以下のプレイログを分析し、
   そのペルソナ視点で評価してください。

   ## コアコンセプト（これに反する提案は却下）
   - creative-sandbox: 戦闘なし、敵なし
   - stress-free: プレイヤーを急かさない
   - player-empower: 制限より自由

   ## 評価項目
   1. 目標達成容易性: 目標に到達しやすいか？
   2. 直感性: 操作は直感的か？
   3. フィードバック: 操作結果は明確か？
   4. 学習曲線: 自然に学べるか？
   5. 満足感: 達成感はあるか？
   ```

5. **レポート生成**
   `feedback/reports/YYYY-MM-DD_{persona}_NNN.md` に保存

6. **自動実装判定**
   以下は自動実装可能:
   - UIテキスト・ヒントの追加/変更
   - 既存パターンに沿った修正
   - テストが通る変更
   - コアコンセプトに違反しない

7. **自動実装実行（該当する場合）**
   - コード変更を生成
   - `cargo test` 実行
   - 成功 → 自動コミット
   - 失敗 → `feedback/pending/` に記録

### `/evaluate all` の場合

全ペルソナで順次評価を実行:
1. casual → 基本UX確認
2. critic → 細かい問題発見
3. optimizer → 効率性問題発見
4. その他のペルソナ

### `/evaluate fix` の場合

1. `feedback/pending/` の改善提案を読む
2. 優先度順に実装を試みる
3. テスト実行
4. 成功したら `feedback/auto-implemented/` に移動

### `/evaluate report` の場合

1. `feedback/reports/` の最新レポートを読む
2. サマリを表示

### `/evaluate trends` の場合

1. `feedback/trends.md` を読む（なければ生成）
2. 傾向分析を表示

### `/evaluate meta` の場合

1. `feedback/meta/` の最新メタ評価を読む
2. システム性能を表示

## 出力ファイル

```
feedback/
├── sessions/           # セッションデータ（JSON）
├── reports/            # 評価レポート（Markdown）
├── auto-implemented/   # 自動実装履歴
├── pending/            # 要検討の改善提案
├── config/             # 評価設定
├── meta/               # メタ評価レポート
├── trends.md           # 傾向分析
└── summary.md          # 累積サマリ
```

## 成功指標

| 指標 | 目標値 |
|------|--------|
| 評価サイクル時間 | 10分以内 |
| 目標達成率 | 80%以上 |
| 自動実装成功率 | 70%以上 |
| 回帰検出率 | 100% |
| コアコンセプト違反 | 0件 |

## 注意事項

- 評価はトークン効率を意識して実行
- スクリーンショットは問題時のみ確認
- 累積学習メモリを活用（--freshで初見評価）
- 改善提案はコアコンセプトを必ずチェック
