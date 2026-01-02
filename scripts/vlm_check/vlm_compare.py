#!/usr/bin/env python3
"""
VLM-based screenshot comparison using Claude Vision API.
Compares two game screenshots to detect visual differences.
"""

import sys
import os
import base64
import json
import argparse
from pathlib import Path

try:
    import anthropic
except ImportError:
    print("Error: anthropic library not installed. Install with: pip install anthropic")
    sys.exit(1)


def load_image_as_base64(image_path: str) -> str:
    """Load an image file and return its base64 encoding."""
    with open(image_path, "rb") as f:
        return base64.standard_b64encode(f.read()).decode("utf-8")


def get_image_media_type(image_path: str) -> str:
    """Determine the media type based on file extension."""
    ext = Path(image_path).suffix.lower()
    media_types = {
        ".png": "image/png",
        ".jpg": "image/jpeg",
        ".jpeg": "image/jpeg",
        ".gif": "image/gif",
        ".webp": "image/webp",
    }
    return media_types.get(ext, "image/png")


def vlm_compare(baseline_path: str, current_path: str) -> dict:
    """
    Compare two screenshots using Claude Vision API.

    Args:
        baseline_path: Path to the baseline (expected) image
        current_path: Path to the current (test) image

    Returns:
        Dictionary with comparison results
    """
    client = anthropic.Anthropic()

    baseline_data = load_image_as_base64(baseline_path)
    current_data = load_image_as_base64(current_path)
    baseline_type = get_image_media_type(baseline_path)
    current_type = get_image_media_type(current_path)

    prompt = """Compare these two game screenshots.

Image 1: Baseline (expected/correct state)
Image 2: Current (state being tested)

Analyze and report:
1. Layout differences (UI element positions, sizes)
2. Color/texture differences
3. Missing elements (present in baseline but not current)
4. New elements (present in current but not baseline)
5. Position/alignment issues

Respond in JSON format:
{
    "identical": true/false,
    "differences": ["list of specific differences"],
    "severity": "none" | "minor" | "major" | "critical",
    "summary": "one-line summary of comparison result"
}

If images are identical or nearly identical, set identical=true and differences=[].
Minor = small visual tweaks, Major = noticeable UI changes, Critical = broken/missing UI."""

    try:
        response = client.messages.create(
            model="claude-sonnet-4-20250514",
            max_tokens=1024,
            messages=[
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": baseline_type,
                                "data": baseline_data,
                            },
                        },
                        {
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": current_type,
                                "data": current_data,
                            },
                        },
                        {
                            "type": "text",
                            "text": prompt,
                        },
                    ],
                }
            ],
        )

        # Parse the JSON response
        response_text = response.content[0].text
        # Try to extract JSON from the response
        try:
            # Find JSON in response (it might be wrapped in markdown code blocks)
            if "```json" in response_text:
                json_str = response_text.split("```json")[1].split("```")[0]
            elif "```" in response_text:
                json_str = response_text.split("```")[1].split("```")[0]
            else:
                json_str = response_text

            result = json.loads(json_str.strip())
            result["success"] = True
            return result
        except json.JSONDecodeError:
            return {
                "success": True,
                "identical": False,
                "differences": [response_text],
                "severity": "unknown",
                "summary": "Could not parse structured response",
                "raw_response": response_text,
            }

    except anthropic.APIError as e:
        return {
            "success": False,
            "error": str(e),
            "identical": False,
            "severity": "error",
        }


def main():
    parser = argparse.ArgumentParser(
        description="Compare game screenshots using Claude Vision API"
    )
    parser.add_argument("baseline", help="Path to baseline image")
    parser.add_argument("current", help="Path to current image")
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output result as JSON",
    )

    args = parser.parse_args()

    if not os.path.exists(args.baseline):
        print(f"Error: Baseline image not found: {args.baseline}")
        sys.exit(1)

    if not os.path.exists(args.current):
        print(f"Error: Current image not found: {args.current}")
        sys.exit(1)

    result = vlm_compare(args.baseline, args.current)

    if args.json:
        print(json.dumps(result, indent=2))
    else:
        if not result.get("success", False):
            print(f"ERROR: {result.get('error', 'Unknown error')}")
            sys.exit(2)

        if result.get("identical", False):
            print(f"PASS: {result.get('summary', 'Images are identical')}")
            sys.exit(0)
        else:
            print(f"DIFF: {result.get('summary', 'Differences detected')}")
            print(f"Severity: {result.get('severity', 'unknown')}")
            if result.get("differences"):
                print("Differences:")
                for diff in result["differences"]:
                    print(f"  - {diff}")
            sys.exit(1 if result.get("severity") in ["major", "critical"] else 0)


if __name__ == "__main__":
    main()
