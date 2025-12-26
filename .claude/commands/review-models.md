# モデルレビュー

評価待ちの3Dモデルをまとめてレビューする。

## 引数
$ARGUMENTS

## 実行手順

### 1. 評価待ちモデルを確認

```python
import sys
sys.path.insert(0, '/home/bacon/github/idle_factory')
from tools.model_training.human_feedback import get_feedback_db

db = get_feedback_db()
print(db.batch_review_report())
```

### 2. 各モデルのスクリーンショットを表示

評価待ちモデルごとに：
1. スクリーンショットを `Read` ツールで表示
2. 人間に評価を依頼（5段階 x 5項目）
3. フィードバックを記録

### 3. 評価項目（1-5）

| 項目 | 説明 |
|------|------|
| shape | 形状：そのモデルらしさ、シルエット |
| style | スタイル：ローポリ感、ゲームに合うか |
| detail | ディテール：パーツのバランス |
| color | 色/マテリアル：色味、金属感 |
| overall | 総合：ゲームで使いたいか |

### 4. 評価記録

```python
db.add_feedback("gen_id_here", {
    "shape": 4,
    "style": 4,
    "detail": 4,
    "color": 4,
    "overall": 4,
    "comments": "コメント",
    "issues": [],  # 問題があれば
    "fixes_applied": [],  # 修正があれば
})
```

### 5. 完了後

```python
print(db.generate_report())
```

## 簡易評価モード

「全部OK」「まとめて評価」の場合：

```python
# 全て4点で一括評価
pending = db.get_pending_reviews()
for item in pending:
    db.add_feedback(item["gen_id"], {
        "shape": 4, "style": 4, "detail": 4, "color": 4, "overall": 4,
        "comments": "バッチレビュー",
    })
```
