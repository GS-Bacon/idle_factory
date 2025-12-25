#!/usr/bin/env python3
"""
デザインパターン準拠検証スクリプト

使用方法:
  python3 tools/validate_patterns.py

検証するパターン:
  P1: 段階的複雑化 - 入力数の分布が適切か
  R2: 比率美学 - 整数比か
  R3: 深さ制限 - Tier毎に区切り、最大8階層許容
"""

import yaml
import sys
from pathlib import Path
from collections import defaultdict


def load_recipes():
    """レシピをロード"""
    recipe_path = Path("assets/data/recipes/vanilla.yaml")
    if not recipe_path.exists():
        print(f"ERROR: {recipe_path} not found")
        sys.exit(1)

    with open(recipe_path, 'r') as f:
        return yaml.safe_load(f)


def validate_p1_gradual_complexity(recipes):
    """P1: 段階的複雑化を検証"""
    print("=== P1: 段階的複雑化 ===")

    input_distribution = defaultdict(int)
    for recipe in recipes:
        input_count = len(recipe.get('inputs', []))
        input_distribution[input_count] += 1

    # 入力数の分布を表示
    print("入力数の分布:")
    for count in sorted(input_distribution.keys()):
        bar = '█' * (input_distribution[count] // 2)
        print(f"  {count}入力: {input_distribution[count]:3d}件 {bar}")

    # 検証: 1-2入力が過半数か
    simple_recipes = input_distribution[1] + input_distribution[2]
    total = sum(input_distribution.values())
    ratio = simple_recipes / total * 100

    if ratio >= 50:
        print(f"✅ シンプルなレシピ(1-2入力)が{ratio:.1f}%で過半数")
        return True
    else:
        print(f"⚠️ シンプルなレシピが{ratio:.1f}%で過半数未満")
        return False


def validate_r2_integer_ratios(recipes):
    """R2: 比率美学 - 全て整数か"""
    print("\n=== R2: 比率美学 ===")

    violations = []
    for recipe in recipes:
        for inp in recipe.get('inputs', []):
            count = inp.get('count', 0)
            if not isinstance(count, int) or count != int(count):
                violations.append((recipe['id'], 'input', inp['item'], count))

        for out in recipe.get('outputs', []):
            count = out.get('count', 0)
            if not isinstance(count, int) or count != int(count):
                violations.append((recipe['id'], 'output', out['item'], count))

    if not violations:
        print("✅ 全レシピが整数比")
        return True
    else:
        print(f"❌ {len(violations)}件の非整数比:")
        for v in violations[:5]:
            print(f"  {v[0]}: {v[2]} = {v[3]}")
        return False


def validate_r3_depth_limit(recipes):
    """R3: 深さ制限 - 依存チェーンが8階層以内か（Tier毎にブレークポイント）"""
    print("\n=== R3: 深さ制限 (8階層以内) ===")

    # レシピの出力→入力の依存関係を構築
    item_to_recipe = {}
    for recipe in recipes:
        for out in recipe.get('outputs', []):
            item_to_recipe[out['item']] = recipe['id']

    def get_depth(item, visited=None):
        if visited is None:
            visited = set()
        if item in visited:
            return 0  # 循環参照
        if item not in item_to_recipe:
            return 1  # 原料

        visited.add(item)
        recipe_id = item_to_recipe[item]
        recipe = next(r for r in recipes if r['id'] == recipe_id)

        max_depth = 0
        for inp in recipe.get('inputs', []):
            depth = get_depth(inp['item'], visited.copy())
            max_depth = max(max_depth, depth)

        return max_depth + 1

    # 全出力アイテムの深さを計算
    depths = {}
    for recipe in recipes:
        for out in recipe.get('outputs', []):
            depths[out['item']] = get_depth(out['item'])

    # 深さが8を超えるものを検出（工場ゲームでは深い依存は自然）
    violations = [(item, depth) for item, depth in depths.items() if depth > 8]

    max_depth = max(depths.values()) if depths else 0
    print(f"最大深さ: {max_depth}階層")

    if not violations:
        print("✅ 全レシピが8階層以内")
        return True
    else:
        print(f"❌ {len(violations)}件が8階層超過:")
        for item, depth in sorted(violations, key=lambda x: -x[1])[:5]:
            print(f"  {item}: {depth}階層")
        return False


def main():
    print("デザインパターン準拠検証\n")

    recipes = load_recipes()
    print(f"検証対象: {len(recipes)}件のレシピ\n")

    results = []
    results.append(('P1', validate_p1_gradual_complexity(recipes)))
    results.append(('R2', validate_r2_integer_ratios(recipes)))
    results.append(('R3', validate_r3_depth_limit(recipes)))

    print("\n=== 結果サマリ ===")
    all_passed = True
    for pattern, passed in results:
        status = "✅ PASS" if passed else "❌ FAIL"
        print(f"  {pattern}: {status}")
        if not passed:
            all_passed = False

    if all_passed:
        print("\n全パターン準拠 ✅")
        return 0
    else:
        print("\nパターン違反あり ❌")
        return 1


if __name__ == "__main__":
    sys.exit(main())
