#!/usr/bin/env python3
"""
VLM Visual Bug Checker for Idle Factory

Uses Claude's vision capabilities to detect visual bugs in game screenshots.
Supports multiple check levels: quick, standard, thorough.

Usage:
    python visual_checker.py screenshot.png
    python visual_checker.py screenshot.png --level thorough
    python visual_checker.py --batch screenshots/*.png
"""

import anthropic
import base64
import sys
import json
import argparse
from pathlib import Path
from datetime import datetime
from typing import Optional

# Check levels with different prompts
CHECK_LEVELS = {
    "quick": {
        "description": "Basic visual sanity check (5 items)",
        "prompt": """このゲームのスクリーンショットを素早くチェックしてください。

重大な問題のみ報告:
1. 画面が真っ黒/真っ白でないか
2. ピンク/マゼンタのテクスチャ抜けがないか
3. UI要素が表示されているか
4. 明らかなクラッシュ/エラー表示がないか
5. 地形が表示されているか

問題があれば「FAIL: 理由」、なければ「PASS」と1行で回答。"""
    },

    "standard": {
        "description": "Standard visual check (10 items)",
        "prompt": """このゲームのスクリーンショットを確認してください。
工場建設・自動化ゲームです。

以下をチェック:
1. テクスチャ抜け（ピンク/マゼンタ色、黒い穴）
2. モデル不良（変形、浮遊、地面へのめり込み）
3. UI崩れ（文字重なり、枠はみ出し、読めない文字）
4. 不自然な色（真っ黒、真っ白、蛍光色の塊）
5. チャンク境界の隙間（グリッド状の黒い線）
6. コンベアベルトの表示（矢印、ベルトテクスチャ）
7. 機械の表示（Miner, Furnace, Crusherの形状）
8. ホットバーUI（画面下部に表示されているか）
9. 地形の自然さ（草、石、鉱石の分布）
10. ライティング（極端に暗い/明るい部分がないか）

結果をJSON形式で:
{
  "status": "PASS" or "FAIL",
  "issues": ["問題1", "問題2"],
  "warnings": ["軽微な問題"],
  "score": 0-100
}"""
    },

    "thorough": {
        "description": "Detailed visual inspection (20+ items)",
        "prompt": """このゲームのスクリーンショットを詳細に検査してください。
Minecraft風の工場建設・自動化ゲーム（ボクセルベース）です。

【テクスチャ・モデル検査】
1. テクスチャ抜け（ピンク/マゼンタ = missing texture）
2. UV座標ずれ（テクスチャが伸びている、歪んでいる）
3. ミップマップ問題（遠くのテクスチャがちらつく）
4. モデル破損（頂点が飛んでいる、面が裏返っている）
5. LOD問題（近くなのに低解像度モデル）

【機械・コンベア検査】
6. コンベアベルトのテクスチャ（矢印が見えるか）
7. コンベア方向の整合性（矢印の向きと接続が一致）
8. 機械モデルの完全性（欠けている部分がないか）
9. 機械の向き（facing方向が視覚的に分かるか）
10. アイテムの表示（コンベア上のアイテムが見えるか）

【ワールド・地形検査】
11. チャンク境界（黒い隙間、段差、不連続）
12. 地形生成の自然さ（浮遊ブロック、穴）
13. バイオーム遷移（急激な色変化がないか）
14. 水面/溶岩の表示（あれば正常か）
15. 影の整合性（光源と影の方向が一致）

【UI検査】
16. ホットバー表示（9スロット、選択表示）
17. デバッグ情報（FPS表示があれば読めるか）
18. クエストUI（表示されていれば正常か）
19. インベントリUI（開いていれば正常か）
20. 文字の可読性（フォントが正しく表示）
21. カーソル表示（クロスヘアが中央にあるか）

【パフォーマンス指標】（表示があれば）
22. FPS値（30以上が望ましい）
23. 描画負荷の兆候（ちらつき、残像）

詳細レポートをJSON形式で:
{
  "status": "PASS" or "FAIL" or "WARNING",
  "score": 0-100,
  "critical_issues": ["重大な問題（ゲーム不能）"],
  "major_issues": ["主要な問題（体験を損なう）"],
  "minor_issues": ["軽微な問題（見た目だけ）"],
  "observations": ["正常だが注目すべき点"],
  "recommendations": ["改善提案"]
}"""
    },

    "conveyor": {
        "description": "Conveyor-specific detailed check",
        "prompt": """このスクリーンショットのコンベアベルトを詳細検査してください。

【コンベア専用チェック】
1. ベルトテクスチャ
   - 矢印マークが表示されているか
   - ベルトの色（グレー系が正常）
   - テクスチャの繰り返しパターン

2. 方向と接続
   - 直線コンベア: 矢印が進行方向を指している
   - コーナー: L字に曲がっている、矢印が曲がる方向
   - T字分岐: 3方向に接続、分岐が視覚的に明確
   - スプリッター: 1入力→2出力が分かる

3. アイテム表示
   - コンベア上にアイテムが見えるか
   - アイテムが浮いていないか
   - アイテムサイズが適切か

4. 機械との接続
   - Minerの出力がコンベアに繋がっているか
   - Furnace/Crusherの入力がコンベアから来ているか
   - 接続部分に隙間がないか

5. 視覚的整合性
   - 全コンベアが同じ高さにあるか
   - 色味が統一されているか
   - アニメーション（動いて見えるか）

JSON形式で:
{
  "status": "PASS" or "FAIL",
  "conveyor_count": 推定数,
  "direction_issues": ["方向の問題"],
  "texture_issues": ["テクスチャの問題"],
  "connection_issues": ["接続の問題"],
  "item_issues": ["アイテム表示の問題"],
  "score": 0-100
}"""
    },

    "ui": {
        "description": "UI-specific detailed check",
        "prompt": """このスクリーンショットのUI要素を詳細検査してください。

【UI専用チェック】
1. ホットバー（画面下部）
   - 9個のスロットが表示されているか
   - 選択中のスロットがハイライトされているか
   - アイテムアイコンが表示されているか
   - 個数表示が読めるか

2. クロスヘア（画面中央）
   - 十字マークが中央に表示されているか
   - 色が背景と区別できるか

3. デバッグ情報（あれば）
   - FPS表示が読めるか
   - 座標表示が読めるか
   - フォントが正常か

4. インベントリUI（開いていれば）
   - グリッドレイアウトが整っているか
   - アイテムアイコンが正しいサイズか
   - スクロール/ページ送りボタン

5. 機械UI（開いていれば）
   - 入力/出力スロットが見えるか
   - 進捗バーが表示されているか
   - 閉じるボタン/方法が明確か

6. フォント・テキスト
   - 文字化けがないか
   - 重なりがないか
   - 読めるサイズか

JSON形式で:
{
  "status": "PASS" or "FAIL",
  "hotbar_ok": true/false,
  "crosshair_ok": true/false,
  "text_readable": true/false,
  "issues": ["問題リスト"],
  "score": 0-100
}"""
    },

    "chunk": {
        "description": "Chunk boundary specific check",
        "prompt": """このスクリーンショットでチャンク境界の問題を検査してください。

【チャンク境界チェック】
ボクセルゲームでは16x16ブロックごとにチャンク分割されています。

1. 黒い隙間
   - グリッド状（16ブロック間隔）の黒い線がないか
   - 地面に細い黒い割れ目がないか
   - 壁に縦の黒い線がないか

2. 地形の不連続
   - チャンク境界で急に地形が変わっていないか
   - 段差が不自然にないか
   - 草→石の遷移が急すぎないか

3. メッシュの欠け
   - ブロックの面が片方だけ表示されていないか
   - 透けて向こうが見える部分がないか
   - 床に穴が開いていないか

4. ライティングの不連続
   - チャンク境界で急に明るさが変わっていないか
   - 影が途中で切れていないか

重点確認: 画面内で16ブロック間隔のパターンを探す

JSON形式で:
{
  "status": "PASS" or "FAIL",
  "black_gaps_found": true/false,
  "terrain_discontinuity": true/false,
  "mesh_holes": true/false,
  "lighting_issues": true/false,
  "details": "詳細説明",
  "score": 0-100
}"""
    }
}


def encode_image(image_path: str) -> str:
    """Encode image to base64"""
    with open(image_path, "rb") as f:
        return base64.standard_b64encode(f.read()).decode("utf-8")


def get_media_type(image_path: str) -> str:
    """Get media type from file extension"""
    ext = Path(image_path).suffix.lower()
    return {
        ".png": "image/png",
        ".jpg": "image/jpeg",
        ".jpeg": "image/jpeg",
        ".gif": "image/gif",
        ".webp": "image/webp",
    }.get(ext, "image/png")


def check_screenshot(
    image_path: str,
    level: str = "standard",
    model: str = "claude-sonnet-4-20250514"
) -> dict:
    """
    Check a screenshot for visual bugs using Claude's vision.

    Args:
        image_path: Path to the screenshot
        level: Check level (quick, standard, thorough, conveyor, ui, chunk)
        model: Claude model to use

    Returns:
        dict with check results
    """
    if level not in CHECK_LEVELS:
        raise ValueError(f"Unknown level: {level}. Available: {list(CHECK_LEVELS.keys())}")

    client = anthropic.Anthropic()

    image_data = encode_image(image_path)
    media_type = get_media_type(image_path)
    prompt = CHECK_LEVELS[level]["prompt"]

    response = client.messages.create(
        model=model,
        max_tokens=2048,
        messages=[{
            "role": "user",
            "content": [
                {
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": media_type,
                        "data": image_data,
                    },
                },
                {
                    "type": "text",
                    "text": prompt
                }
            ],
        }]
    )

    result_text = response.content[0].text

    # Try to parse JSON from response
    try:
        # Find JSON in response
        if "{" in result_text:
            json_start = result_text.index("{")
            json_end = result_text.rindex("}") + 1
            result_json = json.loads(result_text[json_start:json_end])
        else:
            # For quick check, parse simple PASS/FAIL
            if result_text.strip().startswith("PASS"):
                result_json = {"status": "PASS", "score": 100}
            else:
                result_json = {"status": "FAIL", "raw": result_text}
    except (json.JSONDecodeError, ValueError):
        result_json = {"status": "UNKNOWN", "raw": result_text}

    return {
        "image": image_path,
        "level": level,
        "timestamp": datetime.now().isoformat(),
        "model": model,
        "result": result_json,
        "raw_response": result_text,
    }


def check_multiple(
    image_paths: list,
    level: str = "standard",
    model: str = "claude-sonnet-4-20250514"
) -> list:
    """Check multiple screenshots"""
    results = []
    for i, path in enumerate(image_paths):
        print(f"[{i+1}/{len(image_paths)}] Checking {path}...")
        try:
            result = check_screenshot(path, level, model)
            results.append(result)

            status = result["result"].get("status", "UNKNOWN")
            score = result["result"].get("score", "?")
            print(f"  -> {status} (score: {score})")

        except Exception as e:
            print(f"  -> ERROR: {e}")
            results.append({
                "image": path,
                "level": level,
                "error": str(e)
            })

    return results


def print_report(results: list):
    """Print a summary report"""
    print("\n" + "=" * 60)
    print("VLM Visual Check Report")
    print("=" * 60)

    passed = sum(1 for r in results if r.get("result", {}).get("status") == "PASS")
    failed = sum(1 for r in results if r.get("result", {}).get("status") == "FAIL")
    warnings = sum(1 for r in results if r.get("result", {}).get("status") == "WARNING")
    errors = sum(1 for r in results if "error" in r)

    print(f"\nSummary: {passed} PASS, {failed} FAIL, {warnings} WARNING, {errors} ERROR")
    print(f"Total: {len(results)} screenshots\n")

    # Print failures
    for r in results:
        if r.get("result", {}).get("status") == "FAIL":
            print(f"FAIL: {r['image']}")
            issues = r.get("result", {})
            for key in ["critical_issues", "major_issues", "issues"]:
                if key in issues and issues[key]:
                    for issue in issues[key]:
                        print(f"  - {issue}")
            print()

    return failed == 0 and errors == 0


def main():
    parser = argparse.ArgumentParser(
        description="VLM Visual Bug Checker for Idle Factory",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Check Levels:
  quick     - Fast sanity check (5 items, ~2 sec)
  standard  - Normal visual check (10 items, ~5 sec)  [default]
  thorough  - Detailed inspection (20+ items, ~10 sec)
  conveyor  - Conveyor-specific check
  ui        - UI-specific check
  chunk     - Chunk boundary check

Examples:
  %(prog)s screenshot.png
  %(prog)s screenshot.png --level thorough
  %(prog)s --batch screenshots/*.png --level quick
  %(prog)s screenshot.png --output report.json
"""
    )

    parser.add_argument("images", nargs="*", help="Screenshot paths to check")
    parser.add_argument("--batch", nargs="+", help="Batch mode: check multiple files")
    parser.add_argument("--level", "-l", default="standard",
                       choices=list(CHECK_LEVELS.keys()),
                       help="Check level (default: standard)")
    parser.add_argument("--model", "-m", default="claude-sonnet-4-20250514",
                       help="Claude model to use")
    parser.add_argument("--output", "-o", help="Output JSON file for results")
    parser.add_argument("--list-levels", action="store_true",
                       help="List available check levels")

    args = parser.parse_args()

    if args.list_levels:
        print("Available check levels:\n")
        for name, info in CHECK_LEVELS.items():
            print(f"  {name:12} - {info['description']}")
        return 0

    # Collect all images
    images = list(args.images or [])
    if args.batch:
        images.extend(args.batch)

    if not images:
        parser.print_help()
        return 1

    # Filter existing files
    existing = [p for p in images if Path(p).exists()]
    if len(existing) < len(images):
        missing = set(images) - set(existing)
        print(f"Warning: {len(missing)} files not found: {missing}")

    if not existing:
        print("Error: No valid image files found")
        return 1

    # Run checks
    print(f"Running {args.level} check on {len(existing)} image(s)...")
    print(f"Using model: {args.model}\n")

    if len(existing) == 1:
        results = [check_screenshot(existing[0], args.level, args.model)]
    else:
        results = check_multiple(existing, args.level, args.model)

    # Print report
    success = print_report(results)

    # Save JSON if requested
    if args.output:
        with open(args.output, "w") as f:
            json.dump(results, f, indent=2, ensure_ascii=False)
        print(f"\nResults saved to: {args.output}")

    return 0 if success else 1


if __name__ == "__main__":
    sys.exit(main())
