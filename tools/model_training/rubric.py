"""
Style Compliance Scoring Rubric (0-10 scale)
モデルのスタイル準拠度を評価するルーブリック

Total Score = weighted average of all criteria
"""

from typing import Dict, Any

# 評価基準の重み（合計1.0）
WEIGHTS = {
    "primitives": 0.20,      # 許可プリミティブのみ使用
    "materials": 0.15,       # MATERIALSプリセットのみ
    "ratios": 0.25,          # 比率準拠（最重要）
    "triangle_budget": 0.15, # ポリゴン予算内
    "connectivity": 0.10,    # パーツ接続
    "origin": 0.10,          # 原点位置
    "edge_darkening": 0.05,  # エッジ暗化
}

# 許可されたプリミティブ
ALLOWED_PRIMITIVES = [
    "octagon",
    "octagonal_prism",
    "chamfered_cube",
    "hexagon",
    "trapezoid",
]

# 禁止されたプリミティブ（検出対象）
FORBIDDEN_PRIMITIVES = [
    "cylinder",
    "circle",
    "sphere",
    "cone",
    "uv_sphere",
    "ico_sphere",
]

# マテリアルプリセット
MATERIAL_PRESETS = {
    "iron": {"color": (0.29, 0.29, 0.29), "metallic": 1.0, "roughness": 0.5},
    "copper": {"color": (0.72, 0.45, 0.20), "metallic": 1.0, "roughness": 0.4},
    "brass": {"color": (0.79, 0.64, 0.15), "metallic": 1.0, "roughness": 0.4},
    "dark_steel": {"color": (0.18, 0.18, 0.18), "metallic": 1.0, "roughness": 0.6},
    "wood": {"color": (0.55, 0.41, 0.08), "metallic": 0.0, "roughness": 0.8},
    "stone": {"color": (0.41, 0.41, 0.41), "metallic": 0.0, "roughness": 0.7},
}

# アクセントカラー
ACCENT_PRESETS = {
    "danger": (0.8, 0.2, 0.2),
    "warning": (0.8, 0.67, 0.2),
    "power": (0.2, 0.4, 0.8),
    "active": (0.2, 0.8, 0.4),
}

# カテゴリ別ポリゴン予算
TRIANGLE_BUDGETS = {
    "tool": {"min": 50, "recommended": 200, "max": 500},
    "item": {"min": 50, "recommended": 200, "max": 500},
    "machine": {"min": 200, "recommended": 800, "max": 1500},
    "structure": {"min": 300, "recommended": 2000, "max": 4000},
}

# ツール比率ルール
TOOL_RATIOS = {
    "handle_ratio": {"min": 0.60, "max": 0.70, "description": "ハンドル長/全長"},
    "head_width_ratio": {"min": 4.0, "max": 6.0, "description": "ヘッド幅/ハンドル直径"},
    "grip_ratio": {"min": 1.10, "max": 1.20, "description": "グリップ半径/ハンドル半径"},
}

# 機械比率ルール
MACHINE_RATIOS = {
    "body_fill": {"min": 0.70, "max": 0.95, "description": "ボディ/ブロックサイズ"},
    "detail_density": {"min": 0.10, "max": 0.30, "description": "ディテール面積比"},
}

RUBRIC = {
    "weights": WEIGHTS,
    "allowed_primitives": ALLOWED_PRIMITIVES,
    "forbidden_primitives": FORBIDDEN_PRIMITIVES,
    "material_presets": MATERIAL_PRESETS,
    "accent_presets": ACCENT_PRESETS,
    "triangle_budgets": TRIANGLE_BUDGETS,
    "tool_ratios": TOOL_RATIOS,
    "machine_ratios": MACHINE_RATIOS,

    "criteria": {
        "primitives": {
            "description": "許可されたプリミティブ形状のみ使用",
            "scoring": {
                10: "全て許可リストから",
                7: "1つの軽微な違反",
                4: "2-3の違反",
                0: "禁止プリミティブ使用",
            },
        },
        "materials": {
            "description": "MATERIALSプリセットのみ使用",
            "scoring": {
                10: "全てプリセットから",
                7: "カスタムカラーがプリセットの10%以内",
                4: "工業スタイルに合うカスタムカラー",
                0: "明るすぎる/非現実的な色",
            },
        },
        "ratios": {
            "description": "スタイルガイドの比率に準拠",
            "scoring": {
                10: "全比率が範囲内",
                8: "1つの比率が5%外れ",
                6: "1-2の比率が10%外れ",
                4: "複数の比率が大きく外れ",
                0: "完全に間違った比率",
            },
        },
        "triangle_budget": {
            "description": "カテゴリのポリゴン予算内",
            "scoring": {
                10: "推奨範囲内",
                8: "最大値以下、推奨超え",
                5: "最大値の10-20%超過",
                2: "最大値の20-50%超過",
                0: "50%超過または最小未満",
            },
        },
        "connectivity": {
            "description": "全パーツが物理的に接続",
            "scoring": {
                10: "全パーツがオーバーラップ接続",
                7: "全パーツが接触（隙間なし）",
                4: "1-2の小さな隙間（<0.005）",
                0: "浮遊パーツあり",
            },
        },
        "origin": {
            "description": "カテゴリに応じた正しい原点位置",
            "scoring": {
                10: "原点が正確",
                7: "許容範囲内",
                0: "原点が間違った位置",
            },
        },
        "edge_darkening": {
            "description": "頂点カラーによるエッジ暗化適用",
            "scoring": {
                10: "0.85係数で暗化適用",
                5: "部分的/不正確な暗化",
                0: "頂点カラーなし",
            },
        },
    },
}


def calculate_score(evaluation_results: Dict[str, float]) -> float:
    """
    個別基準のスコアから重み付き合計スコアを計算。

    Args:
        evaluation_results: 基準名 → スコア(0-10) のマッピング

    Returns:
        重み付き合計スコア (0-10)
    """
    total = 0.0
    total_weight = 0.0

    for criterion, weight in WEIGHTS.items():
        if criterion in evaluation_results:
            score = evaluation_results[criterion]
            total += score * weight
            total_weight += weight

    # 評価されていない基準がある場合は正規化
    if total_weight > 0 and total_weight < 1.0:
        total = total / total_weight

    return round(total, 2)


def get_ratio_rules(category: str) -> Dict[str, Any]:
    """カテゴリに応じた比率ルールを取得"""
    if category in ["tool", "item"]:
        return TOOL_RATIOS
    elif category == "machine":
        return MACHINE_RATIOS
    else:
        return {}


def get_triangle_budget(category: str) -> Dict[str, int]:
    """カテゴリに応じたポリゴン予算を取得"""
    return TRIANGLE_BUDGETS.get(category, TRIANGLE_BUDGETS["machine"])


def color_distance(c1: tuple, c2: tuple) -> float:
    """2つの色のユークリッド距離を計算"""
    return sum((a - b) ** 2 for a, b in zip(c1[:3], c2[:3])) ** 0.5


def find_closest_preset(color: tuple) -> tuple:
    """最も近いプリセットカラーを見つける"""
    min_dist = float('inf')
    closest = None
    closest_name = None

    for name, preset in MATERIAL_PRESETS.items():
        dist = color_distance(color, preset["color"])
        if dist < min_dist:
            min_dist = dist
            closest = preset["color"]
            closest_name = name

    return closest_name, closest, min_dist
