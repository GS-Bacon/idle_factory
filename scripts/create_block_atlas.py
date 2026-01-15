#!/usr/bin/env python3
"""
Generate block texture atlas for idle_factory.
Creates 16x16 pixel art textures and combines them into a 128x128 atlas.
"""

import random
from PIL import Image, ImageDraw

# Atlas layout (8x8 grid, 16x16 tiles)
TILE_SIZE = 16
ATLAS_SIZE = 8  # tiles per row/column

# Texture indices
TEXTURES = {
    0: "stone",
    1: "grass_top",
    2: "grass_side",
    3: "iron_ore",
    4: "copper_ore",
    5: "coal",
    6: "dirt",
    7: "sand",
}

def add_noise(color, variation=20):
    """Add slight color variation for texture effect."""
    r, g, b = color
    return (
        max(0, min(255, r + random.randint(-variation, variation))),
        max(0, min(255, g + random.randint(-variation, variation))),
        max(0, min(255, b + random.randint(-variation, variation))),
    )

def create_stone_texture():
    """Gray stone with subtle variation."""
    img = Image.new('RGB', (TILE_SIZE, TILE_SIZE))
    base_color = (128, 128, 128)
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            img.putpixel((x, y), add_noise(base_color, 25))
    # Add some darker spots for depth
    for _ in range(8):
        x, y = random.randint(0, 15), random.randint(0, 15)
        img.putpixel((x, y), add_noise((100, 100, 100), 15))
    return img

def create_grass_top_texture():
    """Green grass top view."""
    img = Image.new('RGB', (TILE_SIZE, TILE_SIZE))
    base_color = (76, 153, 0)  # Grass green
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            img.putpixel((x, y), add_noise(base_color, 20))
    # Add some lighter grass blades
    for _ in range(12):
        x, y = random.randint(0, 15), random.randint(0, 15)
        img.putpixel((x, y), add_noise((100, 180, 30), 15))
    return img

def create_grass_side_texture():
    """Grass side - green top, dirt bottom."""
    img = Image.new('RGB', (TILE_SIZE, TILE_SIZE))
    grass_color = (76, 153, 0)
    dirt_color = (139, 90, 43)

    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            if y < 4:  # Top grass layer
                img.putpixel((x, y), add_noise(grass_color, 20))
            elif y < 6:  # Transition
                # Mix grass and dirt
                if random.random() < 0.5:
                    img.putpixel((x, y), add_noise(grass_color, 20))
                else:
                    img.putpixel((x, y), add_noise(dirt_color, 20))
            else:  # Dirt
                img.putpixel((x, y), add_noise(dirt_color, 20))
    return img

def create_ore_texture(base_color, ore_color, ore_count=8):
    """Stone with ore spots."""
    img = create_stone_texture()

    # Add ore spots (2x2 clusters)
    for _ in range(ore_count):
        cx, cy = random.randint(1, 14), random.randint(1, 14)
        for dx in range(2):
            for dy in range(2):
                x, y = cx + dx, cy + dy
                if 0 <= x < 16 and 0 <= y < 16:
                    img.putpixel((x, y), add_noise(ore_color, 15))
    return img

def create_dirt_texture():
    """Brown dirt."""
    img = Image.new('RGB', (TILE_SIZE, TILE_SIZE))
    base_color = (139, 90, 43)
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            img.putpixel((x, y), add_noise(base_color, 25))
    # Add some darker spots
    for _ in range(10):
        x, y = random.randint(0, 15), random.randint(0, 15)
        img.putpixel((x, y), add_noise((100, 65, 30), 15))
    return img

def create_sand_texture():
    """Yellow sand."""
    img = Image.new('RGB', (TILE_SIZE, TILE_SIZE))
    base_color = (210, 180, 140)
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            img.putpixel((x, y), add_noise(base_color, 20))
    # Add some lighter spots
    for _ in range(8):
        x, y = random.randint(0, 15), random.randint(0, 15)
        img.putpixel((x, y), add_noise((230, 200, 160), 15))
    return img

def create_texture(name):
    """Create texture by name."""
    random.seed(hash(name) % 2**32)  # Deterministic per texture

    if name == "stone":
        return create_stone_texture()
    elif name == "grass_top":
        return create_grass_top_texture()
    elif name == "grass_side":
        return create_grass_side_texture()
    elif name == "iron_ore":
        return create_ore_texture((128, 128, 128), (180, 120, 80), ore_count=10)
    elif name == "copper_ore":
        return create_ore_texture((128, 128, 128), (180, 90, 60), ore_count=10)
    elif name == "coal":
        return create_ore_texture((128, 128, 128), (30, 30, 30), ore_count=12)
    elif name == "dirt":
        return create_dirt_texture()
    elif name == "sand":
        return create_sand_texture()
    else:
        # Default: gray
        img = Image.new('RGB', (TILE_SIZE, TILE_SIZE), (128, 128, 128))
        return img

def create_atlas():
    """Create the full texture atlas."""
    atlas = Image.new('RGB', (ATLAS_SIZE * TILE_SIZE, ATLAS_SIZE * TILE_SIZE), (255, 0, 255))

    for idx, name in TEXTURES.items():
        texture = create_texture(name)
        tx = (idx % ATLAS_SIZE) * TILE_SIZE
        ty = (idx // ATLAS_SIZE) * TILE_SIZE
        atlas.paste(texture, (tx, ty))
        print(f"Created texture {idx}: {name} at ({tx}, {ty})")

    return atlas


def create_array_texture():
    """Create vertically stacked texture array for 2D array texture.

    Output: 16x128 image (8 layers of 16x16 stacked vertically)
    Each 16x16 slice becomes one layer in the texture array.
    """
    num_layers = len(TEXTURES)
    array_img = Image.new('RGBA', (TILE_SIZE, TILE_SIZE * num_layers), (255, 0, 255, 255))

    for idx, name in TEXTURES.items():
        texture = create_texture(name)
        # Convert to RGBA
        texture_rgba = texture.convert('RGBA')
        # Stack vertically: layer 0 at top, layer N at bottom
        ty = idx * TILE_SIZE
        array_img.paste(texture_rgba, (0, ty))
        print(f"Array layer {idx}: {name} at y={ty}")

    return array_img, num_layers

def main():
    import os

    # Output path
    output_dir = os.path.join(os.path.dirname(__file__), "..", "assets", "textures")

    # Create legacy atlas (for backwards compatibility)
    print("=== Creating legacy atlas ===")
    atlas = create_atlas()
    output_path = os.path.join(output_dir, "block_atlas_default.png")
    atlas.save(output_path)
    print(f"Saved atlas to: {output_path}")
    print(f"Atlas size: {atlas.size[0]}x{atlas.size[1]}")

    # Create array texture (for VoxelMaterial)
    print("\n=== Creating array texture ===")
    array_img, num_layers = create_array_texture()
    array_output_path = os.path.join(output_dir, "block_textures_array.png")
    array_img.save(array_output_path)
    print(f"\nSaved array texture to: {array_output_path}")
    print(f"Array texture size: {array_img.size[0]}x{array_img.size[1]} ({num_layers} layers)")

if __name__ == "__main__":
    main()
