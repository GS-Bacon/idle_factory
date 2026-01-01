#!/usr/bin/env python3
"""Generate a default block texture atlas with placeholder labels."""

import struct
import zlib

# Atlas configuration
TILE_SIZE = 16
GRID_SIZE = 8
ATLAS_SIZE = TILE_SIZE * GRID_SIZE  # 128x128 (smaller for placeholder)

# Block definitions: (id, name, base_color_rgb)
BLOCKS = [
    (0, "GRASS\nTOP", (76, 175, 80)),    # Green
    (1, "GRASS\nSIDE", (76, 140, 80)),   # Green-brown
    (2, "DIRT", (139, 105, 20)),         # Brown
    (3, "STONE", (128, 128, 128)),       # Gray
    (4, "IRON", (192, 192, 200)),        # Silver
    (5, "COPPER", (224, 128, 80)),       # Orange
    (6, "COAL", (48, 48, 48)),           # Dark gray
    (7, "BEDROCK", (32, 32, 32)),        # Very dark
    (8, "SAND", (224, 192, 128)),        # Yellow-ish
]

# Simple 3x5 font for labels (each char is 3 wide, 5 tall)
FONT = {
    'A': ["###", "# #", "###", "# #", "# #"],
    'B': ["## ", "# #", "## ", "# #", "## "],
    'C': ["###", "#  ", "#  ", "#  ", "###"],
    'D': ["## ", "# #", "# #", "# #", "## "],
    'E': ["###", "#  ", "## ", "#  ", "###"],
    'G': ["###", "#  ", "# #", "# #", "###"],
    'I': ["###", " # ", " # ", " # ", "###"],
    'K': ["# #", "## ", "#  ", "## ", "# #"],
    'L': ["#  ", "#  ", "#  ", "#  ", "###"],
    'N': ["# #", "## ", "# #", "# #", "# #"],
    'O': ["###", "# #", "# #", "# #", "###"],
    'P': ["###", "# #", "###", "#  ", "#  "],
    'R': ["## ", "# #", "## ", "# #", "# #"],
    'S': ["###", "#  ", "###", "  #", "###"],
    'T': ["###", " # ", " # ", " # ", " # "],
    'U': ["# #", "# #", "# #", "# #", "###"],
    ' ': ["   ", "   ", "   ", "   ", "   "],
    '\n': None,  # Line break marker
}

def draw_text(pixels, x, y, text, color, tile_size):
    """Draw text onto the pixel array."""
    lines = text.split('\n')
    line_y = y
    for line in lines:
        char_x = x
        for char in line:
            if char in FONT and FONT[char]:
                pattern = FONT[char]
                for py, row in enumerate(pattern):
                    for px, c in enumerate(row):
                        if c == '#':
                            px_x = char_x + px
                            py_y = line_y + py
                            if 0 <= px_x < tile_size and 0 <= py_y < tile_size:
                                pixels[py_y * tile_size + px_x] = color
            char_x += 4  # 3 pixels + 1 space
        line_y += 6  # 5 pixels + 1 space

def generate_tile(name, base_color):
    """Generate a single tile with label."""
    pixels = [base_color] * (TILE_SIZE * TILE_SIZE)

    # Add border (darker)
    border_color = tuple(max(0, c - 40) for c in base_color)
    for i in range(TILE_SIZE):
        pixels[i] = border_color  # Top
        pixels[(TILE_SIZE - 1) * TILE_SIZE + i] = border_color  # Bottom
        pixels[i * TILE_SIZE] = border_color  # Left
        pixels[i * TILE_SIZE + TILE_SIZE - 1] = border_color  # Right

    # Add text (white or black depending on brightness)
    brightness = sum(base_color) / 3
    text_color = (0, 0, 0) if brightness > 128 else (255, 255, 255)
    draw_text(pixels, 1, 2, name, text_color, TILE_SIZE)

    return pixels

def write_png(filename, width, height, pixels):
    """Write pixels to PNG file."""
    def png_chunk(chunk_type, data):
        chunk_len = struct.pack('>I', len(data))
        chunk_crc = struct.pack('>I', zlib.crc32(chunk_type + data) & 0xffffffff)
        return chunk_len + chunk_type + data + chunk_crc

    signature = b'\x89PNG\r\n\x1a\n'
    ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0)
    ihdr = png_chunk(b'IHDR', ihdr_data)

    raw_data = b''
    for y in range(height):
        raw_data += b'\x00'
        for x in range(width):
            r, g, b = pixels[y * width + x]
            raw_data += bytes([r, g, b])

    compressed = zlib.compress(raw_data)
    idat = png_chunk(b'IDAT', compressed)
    iend = png_chunk(b'IEND', b'')

    with open(filename, 'wb') as f:
        f.write(signature + ihdr + idat + iend)

def main():
    # Create atlas pixels
    pixels = [(64, 64, 64)] * (ATLAS_SIZE * ATLAS_SIZE)  # Default gray

    for block_id, name, color in BLOCKS:
        col = block_id % GRID_SIZE
        row = block_id // GRID_SIZE

        tile = generate_tile(name, color)

        # Copy tile to atlas
        for ty in range(TILE_SIZE):
            for tx in range(TILE_SIZE):
                ax = col * TILE_SIZE + tx
                ay = row * TILE_SIZE + ty
                pixels[ay * ATLAS_SIZE + ax] = tile[ty * TILE_SIZE + tx]

    # Write to file
    output_path = "assets/textures/block_atlas_default.png"
    write_png(output_path, ATLAS_SIZE, ATLAS_SIZE, pixels)
    print(f"Generated: {output_path} ({ATLAS_SIZE}x{ATLAS_SIZE})")

if __name__ == "__main__":
    main()
