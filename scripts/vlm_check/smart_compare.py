#!/usr/bin/env python3
"""
Smart screenshot comparison using SSIM and perceptual hashing.
No API required - runs entirely locally.

Comparison methods:
1. SSIM (Structural Similarity Index) - Detects structural changes
2. Perceptual Hash - Detects if images are semantically similar
3. Edge detection comparison - Detects UI layout changes
"""

import sys
import os
import argparse
import json
from pathlib import Path

try:
    from PIL import Image
    import numpy as np
    from skimage.metrics import structural_similarity as ssim
    from skimage import io, color, feature
    import imagehash
except ImportError as e:
    print(f"Error: Missing library. Install with: pip install Pillow scikit-image imagehash numpy")
    print(f"Details: {e}")
    sys.exit(1)


def load_image(path: str) -> np.ndarray:
    """Load image as numpy array."""
    img = io.imread(path)
    # Convert to RGB if RGBA
    if img.ndim == 3 and img.shape[2] == 4:
        img = img[:, :, :3]
    return img


def compute_ssim(img1: np.ndarray, img2: np.ndarray) -> float:
    """Compute SSIM between two images."""
    # Convert to grayscale for SSIM
    if img1.ndim == 3:
        gray1 = color.rgb2gray(img1)
    else:
        gray1 = img1

    if img2.ndim == 3:
        gray2 = color.rgb2gray(img2)
    else:
        gray2 = img2

    # Resize if different sizes
    if gray1.shape != gray2.shape:
        from skimage.transform import resize
        gray2 = resize(gray2, gray1.shape, anti_aliasing=True)

    # Compute SSIM
    score, _ = ssim(gray1, gray2, full=True, data_range=1.0)
    return float(score)


def compute_phash(path: str, hash_size: int = 16) -> str:
    """Compute perceptual hash of an image."""
    img = Image.open(path)
    return str(imagehash.phash(img, hash_size=hash_size))


def compute_hash_distance(hash1: str, hash2: str) -> int:
    """Compute Hamming distance between two hashes."""
    h1 = imagehash.hex_to_hash(hash1)
    h2 = imagehash.hex_to_hash(hash2)
    return h1 - h2


def detect_edge_changes(img1: np.ndarray, img2: np.ndarray) -> float:
    """Compare edge maps to detect UI layout changes."""
    # Convert to grayscale
    if img1.ndim == 3:
        gray1 = color.rgb2gray(img1)
    else:
        gray1 = img1

    if img2.ndim == 3:
        gray2 = color.rgb2gray(img2)
    else:
        gray2 = img2

    # Resize if needed
    if gray1.shape != gray2.shape:
        from skimage.transform import resize
        gray2 = resize(gray2, gray1.shape, anti_aliasing=True)

    # Detect edges using Canny
    edges1 = feature.canny(gray1, sigma=2)
    edges2 = feature.canny(gray2, sigma=2)

    # Compute similarity of edge maps
    intersection = np.logical_and(edges1, edges2).sum()
    union = np.logical_or(edges1, edges2).sum()

    if union == 0:
        return 1.0

    return float(intersection / union)


def analyze_difference(ssim_score: float, hash_dist: int, edge_sim: float) -> dict:
    """Analyze the differences and provide a human-readable summary."""
    issues = []
    severity = "none"

    # SSIM analysis
    if ssim_score < 0.5:
        issues.append("Major visual changes detected (low SSIM)")
        severity = "critical"
    elif ssim_score < 0.8:
        issues.append("Noticeable visual differences")
        severity = "major"
    elif ssim_score < 0.95:
        issues.append("Minor visual differences (lighting, anti-aliasing)")
        severity = "minor"

    # Hash distance analysis (0 = identical, higher = more different)
    if hash_dist > 20:
        issues.append("Images appear structurally different")
        if severity in ["none", "minor"]:
            severity = "major"
    elif hash_dist > 10:
        issues.append("Some structural differences detected")
        if severity == "none":
            severity = "minor"

    # Edge similarity analysis
    if edge_sim < 0.5:
        issues.append("UI layout has changed significantly")
        if severity in ["none", "minor"]:
            severity = "major"
    elif edge_sim < 0.8:
        issues.append("Some UI elements may have moved")
        if severity == "none":
            severity = "minor"

    if not issues:
        issues.append("Images are visually identical or nearly identical")

    return {
        "issues": issues,
        "severity": severity,
        "identical": ssim_score > 0.98 and hash_dist < 5
    }


def smart_compare(baseline_path: str, current_path: str) -> dict:
    """
    Compare two images using multiple methods.

    Returns a comprehensive analysis without requiring external APIs.
    """
    # Check files exist
    if not os.path.exists(baseline_path):
        return {"error": f"Baseline not found: {baseline_path}", "success": False}
    if not os.path.exists(current_path):
        return {"error": f"Current not found: {current_path}", "success": False}

    try:
        # Load images
        img1 = load_image(baseline_path)
        img2 = load_image(current_path)

        # Check size difference
        size_match = img1.shape[:2] == img2.shape[:2]

        # Compute metrics
        ssim_score = compute_ssim(img1, img2)

        hash1 = compute_phash(baseline_path)
        hash2 = compute_phash(current_path)
        hash_distance = compute_hash_distance(hash1, hash2)

        edge_similarity = detect_edge_changes(img1, img2)

        # Analyze
        analysis = analyze_difference(ssim_score, hash_distance, edge_similarity)

        return {
            "success": True,
            "identical": analysis["identical"],
            "metrics": {
                "ssim": round(ssim_score, 4),
                "hash_distance": hash_distance,
                "edge_similarity": round(edge_similarity, 4),
                "size_match": size_match
            },
            "severity": analysis["severity"],
            "issues": analysis["issues"],
            "summary": analysis["issues"][0] if analysis["issues"] else "No issues",
            "baseline": baseline_path,
            "current": current_path
        }

    except Exception as e:
        return {
            "success": False,
            "error": str(e)
        }


def main():
    parser = argparse.ArgumentParser(
        description="Smart image comparison using SSIM and perceptual hashing (no API required)"
    )
    parser.add_argument("baseline", help="Path to baseline image")
    parser.add_argument("current", help="Path to current image")
    parser.add_argument("--json", action="store_true", help="Output as JSON")
    parser.add_argument("--threshold", type=float, default=0.95,
                       help="SSIM threshold for pass/fail (default: 0.95)")

    args = parser.parse_args()

    result = smart_compare(args.baseline, args.current)

    if args.json:
        print(json.dumps(result, indent=2))
    else:
        if not result.get("success", False):
            print(f"ERROR: {result.get('error', 'Unknown error')}")
            sys.exit(2)

        metrics = result.get("metrics", {})
        print(f"=== Smart Image Comparison ===")
        print(f"SSIM Score: {metrics.get('ssim', 'N/A')} (1.0 = identical)")
        print(f"Hash Distance: {metrics.get('hash_distance', 'N/A')} (0 = identical)")
        print(f"Edge Similarity: {metrics.get('edge_similarity', 'N/A')} (1.0 = identical)")
        print(f"Size Match: {metrics.get('size_match', 'N/A')}")
        print()
        print(f"Severity: {result.get('severity', 'unknown')}")
        print(f"Identical: {result.get('identical', False)}")
        print()
        print("Issues:")
        for issue in result.get("issues", []):
            print(f"  - {issue}")

        # Exit code based on severity
        if result.get("severity") in ["major", "critical"]:
            sys.exit(1)
        elif metrics.get("ssim", 1.0) < args.threshold:
            sys.exit(1)
        else:
            sys.exit(0)


if __name__ == "__main__":
    main()
