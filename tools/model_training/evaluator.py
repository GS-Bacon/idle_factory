"""
Model Style Evaluator
モデルのスタイル準拠度を評価するエンジン

Blender内で実行される評価関数と、外部から呼び出す評価関数を提供。
"""

import json
from pathlib import Path
from typing import Dict, Any, Optional, List, Tuple

from .rubric import (
    RUBRIC, WEIGHTS, calculate_score,
    ALLOWED_PRIMITIVES, FORBIDDEN_PRIMITIVES,
    MATERIAL_PRESETS, ACCENT_PRESETS,
    TRIANGLE_BUDGETS, TOOL_RATIOS, MACHINE_RATIOS,
    color_distance, find_closest_preset, get_triangle_budget,
)


def evaluate_model(
    model_info: Dict[str, Any],
    category: str,
    challenge: Optional[Dict[str, Any]] = None
) -> Dict[str, Any]:
    """
    モデル情報からスタイル準拠度を評価（Blender外部から呼び出し用）。

    Args:
        model_info: Blenderから取得したモデル情報
            - triangles: 三角形数
            - materials: [{"name": str, "color": (r,g,b,a), "metallic": float, "roughness": float}, ...]
            - bounds: {"min": (x,y,z), "max": (x,y,z)}
            - origin: (x, y, z)
            - parts: [{"name": str, "triangles": int}, ...]
            - has_vertex_colors: bool
        category: "tool" | "machine" | "structure"
        challenge: 課題定義（オプション）

    Returns:
        {
            "total_score": float,
            "scores": {"criterion": float, ...},
            "details": {"criterion": {...}, ...},
            "passed": bool,
            "threshold": float
        }
    """
    results = {}

    # 1. マテリアル評価
    results["materials"] = _evaluate_materials(model_info.get("materials", []))

    # 2. ポリゴン予算評価
    results["triangle_budget"] = _evaluate_triangle_budget(
        model_info.get("triangles", 0),
        category
    )

    # 3. 原点位置評価
    results["origin"] = _evaluate_origin(
        model_info.get("origin", (0, 0, 0)),
        model_info.get("bounds", {}),
        category
    )

    # 4. エッジ暗化評価
    results["edge_darkening"] = _evaluate_edge_darkening(
        model_info.get("has_vertex_colors", False)
    )

    # 5. 比率評価（課題定義がある場合）
    if challenge and "ratios" in challenge:
        results["ratios"] = _evaluate_ratios(
            model_info.get("bounds", {}),
            model_info.get("parts", []),
            challenge["ratios"],
            category
        )
    else:
        # デフォルトの比率ルールを使用
        default_ratios = TOOL_RATIOS if category in ["tool", "item"] else MACHINE_RATIOS
        results["ratios"] = _evaluate_ratios(
            model_info.get("bounds", {}),
            model_info.get("parts", []),
            default_ratios,
            category
        )

    # 6. 接続性評価（パーツ情報がある場合）
    if model_info.get("parts"):
        results["connectivity"] = _evaluate_connectivity(model_info.get("parts", []))
    else:
        results["connectivity"] = {"score": 7.0, "note": "パーツ情報なし、デフォルトスコア"}

    # 7. プリミティブ評価（検出が難しいのでデフォルトスコア）
    results["primitives"] = {"score": 8.0, "note": "自動検出未実装、デフォルトスコア"}

    # スコア集計
    scores = {k: v.get("score", 0) for k, v in results.items()}
    total_score = calculate_score(scores)

    # 閾値判定
    threshold = challenge.get("success_threshold", 7.5) if challenge else 7.5

    return {
        "total_score": total_score,
        "scores": scores,
        "details": results,
        "passed": total_score >= threshold,
        "threshold": threshold,
    }


def _evaluate_materials(materials: List[Dict]) -> Dict[str, Any]:
    """マテリアル評価"""
    if not materials:
        return {"score": 0, "violations": ["マテリアルなし"]}

    violations = []
    preset_matches = 0

    for mat in materials:
        color = mat.get("color", (0.5, 0.5, 0.5, 1.0))[:3]
        name = mat.get("name", "unknown")

        # プリセットと一致するかチェック
        closest_name, closest_color, dist = find_closest_preset(color)

        if dist < 0.05:  # ほぼ一致
            preset_matches += 1
        elif dist < 0.15:  # 許容範囲
            violations.append(f"{name}: {closest_name}に近いが完全一致ではない (距離: {dist:.3f})")
        else:
            violations.append(f"{name}: プリセットから離れすぎ (最近: {closest_name}, 距離: {dist:.3f})")

    # スコア計算
    if not violations:
        score = 10.0
    elif len(violations) == 1:
        score = 7.0
    elif len(violations) <= 3:
        score = 4.0
    else:
        score = 2.0

    return {
        "score": score,
        "preset_matches": preset_matches,
        "total_materials": len(materials),
        "violations": violations,
    }


def _evaluate_triangle_budget(triangles: int, category: str) -> Dict[str, Any]:
    """ポリゴン予算評価"""
    budget = get_triangle_budget(category)
    min_tri = budget["min"]
    recommended = budget["recommended"]
    max_tri = budget["max"]

    if triangles < min_tri:
        score = 3.0
        status = "under_min"
    elif triangles <= recommended:
        score = 10.0
        status = "optimal"
    elif triangles <= max_tri:
        score = 8.0
        status = "acceptable"
    elif triangles <= max_tri * 1.2:
        score = 5.0
        status = "over_10"
    elif triangles <= max_tri * 1.5:
        score = 2.0
        status = "over_20"
    else:
        score = 0.0
        status = "over_50"

    return {
        "score": score,
        "actual": triangles,
        "min": min_tri,
        "recommended": recommended,
        "max": max_tri,
        "status": status,
    }


def _evaluate_origin(
    origin: Tuple[float, float, float],
    bounds: Dict[str, Tuple[float, float, float]],
    category: str
) -> Dict[str, Any]:
    """原点位置評価"""
    if not bounds:
        return {"score": 5.0, "note": "バウンド情報なし"}

    min_pt = bounds.get("min", (0, 0, 0))
    max_pt = bounds.get("max", (0, 0, 0))

    if category == "machine":
        # 機械: 底面中央 (0, 0, 0)
        expected = "bottom_center"
        # 原点がほぼ(0,0,0)で、min_zがほぼ0であるべき
        origin_deviation = sum(abs(o) for o in origin)
        bottom_deviation = abs(min_pt[2])  # Z軸の最小値が0に近いべき
        total_deviation = origin_deviation + bottom_deviation
    else:
        # ツール: バウンディングボックスの中心
        expected = "center"
        center = tuple((min_pt[i] + max_pt[i]) / 2 for i in range(3))
        total_deviation = sum(abs(origin[i] - center[i]) for i in range(3))

    tolerance = 0.02

    if total_deviation < tolerance:
        score = 10.0
    elif total_deviation < tolerance * 3:
        score = 7.0
    else:
        score = 3.0

    return {
        "score": score,
        "expected": expected,
        "actual": origin,
        "deviation": total_deviation,
        "tolerance": tolerance,
    }


def _evaluate_edge_darkening(has_vertex_colors: bool) -> Dict[str, Any]:
    """エッジ暗化評価"""
    if has_vertex_colors:
        return {"score": 10.0, "applied": True}
    else:
        return {"score": 0.0, "applied": False, "fix": "apply_edge_darkening(obj, 0.85)"}


def _evaluate_ratios(
    bounds: Dict[str, Tuple[float, float, float]],
    parts: List[Dict],
    expected_ratios: Dict[str, Any],
    category: str
) -> Dict[str, Any]:
    """比率評価"""
    if not bounds:
        return {"score": 5.0, "note": "バウンド情報なし"}

    min_pt = bounds.get("min", (0, 0, 0))
    max_pt = bounds.get("max", (0, 0, 0))

    # 全体サイズ
    total_height = max_pt[2] - min_pt[2]
    total_width = max_pt[0] - min_pt[0]
    total_depth = max_pt[1] - min_pt[1]

    violations = []
    checked_ratios = {}

    # パーツ名からハンドルとヘッドを探す
    handle_part = None
    head_part = None

    for part in parts:
        name = part.get("name", "").lower()
        if "handle" in name:
            handle_part = part
        elif "head" in name or "blade" in name:
            head_part = part

    # 比率チェック
    for ratio_name, ratio_def in expected_ratios.items():
        if isinstance(ratio_def, dict):
            min_val = ratio_def.get("min", 0)
            max_val = ratio_def.get("max", 1)
        else:
            continue

        actual = None

        # 比率の種類に応じて計算
        if ratio_name == "handle_ratio" and handle_part and total_height > 0:
            # ハンドル高さ/全高
            handle_bounds = handle_part.get("bounds", {})
            if handle_bounds:
                handle_height = handle_bounds.get("max", (0, 0, 0))[2] - handle_bounds.get("min", (0, 0, 0))[2]
                actual = handle_height / total_height
        elif ratio_name == "body_fill" and category == "machine":
            # ボディ充填率（機械用）
            block_size = 1.0  # 1ブロック = 1.0
            actual = max(total_width, total_depth, total_height) / block_size

        if actual is not None:
            checked_ratios[ratio_name] = {
                "actual": actual,
                "expected": f"[{min_val}, {max_val}]",
                "in_range": min_val <= actual <= max_val
            }

            if not (min_val <= actual <= max_val):
                deviation = min(abs(actual - min_val), abs(actual - max_val))
                violations.append({
                    "ratio": ratio_name,
                    "actual": round(actual, 3),
                    "expected": f"[{min_val}, {max_val}]",
                    "deviation": round(deviation, 3)
                })

    # スコア計算
    if not expected_ratios:
        score = 7.0  # 比率チェックなし
    elif not violations:
        score = 10.0
    elif len(violations) == 1:
        score = 7.0
    elif len(violations) <= 2:
        score = 5.0
    else:
        score = 3.0

    return {
        "score": score,
        "checked": checked_ratios,
        "violations": violations,
    }


def _evaluate_connectivity(parts: List[Dict]) -> Dict[str, Any]:
    """接続性評価（簡易版）"""
    if len(parts) <= 1:
        return {"score": 10.0, "note": "単一パーツ"}

    # パーツ間の距離をチェック（バウンド情報がある場合）
    floating_parts = []

    for i, part in enumerate(parts):
        bounds = part.get("bounds", {})
        if not bounds:
            continue

        is_connected = False
        for j, other in enumerate(parts):
            if i == j:
                continue
            other_bounds = other.get("bounds", {})
            if not other_bounds:
                continue

            # バウンディングボックスが重なるか隣接するかチェック
            if _bounds_overlap_or_touch(bounds, other_bounds):
                is_connected = True
                break

        if not is_connected:
            floating_parts.append(part.get("name", f"part_{i}"))

    if not floating_parts:
        score = 10.0
    elif len(floating_parts) == 1:
        score = 5.0
    else:
        score = 2.0

    return {
        "score": score,
        "floating_parts": floating_parts,
        "total_parts": len(parts),
    }


def _bounds_overlap_or_touch(b1: Dict, b2: Dict, tolerance: float = 0.01) -> bool:
    """2つのバウンディングボックスが重なるか接触するか判定"""
    min1 = b1.get("min", (0, 0, 0))
    max1 = b1.get("max", (0, 0, 0))
    min2 = b2.get("min", (0, 0, 0))
    max2 = b2.get("max", (0, 0, 0))

    for i in range(3):
        if max1[i] + tolerance < min2[i] or max2[i] + tolerance < min1[i]:
            return False

    return True


# =============================================================================
# Blender内で実行する評価関数（文字列として渡される）
# =============================================================================

BLENDER_EVALUATE_CODE = '''
def evaluate_style_in_blender(obj, category="tool"):
    """
    Blender内でモデルのスタイルを評価し、JSON形式で結果を返す。

    Usage in MCP:
        result_json = evaluate_style_in_blender(bpy.data.objects["MyModel"], "tool")
        print(result_json)
    """
    import bpy
    import json
    from mathutils import Vector

    # 基本情報収集
    info = {
        "name": obj.name,
        "triangles": 0,
        "materials": [],
        "bounds": {},
        "origin": list(obj.location),
        "parts": [],
        "has_vertex_colors": False,
    }

    if obj.type != 'MESH':
        return json.dumps({"error": "Not a mesh object"})

    mesh = obj.data

    # 三角形数
    info["triangles"] = sum(len(poly.vertices) - 2 for poly in mesh.polygons)

    # マテリアル
    for mat in mesh.materials:
        if mat and mat.use_nodes:
            bsdf = None
            for node in mat.node_tree.nodes:
                if node.type == 'BSDF_PRINCIPLED':
                    bsdf = node
                    break
            if bsdf:
                color = list(bsdf.inputs["Base Color"].default_value)
                info["materials"].append({
                    "name": mat.name,
                    "color": color,
                    "metallic": bsdf.inputs["Metallic"].default_value,
                    "roughness": bsdf.inputs["Roughness"].default_value,
                })

    # バウンディングボックス
    bbox = [obj.matrix_world @ Vector(corner) for corner in obj.bound_box]
    info["bounds"] = {
        "min": [min(v[i] for v in bbox) for i in range(3)],
        "max": [max(v[i] for v in bbox) for i in range(3)],
    }

    # 頂点カラー
    info["has_vertex_colors"] = len(mesh.vertex_colors) > 0

    return json.dumps(info, indent=2)
'''


def get_blender_evaluate_code() -> str:
    """Blender内で実行する評価コードを取得"""
    return BLENDER_EVALUATE_CODE


def evaluate_style_from_blender(model_json: str, category: str, challenge: Optional[Dict] = None) -> Dict:
    """
    Blenderから取得したJSON情報を使って評価を実行。

    Args:
        model_json: evaluate_style_in_blender()の戻り値
        category: モデルカテゴリ
        challenge: 課題定義（オプション）

    Returns:
        評価結果
    """
    model_info = json.loads(model_json)
    return evaluate_model(model_info, category, challenge)
