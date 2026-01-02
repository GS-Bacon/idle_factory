#!/usr/bin/env python3
"""
Pixel-based screenshot comparison for visual regression testing.
Compares current screenshots against baseline images.
"""

import sys
import os
import argparse
from pathlib import Path

try:
    from PIL import Image, ImageChops
except ImportError:
    print("Error: PIL not installed. Install with: pip install Pillow")
    sys.exit(1)


def compare_images(baseline_path: str, current_path: str, threshold: float = 0.01):
    """
    Compare two images and return the difference ratio.

    Args:
        baseline_path: Path to the baseline (expected) image
        current_path: Path to the current (test) image
        threshold: Maximum allowed difference ratio (0.01 = 1%)

    Returns:
        Dictionary with match result and diff information
    """
    try:
        baseline = Image.open(baseline_path)
        current = Image.open(current_path)
    except FileNotFoundError as e:
        return {
            "match": False,
            "error": f"File not found: {e.filename}",
            "diff_ratio": 1.0,
        }

    # Ensure same size
    if baseline.size != current.size:
        return {
            "match": False,
            "error": f"Size mismatch: baseline={baseline.size}, current={current.size}",
            "diff_ratio": 1.0,
        }

    # Convert to same mode
    if baseline.mode != current.mode:
        baseline = baseline.convert("RGB")
        current = current.convert("RGB")

    # Calculate difference
    diff = ImageChops.difference(baseline, current)

    # Calculate diff ratio
    # Sum of all pixel differences / max possible difference
    diff_data = list(diff.getdata())
    if len(diff_data[0]) == 4:  # RGBA
        total_diff = sum(sum(p[:3]) for p in diff_data)  # Ignore alpha
        max_diff = baseline.width * baseline.height * 3 * 255
    else:  # RGB
        total_diff = sum(sum(p) for p in diff_data)
        max_diff = baseline.width * baseline.height * len(diff_data[0]) * 255

    diff_ratio = total_diff / max_diff if max_diff > 0 else 0

    return {
        "match": diff_ratio < threshold,
        "diff_ratio": diff_ratio,
        "diff_percent": diff_ratio * 100,
        "threshold": threshold,
        "baseline": baseline_path,
        "current": current_path,
    }


def save_diff_image(baseline_path: str, current_path: str, output_path: str):
    """
    Create and save a visual diff image showing differences.
    """
    try:
        baseline = Image.open(baseline_path).convert("RGB")
        current = Image.open(current_path).convert("RGB")
    except FileNotFoundError as e:
        print(f"Error: {e}")
        return False

    if baseline.size != current.size:
        print(f"Size mismatch: {baseline.size} vs {current.size}")
        return False

    # Create diff image with enhanced visibility
    diff = ImageChops.difference(baseline, current)

    # Amplify differences for visibility
    diff = diff.point(lambda x: min(255, x * 10))

    # Save diff image
    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    diff.save(output_path)
    return True


def main():
    parser = argparse.ArgumentParser(
        description="Compare screenshots for visual regression testing"
    )
    parser.add_argument("baseline", help="Path to baseline image")
    parser.add_argument("current", help="Path to current image")
    parser.add_argument(
        "--threshold",
        type=float,
        default=0.01,
        help="Maximum allowed difference ratio (default: 0.01 = 1%%)",
    )
    parser.add_argument(
        "--diff-output",
        help="Path to save diff image (optional)",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output result as JSON",
    )

    args = parser.parse_args()

    result = compare_images(args.baseline, args.current, args.threshold)

    if args.diff_output and not result.get("error"):
        save_diff_image(args.baseline, args.current, args.diff_output)
        result["diff_image"] = args.diff_output

    if args.json:
        import json
        print(json.dumps(result, indent=2))
    else:
        if result.get("error"):
            print(f"ERROR: {result['error']}")
            sys.exit(2)
        elif result["match"]:
            print(f"PASS: Images match (diff={result['diff_percent']:.4f}%)")
            sys.exit(0)
        else:
            print(f"FAIL: Images differ (diff={result['diff_percent']:.4f}% > threshold={args.threshold*100}%)")
            sys.exit(1)


if __name__ == "__main__":
    main()
