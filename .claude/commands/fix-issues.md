# /fix-issues - Issue自動解決

issues.mdの未解決タスクを自動的に解決していくスキル。

## 使用方法

- `/fix-issues` - 次の未着手タスクを1つ解決
- `/fix-issues all` - 今週の未着手タスクを全て解決
- `/fix-issues #8` - 特定のタスク番号を解決
- `/fix-issues critical` - critical/highのみ解決
- `/fix-issues mid` - midのみ解決
- `/fix-issues status` - 現在の進捗を表示
- `/fix-issues add "タスク内容"` - 新規タスクを追加

## 引数: $ARGUMENTS

## 実行手順

### 1. issues.md読み込み

```
.specify/memory/issues.md を読み込み、以下を抽出:
- 未着手タスク（状態が「未着手」のもの）
- 優先度（critical/high → mid → low）
- 関連情報（関連項目、指摘元）
```

### 2. タスク選択

引数に応じてタスクを選択:
- 引数なし → 最も優先度の高い未着手タスク1つ
- `all` → 今週カテゴリの未着手タスク全て
- `#N` → 指定番号のタスク
- `critical`/`mid`/`low` → 該当優先度のタスク

### 3. タスク実行

タスクの種類に応じて実行:

#### ドキュメント系
```
- GETTING_STARTED.md作成 → 既存のREADME, CLAUDE.mdを参考に作成
- README修正 → 指摘内容に基づいて修正
```

#### コード品質系
```
- Clippy警告修正 → cargo clippy --fix
- unwrap修正 → .ok(), .unwrap_or_default(), Result返却に変更
- clone最適化 → 参照、take、Arc使用に変更
- debug!変更 → info! → debug! に置換
```

#### テスト系
```
- E2Eテスト → /e2e-test で動作確認
- モックアセット → 最小限のダミーアセット作成
- CI設定 → .github/workflows/ 作成
```

#### 設計系
```
- ファイル分割 → モジュール分割、re-export設定
- spec-impl-gap → 仕様と実装の差分を解消
```

### 4. 完了処理

1. **issues.md更新**
   - タスクの状態を「✅完了」に変更
   - Doneセクションに追記

2. **テスト実行**
   ```bash
   cargo test --lib
   cargo clippy
   ```

3. **changelog記録**
   - `.specify/memory/changelog.md` に追記

### 5. 次のタスクへ

`all`オプションの場合、次の未着手タスクを続行。

## タスク種別と対応方法

| 種別 | 例 | 対応方法 |
|------|-----|---------|
| ドキュメント | GETTING_STARTED.md | テンプレートから生成 |
| Clippy | 警告修正 | `cargo clippy --fix` |
| unwrap | 高リスク箇所 | 個別に適切なエラー処理に変更 |
| clone | ホットパス最適化 | profiling結果に基づいて対応 |
| テスト | E2Eスキップ解消 | モックアセット作成、テスト修正 |
| CI | GitHub Actions | ワークフローファイル作成 |
| 分割 | 大きなファイル分割 | モジュール分割 |

## 優先度判定

```
critical/high (今週)
  ├─ ビルドが通らない
  ├─ テストが失敗する
  ├─ Clippy警告
  └─ 高リスクunwrap

mid (2週間以内)
  ├─ テストカバレッジ改善
  ├─ パフォーマンス問題
  ├─ ドキュメント不足
  └─ CI/CD未整備

low (1ヶ月以内)
  ├─ リファクタリング
  ├─ 設計改善
  └─ 圧縮形式変更
```

## 自動化できないタスク

以下は手動対応が必要:
- **設計判断が必要**: アーキテクチャ変更
- **外部依存**: Steam連携、外部API
- **クリエイティブ**: アセット制作、UI設計
- **人間確認必須**: セキュリティ、破壊的変更

→ これらは `issues.md` に「要手動」タグを付けてスキップ

## 実行例

### 例1: 次のタスクを解決
```
> /fix-issues

issues.mdを確認...
未着手タスク: 8件
次のタスク: #8 docs/GETTING_STARTED.md作成 (critical)

GETTING_STARTED.mdを作成中...
- 概要
- クイックスタート
- 基本操作
- よくある質問

作成完了。テスト実行中...
cargo test: 177 passed
cargo clippy: 0 warnings

issues.md更新:
- #8: 未着手 → ✅完了
- Doneセクションに追記
```

### 例2: 全ての今週タスクを解決
```
> /fix-issues all

issues.mdを確認...
今週の未着手タスク: 1件
- #8 docs/GETTING_STARTED.md作成

順次実行中...
[1/1] #8 GETTING_STARTED.md作成 → ✅完了

全タスク完了！
```

### 例3: 進捗確認
```
> /fix-issues status

## 進捗状況

### 今週 (critical/high)
- ✅完了: 7/8
- 未着手: 1/8 (#8)

### 2週間以内 (mid)
- ✅完了: 1/6
- 未着手: 5/6

### 1ヶ月以内 (low)
- 未着手: 3/3

### 計測値
- Clippy: 0件 ✅
- テスト: 177 pass ✅
- unwrap: 48箇所 (高リスク0)
```

## issues.md形式

### タスク形式
```markdown
| # | タスク | 関連 | 状態 |
|---|--------|------|------|
| 8 | docs/GETTING_STARTED.md作成 | A1 | 未着手 |
```

### 状態
- `未着手` - まだ着手していない
- `作業中` - 現在作業中
- `✅完了` - 完了済み
- `要手動` - 自動化不可、手動対応必要
- `保留` - 一時的に保留

## 注意事項

- **テスト必須**: 変更後は必ず `cargo test` を実行
- **コミット**: 各タスク完了後に自動コミットしない（ユーザー確認待ち）
- **ロールバック**: 失敗時は変更を元に戻す
- **依存関係**: タスク間の依存を考慮して順序を決定
- **changelog**: 完了タスクは必ずchangelogに記録
