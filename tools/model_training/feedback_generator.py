"""
Feedback Generator
評価結果から改善プロンプトを生成

スコアが低い基準について具体的な修正指示を生成する。
"""

from typing import Dict, Any, List, Optional

# 基準ごとのフィードバックテンプレート
FEEDBACK_TEMPLATES = {
    "primitives": {
        "issue": "禁止されたプリミティブが検出されました: {violations}",
        "fix": """
許可されたプリミティブに置き換えてください:
- cylinder → create_octagonal_prism()
- circle → create_octagon()
- sphere → create_chamfered_cube() (小さいchamfer)

許可リスト: octagon, octagonal_prism, chamfered_cube, hexagon, trapezoid
""",
    },

    "materials": {
        "issue": "プリセット外のマテリアルが使用されています",
        "fix": """
MATERIALSプリセットのみを使用してください:

| 名前 | RGB | Metallic | Roughness |
|------|-----|----------|-----------|
| iron | (0.29, 0.29, 0.29) | 1.0 | 0.5 |
| copper | (0.72, 0.45, 0.20) | 1.0 | 0.4 |
| brass | (0.79, 0.64, 0.15) | 1.0 | 0.4 |
| dark_steel | (0.18, 0.18, 0.18) | 1.0 | 0.6 |
| wood | (0.55, 0.41, 0.08) | 0.0 | 0.8 |
| stone | (0.41, 0.41, 0.41) | 0.0 | 0.7 |

使用方法: apply_preset_material(obj, 'iron')
""",
    },

    "ratios": {
        "issue": "比率が規定範囲外です",
        "fix": """
以下の比率を調整してください:

ツール用:
- ハンドル長: 全長の 60-70%
- ヘッド幅: ハンドル直径の 4-6倍
- グリップ半径: ハンドル半径の 110-120%

機械用:
- ボディ充填率: ブロックサイズの 70-95%
- ディテール密度: 表面の 10-30%

具体的な違反:
{ratio_details}
""",
    },

    "triangle_budget": {
        "issue": "ポリゴン数が予算外です (現在: {actual})",
        "fix_over": """
ポリゴン数を削減してください:
1. 小さな装飾パーツを統合
2. グリップリングを減らす (5→3)
3. ボルト数を減らす
4. 複雑な形状をchamfered_cubeで代替

現在: {actual} → 目標: {max}以下
""",
        "fix_under": """
モデルが単純すぎます。以下を追加:
1. グリップリングをハンドルに追加
2. 角にボルト装飾を追加
3. エッジトリムやフレームを追加

現在: {actual} → 最低: {min}以上
""",
    },

    "connectivity": {
        "issue": "接続されていないパーツがあります: {floating_parts}",
        "fix": """
全パーツが物理的に接続されていることを確認:
- パーツ間のオーバーラップ: 0.003-0.005単位
- ハンドルとヘッドの間にcollar/jointを追加
- Boolean演算前に位置を確認

浮遊パーツ: {floating_parts}
""",
    },

    "edge_darkening": {
        "issue": "エッジ暗化が適用されていません",
        "fix": """
全パーツ結合後に以下を追加:

```python
apply_edge_darkening(result, factor=0.85)
```

これにより頂点カラーでエッジが暗くなり、立体感が出ます。
""",
    },

    "origin": {
        "issue": "原点位置が正しくありません",
        "fix_tool": """
ツール/アイテムの原点はバウンディングボックスの中央:

```python
set_origin_center(obj)
# または
finalize_model(obj, "tool")
```
""",
        "fix_machine": """
機械の原点は底面中央 (0, 0, 0):

```python
set_origin_bottom_center(obj)
# または
finalize_model(obj, "machine")
```
""",
    },
}


def generate_improvement_prompt(
    evaluation: Dict[str, Any],
    challenge: Dict[str, Any],
    iteration: int = 1
) -> str:
    """
    評価結果から改善プロンプトを生成。

    Args:
        evaluation: evaluate_model()の戻り値
        challenge: 課題定義
        iteration: 現在のイテレーション番号

    Returns:
        改善指示を含むマークダウン文字列
    """
    total_score = evaluation.get("total_score", 0)
    threshold = evaluation.get("threshold", 7.5)
    scores = evaluation.get("scores", {})
    details = evaluation.get("details", {})

    parts = [
        f"# 改善が必要です: {challenge.get('name', 'Unknown')}",
        "",
        f"**イテレーション {iteration}** | スコア: **{total_score:.1f}/10** (閾値: {threshold})",
        "",
        "---",
        "",
        "## 問題点 (優先度順)",
        "",
    ]

    # 重み付きでソート（影響度が高い順）
    weights = {
        "ratios": 0.25,
        "primitives": 0.20,
        "materials": 0.15,
        "triangle_budget": 0.15,
        "connectivity": 0.10,
        "origin": 0.10,
        "edge_darkening": 0.05,
    }

    issues = []
    for criterion, score in scores.items():
        if score < 8:  # 改善の余地あり
            impact = weights.get(criterion, 0.1) * (10 - score)
            issues.append((criterion, score, impact, details.get(criterion, {})))

    issues.sort(key=lambda x: -x[2])  # 影響度降順

    for criterion, score, impact, detail in issues:
        template = FEEDBACK_TEMPLATES.get(criterion, {})
        if not template:
            continue

        parts.append(f"### {criterion.upper()} (スコア: {score:.1f}/10)")
        parts.append("")

        # Issue
        issue_text = template.get("issue", "問題が検出されました")
        try:
            issue_text = issue_text.format(**_flatten_detail(detail))
        except (KeyError, ValueError):
            pass
        parts.append(f"**問題**: {issue_text}")
        parts.append("")

        # Fix
        fix_key = "fix"
        if criterion == "triangle_budget":
            if detail.get("status") in ["over_10", "over_20", "over_50"]:
                fix_key = "fix_over"
            elif detail.get("status") == "under_min":
                fix_key = "fix_under"
        elif criterion == "origin":
            category = challenge.get("category", "tool")
            fix_key = f"fix_{category}"

        fix_text = template.get(fix_key, template.get("fix", ""))
        try:
            fix_text = fix_text.format(**_flatten_detail(detail))
        except (KeyError, ValueError):
            pass

        parts.append("**修正方法**:")
        parts.append(fix_text)
        parts.append("")

    # 参照コード
    parts.append("---")
    parts.append("")
    parts.append("## 参照コードパターン")
    parts.append("")
    parts.append("```python")
    parts.append("# tools/blender_scripts/_base.py から")
    parts.append("")
    parts.append("# プリミティブ生成")
    parts.append("handle = create_octagonal_prism(radius=0.012, height=0.15, location=(0,0,0), name='Handle')")
    parts.append("head = create_chamfered_cube(size=(0.05, 0.03, 0.04), chamfer=0.003, location=(0,0,0.1), name='Head')")
    parts.append("")
    parts.append("# マテリアル適用")
    parts.append("apply_preset_material(handle, 'wood')")
    parts.append("apply_preset_material(head, 'iron')")
    parts.append("")
    parts.append("# 結合とファイナライズ")
    parts.append("result = join_all_meshes([handle, head, ...], 'ModelName')")
    parts.append("apply_edge_darkening(result, 0.85)")
    parts.append("finalize_model(result, 'tool')  # or 'machine'")
    parts.append("```")
    parts.append("")

    # 課題固有のヒント
    if challenge.get("hints"):
        parts.append("## 課題ヒント")
        parts.append("")
        for hint in challenge["hints"]:
            parts.append(f"- {hint}")
        parts.append("")

    # 参照モデル
    if challenge.get("reference_models"):
        parts.append("## 参照モデル")
        parts.append("")
        for ref in challenge["reference_models"]:
            parts.append(f"- `tools/blender_scripts/{ref}.py`")
        parts.append("")

    return "\n".join(parts)


def _flatten_detail(detail: Dict) -> Dict:
    """ネストした詳細情報をフラット化"""
    flat = {}
    for k, v in detail.items():
        if isinstance(v, dict):
            for k2, v2 in v.items():
                flat[f"{k}_{k2}"] = v2
        elif isinstance(v, list):
            flat[k] = ", ".join(str(x) for x in v) if v else "なし"
        else:
            flat[k] = v
    return flat


def generate_success_summary(
    evaluation: Dict[str, Any],
    challenge: Dict[str, Any],
    iterations: int
) -> str:
    """
    成功時のサマリーを生成。

    Args:
        evaluation: 最終評価結果
        challenge: 課題定義
        iterations: 所要イテレーション数

    Returns:
        サマリー文字列
    """
    total_score = evaluation.get("total_score", 0)
    scores = evaluation.get("scores", {})

    parts = [
        f"# 成功: {challenge.get('name', 'Unknown')}",
        "",
        f"**最終スコア**: {total_score:.1f}/10",
        f"**イテレーション数**: {iterations}",
        "",
        "## スコア詳細",
        "",
        "| 基準 | スコア |",
        "|------|--------|",
    ]

    for criterion, score in sorted(scores.items()):
        emoji = "✅" if score >= 8 else "⚠️" if score >= 5 else "❌"
        parts.append(f"| {criterion} | {emoji} {score:.1f} |")

    parts.append("")
    parts.append("---")
    parts.append("")
    parts.append(f"エクスポート先: `assets/models/{challenge.get('category', 'item')}s/{challenge.get('id', 'unknown').split('_')[-1]}.gltf`")

    return "\n".join(parts)


def generate_failure_summary(
    best_evaluation: Dict[str, Any],
    challenge: Dict[str, Any],
    iterations: int
) -> str:
    """
    失敗時のサマリーを生成。

    Args:
        best_evaluation: 最高スコアの評価結果
        challenge: 課題定義
        iterations: 実行イテレーション数

    Returns:
        サマリー文字列
    """
    total_score = best_evaluation.get("total_score", 0)
    threshold = best_evaluation.get("threshold", 7.5)

    parts = [
        f"# 未達成: {challenge.get('name', 'Unknown')}",
        "",
        f"**最高スコア**: {total_score:.1f}/10 (閾値: {threshold})",
        f"**イテレーション数**: {iterations}",
        "",
        "## 改善が必要な領域",
        "",
    ]

    scores = best_evaluation.get("scores", {})
    for criterion, score in sorted(scores.items(), key=lambda x: x[1]):
        if score < 7:
            parts.append(f"- **{criterion}**: {score:.1f}/10")

    parts.append("")
    parts.append("手動での調整を検討してください。")

    return "\n".join(parts)
