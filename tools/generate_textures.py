#!/usr/bin/env python3
"""
Blender-based Texture Generator
16x16ピクセルのブロックテクスチャを生成

使い方:
    blender --background --python generate_textures.py
"""

import bpy
import os
import random
from pathlib import Path


def create_procedural_texture(name: str, colors: list, pattern: str = "noise") -> None:
    """Blenderでプロシージャルテクスチャを生成してPNGに保存"""

    # 新しい画像を作成 (16x16)
    size = 16
    img = bpy.data.images.new(name, width=size, height=size, alpha=True)

    # ピクセルデータを生成
    pixels = []

    for y in range(size):
        for x in range(size):
            if pattern == "solid":
                r, g, b = colors[0]
            elif pattern == "noise":
                base = colors[0]
                var = 20
                r = max(0, min(255, base[0] + random.randint(-var, var))) / 255.0
                g = max(0, min(255, base[1] + random.randint(-var, var))) / 255.0
                b = max(0, min(255, base[2] + random.randint(-var, var))) / 255.0
                pixels.extend([r, g, b, 1.0])
                continue
            elif pattern == "grass_side":
                grass = colors[0]
                dirt = colors[1]
                if y > size - 4:  # 上部は草
                    base = grass
                elif y > size - 6:  # 境界
                    base = grass if random.random() > 0.5 else dirt
                else:  # 下部は土
                    base = dirt
                var = 15
                r = max(0, min(255, base[0] + random.randint(-var, var))) / 255.0
                g = max(0, min(255, base[1] + random.randint(-var, var))) / 255.0
                b = max(0, min(255, base[2] + random.randint(-var, var))) / 255.0
                pixels.extend([r, g, b, 1.0])
                continue
            elif pattern == "ore":
                stone = colors[0]
                ore = colors[1]
                # 基本は石
                base = stone
                # ランダムに鉱石の塊
                if random.random() < 0.15:
                    base = ore
                var = 15
                r = max(0, min(255, base[0] + random.randint(-var, var))) / 255.0
                g = max(0, min(255, base[1] + random.randint(-var, var))) / 255.0
                b = max(0, min(255, base[2] + random.randint(-var, var))) / 255.0
                pixels.extend([r, g, b, 1.0])
                continue
            elif pattern == "bedrock":
                if random.random() < 0.3:
                    v = random.randint(10, 30) / 255.0
                else:
                    v = random.randint(40, 60) / 255.0
                pixels.extend([v, v, v, 1.0])
                continue

            # デフォルト（solid）
            r, g, b = colors[0]
            r, g, b = r / 255.0, g / 255.0, b / 255.0
            pixels.extend([r, g, b, 1.0])

    img.pixels = pixels
    img.update()

    return img


def save_texture(img, output_path: str):
    """画像をPNGとして保存"""
    img.filepath_raw = output_path
    img.file_format = 'PNG'
    img.save()
    print(f"Saved: {output_path}")


def generate_all_textures():
    """全テクスチャを生成"""
    output_dir = Path("/home/bacon/idle_factory/assets/textures/blocks")
    output_dir.mkdir(parents=True, exist_ok=True)

    textures = {
        "grass_top": ([(76, 153, 0)], "noise"),
        "grass_side": ([(76, 153, 0), (139, 90, 43)], "grass_side"),
        "dirt": ([(139, 90, 43)], "noise"),
        "stone": ([(128, 128, 128)], "noise"),
        "iron_ore": ([(128, 128, 128), (150, 110, 80)], "ore"),
        "copper_ore": ([(128, 128, 128), (184, 115, 51)], "ore"),
        "coal_ore": ([(128, 128, 128), (30, 30, 30)], "ore"),
        "sand": ([(219, 194, 134)], "noise"),
        "bedrock": ([(40, 40, 40)], "bedrock"),
    }

    for name, (colors, pattern) in textures.items():
        img = create_procedural_texture(name, colors, pattern)
        save_texture(img, str(output_dir / f"{name}.png"))
        bpy.data.images.remove(img)

    print(f"Generated {len(textures)} textures")


if __name__ == "__main__":
    generate_all_textures()
