#!/usr/bin/env python3
"""
Texture Generator - 地形ブロックのテクスチャ画像を生成

16x16ピクセルのシンプルなテクスチャを生成。
Astroneer風のテクスチャレス・カラーベースのスタイル。
"""

from PIL import Image, ImageDraw
import random
from pathlib import Path


def create_solid_texture(color: tuple, size: int = 16) -> Image.Image:
    """単色テクスチャ"""
    return Image.new('RGBA', (size, size), color)


def create_noise_texture(base_color: tuple, variation: int = 20, size: int = 16) -> Image.Image:
    """ノイズ入りテクスチャ"""
    img = Image.new('RGBA', (size, size))
    pixels = img.load()

    for y in range(size):
        for x in range(size):
            r = max(0, min(255, base_color[0] + random.randint(-variation, variation)))
            g = max(0, min(255, base_color[1] + random.randint(-variation, variation)))
            b = max(0, min(255, base_color[2] + random.randint(-variation, variation)))
            pixels[x, y] = (r, g, b, 255)

    return img


def create_grass_texture(size: int = 16) -> Image.Image:
    """草ブロック - 上面が緑、側面は土と草の境界"""
    img = Image.new('RGBA', (size, size))
    pixels = img.load()

    grass_color = (76, 153, 0)
    dirt_color = (139, 90, 43)

    for y in range(size):
        for x in range(size):
            # 上部3ピクセルは草
            if y < 3:
                r = grass_color[0] + random.randint(-15, 15)
                g = grass_color[1] + random.randint(-15, 15)
                b = grass_color[2] + random.randint(-10, 10)
            # 境界部分はランダムに草か土
            elif y < 5:
                if random.random() > 0.5:
                    r, g, b = grass_color
                else:
                    r, g, b = dirt_color
                r += random.randint(-10, 10)
                g += random.randint(-10, 10)
                b += random.randint(-10, 10)
            # 下部は土
            else:
                r = dirt_color[0] + random.randint(-15, 15)
                g = dirt_color[1] + random.randint(-15, 15)
                b = dirt_color[2] + random.randint(-10, 10)

            pixels[x, y] = (max(0, min(255, r)), max(0, min(255, g)), max(0, min(255, b)), 255)

    return img


def create_grass_top_texture(size: int = 16) -> Image.Image:
    """草ブロック上面 - 緑のみ"""
    return create_noise_texture((76, 153, 0), variation=15, size=size)


def create_dirt_texture(size: int = 16) -> Image.Image:
    """土ブロック"""
    return create_noise_texture((139, 90, 43), variation=15, size=size)


def create_stone_texture(size: int = 16) -> Image.Image:
    """石ブロック - グレーにちょっとした模様"""
    img = Image.new('RGBA', (size, size))
    pixels = img.load()

    base = (128, 128, 128)

    for y in range(size):
        for x in range(size):
            # 時々暗い点を入れる
            if random.random() < 0.1:
                r = base[0] - 30 + random.randint(-10, 10)
                g = base[1] - 30 + random.randint(-10, 10)
                b = base[2] - 30 + random.randint(-10, 10)
            else:
                r = base[0] + random.randint(-15, 15)
                g = base[1] + random.randint(-15, 15)
                b = base[2] + random.randint(-15, 15)

            pixels[x, y] = (max(0, min(255, r)), max(0, min(255, g)), max(0, min(255, b)), 255)

    return img


def create_ore_texture(base_color: tuple, ore_color: tuple, ore_density: float = 0.15, size: int = 16) -> Image.Image:
    """鉱石テクスチャ - 石ベースに鉱石の点"""
    img = create_stone_texture(size)
    pixels = img.load()

    # 鉱石の塊を数個配置
    num_clusters = random.randint(2, 4)
    for _ in range(num_clusters):
        cx = random.randint(2, size - 3)
        cy = random.randint(2, size - 3)
        cluster_size = random.randint(2, 4)

        for dy in range(-cluster_size // 2, cluster_size // 2 + 1):
            for dx in range(-cluster_size // 2, cluster_size // 2 + 1):
                px, py = cx + dx, cy + dy
                if 0 <= px < size and 0 <= py < size:
                    if random.random() < 0.7:  # 塊内でもランダム
                        r = ore_color[0] + random.randint(-20, 20)
                        g = ore_color[1] + random.randint(-20, 20)
                        b = ore_color[2] + random.randint(-20, 20)
                        pixels[px, py] = (max(0, min(255, r)), max(0, min(255, g)), max(0, min(255, b)), 255)

    return img


def create_iron_ore_texture(size: int = 16) -> Image.Image:
    """鉄鉱石 - 茶色がかった色"""
    return create_ore_texture((128, 128, 128), (150, 110, 80), size=size)


def create_copper_ore_texture(size: int = 16) -> Image.Image:
    """銅鉱石 - オレンジ"""
    return create_ore_texture((128, 128, 128), (184, 115, 51), size=size)


def create_coal_ore_texture(size: int = 16) -> Image.Image:
    """石炭 - 黒"""
    return create_ore_texture((128, 128, 128), (30, 30, 30), size=size)


def create_sand_texture(size: int = 16) -> Image.Image:
    """砂ブロック"""
    return create_noise_texture((219, 194, 134), variation=15, size=size)


def create_water_texture(size: int = 16) -> Image.Image:
    """水ブロック - 半透明の青"""
    img = Image.new('RGBA', (size, size))
    pixels = img.load()

    base = (64, 164, 223)

    for y in range(size):
        for x in range(size):
            r = base[0] + random.randint(-10, 10)
            g = base[1] + random.randint(-10, 10)
            b = base[2] + random.randint(-10, 10)
            # 半透明
            pixels[x, y] = (max(0, min(255, r)), max(0, min(255, g)), max(0, min(255, b)), 180)

    return img


def create_bedrock_texture(size: int = 16) -> Image.Image:
    """岩盤 - 暗いグレーに黒い模様"""
    img = Image.new('RGBA', (size, size))
    pixels = img.load()

    for y in range(size):
        for x in range(size):
            if random.random() < 0.3:
                r = g = b = random.randint(10, 30)
            else:
                r = g = b = random.randint(40, 60)
            pixels[x, y] = (r, g, b, 255)

    return img


def generate_all_textures(output_dir: str = "assets/textures/blocks"):
    """全テクスチャを生成"""
    Path(output_dir).mkdir(parents=True, exist_ok=True)

    textures = {
        "grass_side": create_grass_texture,
        "grass_top": create_grass_top_texture,
        "dirt": create_dirt_texture,
        "stone": create_stone_texture,
        "iron_ore": create_iron_ore_texture,
        "copper_ore": create_copper_ore_texture,
        "coal_ore": create_coal_ore_texture,
        "sand": create_sand_texture,
        "water": create_water_texture,
        "bedrock": create_bedrock_texture,
    }

    for name, generator in textures.items():
        img = generator()
        path = Path(output_dir) / f"{name}.png"
        img.save(path)
        print(f"Saved: {path}")

    print(f"Generated {len(textures)} textures")


if __name__ == "__main__":
    generate_all_textures()
