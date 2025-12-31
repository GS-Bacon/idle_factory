#!/usr/bin/env python3
"""
Voxel Model Evaluator - 自動品質評価と改善提案

生成されたボクセルモデルを評価し、改善点を提案する。
"""

import json
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass, field, asdict
from datetime import datetime


@dataclass
class EvaluationResult:
    """評価結果"""
    model_name: str
    category: str
    scores: Dict[str, float] = field(default_factory=dict)
    total_score: float = 0.0
    issues: List[str] = field(default_factory=list)
    suggestions: List[str] = field(default_factory=list)
    passed: bool = False
    timestamp: str = ""

    def __post_init__(self):
        if not self.timestamp:
            self.timestamp = datetime.now().isoformat()


# 評価基準
CRITERIA = {
    "symmetry": {
        "weight": 0.15,
        "description": "左右対称性（機械は対称が望ましい）"
    },
    "fill_ratio": {
        "weight": 0.20,
        "description": "充填率（20-60%が理想）"
    },
    "color_variety": {
        "weight": 0.15,
        "description": "色の多様性（2-5色が理想）"
    },
    "feature_presence": {
        "weight": 0.25,
        "description": "特徴的要素の有無"
    },
    "proportions": {
        "weight": 0.25,
        "description": "比率の適切さ"
    }
}

# カテゴリ別の期待値
CATEGORY_EXPECTATIONS = {
    "item": {
        "size": (8, 8, 16),
        "fill_ratio_range": (0.15, 0.40),
        "color_count_range": (2, 4),
        "features": ["handle", "head"],
    },
    "machine": {
        "size": (16, 16, 16),
        "fill_ratio_range": (0.30, 0.70),
        "color_count_range": (3, 6),
        "features": ["body", "output", "indicator"],
    },
    "conveyor": {
        "size": (16, 16, 4),
        "fill_ratio_range": (0.40, 0.80),
        "color_count_range": (3, 5),
        "features": ["frame", "belt", "arrow"],
    },
}

# 成功閾値
PASS_THRESHOLD = 7.0


def evaluate_symmetry(voxels: Dict, size: Tuple[int, int, int]) -> Tuple[float, List[str]]:
    """左右対称性を評価"""
    sx, sy, sz = size
    symmetric_count = 0
    total_count = 0

    for (x, y, z), color in voxels.items():
        mirror_x = sx - 1 - x
        mirror_pos = (mirror_x, y, z)

        if mirror_pos in voxels:
            if voxels[mirror_pos] == color:
                symmetric_count += 1
        total_count += 1

    if total_count == 0:
        return 0.0, ["ボクセルがありません"]

    ratio = symmetric_count / total_count
    score = ratio * 10

    issues = []
    if ratio < 0.7:
        issues.append(f"対称性が低い ({ratio:.0%})")

    return score, issues


def evaluate_fill_ratio(voxels: Dict, size: Tuple[int, int, int],
                        expected_range: Tuple[float, float]) -> Tuple[float, List[str]]:
    """充填率を評価"""
    sx, sy, sz = size
    total_volume = sx * sy * sz
    fill_ratio = len(voxels) / total_volume

    min_ratio, max_ratio = expected_range
    issues = []

    if fill_ratio < min_ratio:
        score = (fill_ratio / min_ratio) * 7
        issues.append(f"充填率が低すぎる ({fill_ratio:.0%} < {min_ratio:.0%})")
    elif fill_ratio > max_ratio:
        score = max(0, 10 - (fill_ratio - max_ratio) * 20)
        issues.append(f"充填率が高すぎる ({fill_ratio:.0%} > {max_ratio:.0%})")
    else:
        # 理想範囲内
        mid = (min_ratio + max_ratio) / 2
        deviation = abs(fill_ratio - mid) / (max_ratio - min_ratio)
        score = 10 - deviation * 3

    return score, issues


def evaluate_color_variety(voxels: Dict, expected_range: Tuple[int, int]) -> Tuple[float, List[str]]:
    """色の多様性を評価"""
    colors = set(voxels.values())
    color_count = len(colors)
    min_colors, max_colors = expected_range
    issues = []

    if color_count < min_colors:
        score = (color_count / min_colors) * 6
        issues.append(f"色が少なすぎる ({color_count} < {min_colors})")
    elif color_count > max_colors:
        score = max(5, 10 - (color_count - max_colors) * 2)
        issues.append(f"色が多すぎる ({color_count} > {max_colors})")
    else:
        score = 10

    return score, issues


def evaluate_feature_presence(voxels: Dict, size: Tuple[int, int, int],
                              category: str) -> Tuple[float, List[str]]:
    """特徴的要素の有無を評価"""
    sx, sy, sz = size
    issues = []
    found_features = []

    # Z方向の分布を確認
    z_layers = {}
    for (x, y, z), color in voxels.items():
        if z not in z_layers:
            z_layers[z] = set()
        z_layers[z].add(color)

    # 底面があるか
    if 0 in z_layers:
        found_features.append("base")

    # 上部構造があるか
    if sz - 1 in z_layers or sz - 2 in z_layers:
        found_features.append("top")

    # 色の分布で特徴を推定
    colors = set(voxels.values())
    if len(colors) >= 2:
        found_features.append("multi_material")

    # アクセント色があるか（警告色、アクティブ色など）
    # パレットインデックス 10-13 がアクセント色
    accent_colors = {10, 11, 12, 13, 14, 15, 16, 17}
    if any(c in accent_colors for c in colors):
        found_features.append("accent")

    # カテゴリ別の必須要素チェック
    expectations = CATEGORY_EXPECTATIONS.get(category, {})
    expected_features = expectations.get("features", [])

    feature_score = len(found_features) / max(len(expected_features), 1) * 10

    if not found_features:
        issues.append("特徴的な要素がない")
    elif len(found_features) < len(expected_features):
        missing = set(expected_features) - set(found_features)
        issues.append(f"特徴が不足: {', '.join(missing)}")

    return min(10, feature_score), issues


def evaluate_proportions(voxels: Dict, size: Tuple[int, int, int],
                         category: str) -> Tuple[float, List[str]]:
    """比率の適切さを評価"""
    sx, sy, sz = size
    issues = []

    # バウンディングボックスを計算
    if not voxels:
        return 0.0, ["ボクセルがありません"]

    xs = [x for x, y, z in voxels.keys()]
    ys = [y for x, y, z in voxels.keys()]
    zs = [z for x, y, z in voxels.keys()]

    actual_width = max(xs) - min(xs) + 1
    actual_depth = max(ys) - min(ys) + 1
    actual_height = max(zs) - min(zs) + 1

    # 使用率
    width_usage = actual_width / sx
    depth_usage = actual_depth / sy
    height_usage = actual_height / sz

    # カテゴリ別の期待比率
    if category == "item":
        # アイテムは縦長が理想
        if actual_height < actual_width:
            issues.append("アイテムは縦長が望ましい")
            score = 6
        else:
            score = 10
    elif category == "conveyor":
        # コンベアは平たいのが理想
        if actual_height > actual_width * 0.5:
            issues.append("コンベアは平たくすべき")
            score = 6
        else:
            score = 10
    else:
        # 機械は立方体に近いのが理想
        aspect_ratio = max(actual_width, actual_depth, actual_height) / \
                       max(1, min(actual_width, actual_depth, actual_height))
        if aspect_ratio > 2.5:
            issues.append(f"アスペクト比が大きすぎる ({aspect_ratio:.1f})")
            score = max(4, 10 - aspect_ratio)
        else:
            score = 10

    # 空間使用率
    avg_usage = (width_usage + depth_usage + height_usage) / 3
    if avg_usage < 0.5:
        issues.append(f"空間使用率が低い ({avg_usage:.0%})")
        score = min(score, 7)

    return score, issues


def evaluate_model(voxels: Dict[Tuple[int, int, int], int],
                   size: Tuple[int, int, int],
                   model_name: str,
                   category: str = "machine") -> EvaluationResult:
    """モデルを総合評価"""
    result = EvaluationResult(model_name=model_name, category=category)

    expectations = CATEGORY_EXPECTATIONS.get(category, CATEGORY_EXPECTATIONS["machine"])

    # 各基準で評価
    evaluations = [
        ("symmetry", evaluate_symmetry(voxels, size)),
        ("fill_ratio", evaluate_fill_ratio(
            voxels, size, expectations.get("fill_ratio_range", (0.3, 0.7)))),
        ("color_variety", evaluate_color_variety(
            voxels, expectations.get("color_count_range", (3, 6)))),
        ("feature_presence", evaluate_feature_presence(voxels, size, category)),
        ("proportions", evaluate_proportions(voxels, size, category)),
    ]

    total_weight = 0
    weighted_score = 0

    for criterion, (score, issues) in evaluations:
        weight = CRITERIA[criterion]["weight"]
        result.scores[criterion] = round(score, 1)
        weighted_score += score * weight
        total_weight += weight
        result.issues.extend(issues)

    result.total_score = round(weighted_score / total_weight, 1) if total_weight > 0 else 0
    result.passed = result.total_score >= PASS_THRESHOLD

    # 改善提案を生成
    result.suggestions = generate_suggestions(result)

    return result


def generate_suggestions(result: EvaluationResult) -> List[str]:
    """評価結果から改善提案を生成"""
    suggestions = []

    for criterion, score in result.scores.items():
        if score < 7:
            if criterion == "symmetry":
                suggestions.append("左右対称になるようボクセルを配置")
            elif criterion == "fill_ratio":
                if "低すぎる" in str(result.issues):
                    suggestions.append("ボクセルを追加して密度を上げる")
                else:
                    suggestions.append("不要なボクセルを削除して軽量化")
            elif criterion == "color_variety":
                if "少なすぎる" in str(result.issues):
                    suggestions.append("アクセント色（warning, active）を追加")
                else:
                    suggestions.append("色数を減らして統一感を出す")
            elif criterion == "feature_presence":
                suggestions.append("特徴的な要素（出力口、インジケータ等）を追加")
            elif criterion == "proportions":
                suggestions.append("バウンディングボックスを調整して比率を改善")

    return suggestions


def format_evaluation(result: EvaluationResult) -> str:
    """評価結果を整形して出力"""
    lines = [
        f"=== {result.model_name} ({result.category}) ===",
        f"Total Score: {result.total_score}/10 {'✓ PASS' if result.passed else '✗ FAIL'}",
        "",
        "Scores:",
    ]

    for criterion, score in result.scores.items():
        bar = "█" * int(score) + "░" * (10 - int(score))
        status = "✓" if score >= 7 else "✗"
        lines.append(f"  {criterion:20s} [{bar}] {score:4.1f} {status}")

    if result.issues:
        lines.append("")
        lines.append("Issues:")
        for issue in result.issues:
            lines.append(f"  - {issue}")

    if result.suggestions:
        lines.append("")
        lines.append("Suggestions:")
        for suggestion in result.suggestions:
            lines.append(f"  → {suggestion}")

    return "\n".join(lines)


# 学習データベース
LEARNING_DB_PATH = Path("tools/model_learning.json")


def load_learning_db() -> Dict:
    """学習データベースを読み込み"""
    if LEARNING_DB_PATH.exists():
        with open(LEARNING_DB_PATH) as f:
            return json.load(f)
    return {"models": [], "patterns": {}}


def save_learning_db(db: Dict):
    """学習データベースを保存"""
    LEARNING_DB_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(LEARNING_DB_PATH, "w") as f:
        json.dump(db, f, indent=2, ensure_ascii=False)


def record_evaluation(result: EvaluationResult):
    """評価結果を学習データベースに記録"""
    db = load_learning_db()

    record = asdict(result)
    db["models"].append(record)

    # パターンを学習
    category = result.category
    if category not in db["patterns"]:
        db["patterns"][category] = {
            "successful_scores": {},
            "common_issues": {},
        }

    if result.passed:
        for criterion, score in result.scores.items():
            if criterion not in db["patterns"][category]["successful_scores"]:
                db["patterns"][category]["successful_scores"][criterion] = []
            db["patterns"][category]["successful_scores"][criterion].append(score)

    for issue in result.issues:
        if issue not in db["patterns"][category]["common_issues"]:
            db["patterns"][category]["common_issues"][issue] = 0
        db["patterns"][category]["common_issues"][issue] += 1

    save_learning_db(db)


def get_guidance(category: str) -> Dict:
    """過去の学習から生成ガイダンスを取得"""
    db = load_learning_db()

    if category not in db.get("patterns", {}):
        return {"message": "No learning data available"}

    patterns = db["patterns"][category]

    # 平均成功スコア
    avg_scores = {}
    for criterion, scores in patterns.get("successful_scores", {}).items():
        if scores:
            avg_scores[criterion] = sum(scores) / len(scores)

    # よくある問題（上位3件）
    common_issues = sorted(
        patterns.get("common_issues", {}).items(),
        key=lambda x: x[1],
        reverse=True
    )[:3]

    return {
        "target_scores": avg_scores,
        "avoid_issues": [issue for issue, count in common_issues],
    }


if __name__ == "__main__":
    # テスト用
    import sys
    sys.path.insert(0, str(Path(__file__).parent))

    from voxel_generator import VoxelModel

    # テストモデル
    model = VoxelModel(16, 16, 16)
    model.fill_box(2, 2, 0, 13, 13, 10, "iron")
    model.fill_box(4, 4, 10, 11, 11, 14, "dark_steel")

    result = evaluate_model(model.voxels, (16, 16, 16), "test_machine", "machine")
    print(format_evaluation(result))
