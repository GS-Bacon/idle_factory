#!/usr/bin/env python3
"""
Voxel Model Generator - .vox file generator for MagicaVoxel

AIが座標とマテリアルを指定するだけで、正確なボクセルモデルを生成できる。
比率ミスやプロシージャル生成のバグを防ぐ。

使い方:
    from voxel_generator import VoxelModel, PALETTE

    model = VoxelModel(16, 16, 16)
    model.fill_box(0, 0, 0, 15, 1, 15, PALETTE["frame"])
    model.fill_box(2, 2, 0, 13, 2, 15, PALETTE["belt"])
    model.save("conveyor_straight.vox")
"""

import struct
from pathlib import Path
from typing import Dict, Tuple, List, Optional

# ゲームのスタイルに合わせたカラーパレット (RGBA)
PALETTE = {
    # 基本マテリアル
    "iron": (115, 115, 120, 255),       # 鉄 - ダークグレー
    "copper": (184, 115, 51, 255),      # 銅 - オレンジブラウン
    "brass": (201, 163, 38, 255),       # 真鍮 - ゴールド
    "dark_steel": (46, 46, 51, 255),    # ダークスチール
    "wood": (140, 105, 20, 255),        # 木
    "stone": (105, 105, 105, 255),      # 石

    # コンベア用
    "frame": (102, 102, 102, 255),      # フレーム - グレー
    "belt": (68, 68, 68, 255),          # ベルト - ダークグレー
    "roller": (34, 34, 34, 255),        # ローラー - ほぼ黒
    "arrow": (255, 255, 0, 255),        # 矢印 - 黄色

    # 機械用
    "furnace_body": (139, 90, 43, 255), # 精錬炉 - 茶色
    "furnace_glow": (255, 100, 30, 255),# 精錬炉グロー - オレンジ
    "crusher_body": (102, 77, 128, 255),# 粉砕機 - 紫
    "miner_body": (204, 153, 51, 255),  # 採掘機 - ゴールド

    # アクセント
    "danger": (204, 51, 51, 255),       # 危険 - 赤
    "warning": (204, 170, 51, 255),     # 警告 - 黄
    "power": (51, 102, 204, 255),       # 電力 - 青
    "active": (51, 204, 102, 255),      # アクティブ - 緑
}

# パレットインデックスマッピング (1-255, 0は透明)
PALETTE_INDEX = {name: i + 1 for i, name in enumerate(PALETTE.keys())}


class VoxelModel:
    """ボクセルモデルを構築し、.voxファイルに出力するクラス"""

    def __init__(self, size_x: int = 16, size_y: int = 16, size_z: int = 16):
        """
        Args:
            size_x: X軸サイズ (1-256)
            size_y: Y軸サイズ (1-256)
            size_z: Z軸サイズ (1-256, 重力方向)
        """
        self.size_x = min(max(size_x, 1), 256)
        self.size_y = min(max(size_y, 1), 256)
        self.size_z = min(max(size_z, 1), 256)
        self.voxels: Dict[Tuple[int, int, int], int] = {}

    def set_voxel(self, x: int, y: int, z: int, color_index: int) -> None:
        """単一ボクセルを設置"""
        if 0 <= x < self.size_x and 0 <= y < self.size_y and 0 <= z < self.size_z:
            if 1 <= color_index <= 255:
                self.voxels[(x, y, z)] = color_index

    def set_voxel_named(self, x: int, y: int, z: int, material: str) -> None:
        """マテリアル名でボクセルを設置"""
        if material in PALETTE_INDEX:
            self.set_voxel(x, y, z, PALETTE_INDEX[material])

    def remove_voxel(self, x: int, y: int, z: int) -> None:
        """ボクセルを削除"""
        self.voxels.pop((x, y, z), None)

    def fill_box(self, x1: int, y1: int, z1: int,
                 x2: int, y2: int, z2: int,
                 color: Tuple[int, int, int, int] | int | str) -> None:
        """直方体を塗りつぶす

        Args:
            x1, y1, z1: 開始座標
            x2, y2, z2: 終了座標 (含む)
            color: RGBA タプル、パレットインデックス、またはマテリアル名
        """
        color_index = self._resolve_color(color)
        for x in range(min(x1, x2), max(x1, x2) + 1):
            for y in range(min(y1, y2), max(y1, y2) + 1):
                for z in range(min(z1, z2), max(z1, z2) + 1):
                    self.set_voxel(x, y, z, color_index)

    def fill_box_hollow(self, x1: int, y1: int, z1: int,
                        x2: int, y2: int, z2: int,
                        color: Tuple[int, int, int, int] | int | str,
                        thickness: int = 1) -> None:
        """中空の直方体を作成"""
        color_index = self._resolve_color(color)
        for x in range(min(x1, x2), max(x1, x2) + 1):
            for y in range(min(y1, y2), max(y1, y2) + 1):
                for z in range(min(z1, z2), max(z1, z2) + 1):
                    # 外側からthickness以内なら塗る
                    if (x - x1 < thickness or x2 - x < thickness or
                        y - y1 < thickness or y2 - y < thickness or
                        z - z1 < thickness or z2 - z < thickness):
                        self.set_voxel(x, y, z, color_index)

    def fill_cylinder(self, cx: int, cy: int, z1: int, z2: int,
                      radius: int, color: Tuple[int, int, int, int] | int | str) -> None:
        """円柱を塗りつぶす (Z軸方向)"""
        color_index = self._resolve_color(color)
        r2 = radius * radius
        for x in range(cx - radius, cx + radius + 1):
            for y in range(cy - radius, cy + radius + 1):
                if (x - cx) ** 2 + (y - cy) ** 2 <= r2:
                    for z in range(min(z1, z2), max(z1, z2) + 1):
                        self.set_voxel(x, y, z, color_index)

    def draw_line(self, x1: int, y1: int, z1: int,
                  x2: int, y2: int, z2: int,
                  color: Tuple[int, int, int, int] | int | str) -> None:
        """3D線を描画 (Bresenham)"""
        color_index = self._resolve_color(color)
        dx, dy, dz = abs(x2 - x1), abs(y2 - y1), abs(z2 - z1)
        sx = 1 if x1 < x2 else -1
        sy = 1 if y1 < y2 else -1
        sz = 1 if z1 < z2 else -1

        if dx >= dy and dx >= dz:
            err_y, err_z = 2 * dy - dx, 2 * dz - dx
            while x1 != x2:
                self.set_voxel(x1, y1, z1, color_index)
                if err_y > 0:
                    y1 += sy
                    err_y -= 2 * dx
                if err_z > 0:
                    z1 += sz
                    err_z -= 2 * dx
                err_y += 2 * dy
                err_z += 2 * dz
                x1 += sx
        elif dy >= dx and dy >= dz:
            err_x, err_z = 2 * dx - dy, 2 * dz - dy
            while y1 != y2:
                self.set_voxel(x1, y1, z1, color_index)
                if err_x > 0:
                    x1 += sx
                    err_x -= 2 * dy
                if err_z > 0:
                    z1 += sz
                    err_z -= 2 * dy
                err_x += 2 * dx
                err_z += 2 * dz
                y1 += sy
        else:
            err_x, err_y = 2 * dx - dz, 2 * dy - dz
            while z1 != z2:
                self.set_voxel(x1, y1, z1, color_index)
                if err_x > 0:
                    x1 += sx
                    err_x -= 2 * dz
                if err_y > 0:
                    y1 += sy
                    err_y -= 2 * dz
                err_x += 2 * dx
                err_y += 2 * dy
                z1 += sz
        self.set_voxel(x2, y2, z2, color_index)

    def draw_arrow(self, x: int, y: int, z: int,
                   direction: str = "+y",
                   color: Tuple[int, int, int, int] | int | str = "arrow") -> None:
        """矢印を描画

        Args:
            x, y, z: 矢印の先端位置
            direction: "+x", "-x", "+y", "-y", "+z", "-z"
            color: 色
        """
        color_index = self._resolve_color(color)

        # 矢印の長さと幅
        length = 4
        width = 2

        if direction == "+y":
            # 軸
            for i in range(length):
                self.set_voxel(x, y - i, z, color_index)
            # 矢尻
            for i in range(1, width + 1):
                self.set_voxel(x - i, y - i, z, color_index)
                self.set_voxel(x + i, y - i, z, color_index)
        elif direction == "-y":
            for i in range(length):
                self.set_voxel(x, y + i, z, color_index)
            for i in range(1, width + 1):
                self.set_voxel(x - i, y + i, z, color_index)
                self.set_voxel(x + i, y + i, z, color_index)
        elif direction == "+x":
            for i in range(length):
                self.set_voxel(x - i, y, z, color_index)
            for i in range(1, width + 1):
                self.set_voxel(x - i, y - i, z, color_index)
                self.set_voxel(x - i, y + i, z, color_index)
        elif direction == "-x":
            for i in range(length):
                self.set_voxel(x + i, y, z, color_index)
            for i in range(1, width + 1):
                self.set_voxel(x + i, y - i, z, color_index)
                self.set_voxel(x + i, y + i, z, color_index)
        elif direction == "+z":
            for i in range(length):
                self.set_voxel(x, y, z - i, color_index)
            for i in range(1, width + 1):
                self.set_voxel(x - i, y, z - i, color_index)
                self.set_voxel(x + i, y, z - i, color_index)
        elif direction == "-z":
            for i in range(length):
                self.set_voxel(x, y, z + i, color_index)
            for i in range(1, width + 1):
                self.set_voxel(x - i, y, z + i, color_index)
                self.set_voxel(x + i, y, z + i, color_index)

    def _resolve_color(self, color) -> int:
        """色をパレットインデックスに変換"""
        if isinstance(color, int):
            return max(1, min(255, color))
        elif isinstance(color, str):
            return PALETTE_INDEX.get(color, 1)
        elif isinstance(color, tuple):
            # RGBAタプルの場合、パレットから最も近い色を探す
            return self._find_closest_palette_index(color)
        return 1

    def _find_closest_palette_index(self, rgba: Tuple[int, int, int, int]) -> int:
        """RGBAに最も近いパレットインデックスを返す"""
        min_dist = float('inf')
        closest_index = 1
        for name, color in PALETTE.items():
            dist = sum((a - b) ** 2 for a, b in zip(rgba[:3], color[:3]))
            if dist < min_dist:
                min_dist = dist
                closest_index = PALETTE_INDEX[name]
        return closest_index

    def _build_palette(self) -> bytes:
        """256色パレットをビルド"""
        palette_data = bytearray()
        for i in range(256):
            if i < len(PALETTE):
                name = list(PALETTE.keys())[i]
                r, g, b, a = PALETTE[name]
                palette_data.extend([r, g, b, a])
            else:
                # 未使用色はグレー
                gray = (i * 255) // 256
                palette_data.extend([gray, gray, gray, 255])
        return bytes(palette_data)

    def _make_chunk(self, chunk_id: bytes, content: bytes, children: bytes = b'') -> bytes:
        """チャンクを作成"""
        return (chunk_id +
                struct.pack('<I', len(content)) +
                struct.pack('<I', len(children)) +
                content + children)

    def save(self, filename: str) -> None:
        """VOXファイルとして保存"""
        path = Path(filename)
        path.parent.mkdir(parents=True, exist_ok=True)

        # SIZE chunk
        size_content = struct.pack('<III', self.size_x, self.size_y, self.size_z)
        size_chunk = self._make_chunk(b'SIZE', size_content)

        # XYZI chunk
        voxel_list = list(self.voxels.items())
        xyzi_content = struct.pack('<I', len(voxel_list))
        for (x, y, z), color_index in voxel_list:
            xyzi_content += struct.pack('<BBBB', x, y, z, color_index)
        xyzi_chunk = self._make_chunk(b'XYZI', xyzi_content)

        # RGBA chunk
        rgba_chunk = self._make_chunk(b'RGBA', self._build_palette())

        # MAIN chunk
        children = size_chunk + xyzi_chunk + rgba_chunk
        main_chunk = self._make_chunk(b'MAIN', b'', children)

        # Write file
        with open(path, 'wb') as f:
            f.write(b'VOX ')
            f.write(struct.pack('<I', 150))  # Version
            f.write(main_chunk)

        print(f"Saved: {path} ({len(self.voxels)} voxels)")

    def get_stats(self) -> Dict:
        """モデル統計を返す"""
        return {
            "size": (self.size_x, self.size_y, self.size_z),
            "voxel_count": len(self.voxels),
            "volume_ratio": len(self.voxels) / (self.size_x * self.size_y * self.size_z)
        }


# =============================================================================
# プリセットモデル生成関数
# =============================================================================

def create_conveyor_straight() -> VoxelModel:
    """直進コンベアを生成 - Kenney風シームレスデザイン

    特徴:
    - 先端にストッパーなし（連結時に一直線）
    - 山形パターン（^）で進行方向を示す（ベルト面と同じ高さでフラット）
    - 低いサイドレールで連結感
    """
    model = VoxelModel(16, 16, 2)  # 高さ2に変更（より低く）

    # ベルト面（端まで伸ばす）
    model.fill_box(1, 0, 0, 14, 15, 0, "belt")

    # サイドレール（低く薄く）
    model.fill_box(0, 0, 0, 0, 15, 1, "frame")   # 左レール
    model.fill_box(15, 0, 0, 15, 15, 1, "frame") # 右レール

    # 山形パターン（^）を複数配置 - 進行方向（+Y）を示す
    # Z=0でベルト面と同じ高さに配置（フラットな模様として見える）
    def draw_chevron(y_center: int):
        """中央に山形を描画"""
        # 山形の頂点から両側に広がる
        model.set_voxel_named(7, y_center, 0, "arrow")
        model.set_voxel_named(8, y_center, 0, "arrow")
        model.set_voxel_named(6, y_center - 1, 0, "arrow")
        model.set_voxel_named(9, y_center - 1, 0, "arrow")
        model.set_voxel_named(5, y_center - 2, 0, "arrow")
        model.set_voxel_named(10, y_center - 2, 0, "arrow")
        model.set_voxel_named(4, y_center - 3, 0, "arrow")
        model.set_voxel_named(11, y_center - 3, 0, "arrow")

    # 3つの山形を等間隔に配置
    draw_chevron(4)
    draw_chevron(9)
    draw_chevron(14)

    return model


def create_conveyor_corner_left() -> VoxelModel:
    """左折コンベアを生成（後ろから入力、左へ出力）- Kenney風シームレス

    入力: -Y方向から
    出力: -X方向へ

    矢印はL字カーブで連続的に描画（接続時に繋がって見える）
    """
    model = VoxelModel(16, 16, 2)

    # L字ベルト面
    model.fill_box(1, 0, 0, 14, 14, 0, "belt")   # 入力側（縦）
    model.fill_box(0, 1, 0, 14, 14, 0, "belt")   # 出力側（横）

    # 外側レール（L字の外角）
    model.fill_box(15, 0, 0, 15, 15, 1, "frame") # 入力右
    model.fill_box(0, 15, 0, 15, 15, 1, "frame") # 外角

    # 内側レール（L字の内角）- コーナー部分は開ける
    model.fill_box(0, 0, 0, 0, 0, 1, "frame")    # 入力左端

    # L字カーブ矢印 - 入口から曲がって出口へ連続
    # 入力側から来る矢印（下から上へ）
    for y in range(0, 5):
        model.set_voxel_named(7, y, 0, "arrow")
        model.set_voxel_named(8, y, 0, "arrow")

    # カーブ部分（対角線）
    model.set_voxel_named(7, 5, 0, "arrow")
    model.set_voxel_named(8, 5, 0, "arrow")
    model.set_voxel_named(6, 6, 0, "arrow")
    model.set_voxel_named(7, 6, 0, "arrow")
    model.set_voxel_named(5, 7, 0, "arrow")
    model.set_voxel_named(6, 7, 0, "arrow")
    model.set_voxel_named(5, 8, 0, "arrow")

    # 出力側へ向かう矢印（右から左へ）
    for x in range(0, 5):
        model.set_voxel_named(x, 7, 0, "arrow")
        model.set_voxel_named(x, 8, 0, "arrow")

    return model


def create_conveyor_corner_right() -> VoxelModel:
    """右折コンベアを生成（後ろから入力、右へ出力）- Kenney風シームレス

    入力: -Y方向から
    出力: +X方向へ

    矢印はL字カーブで連続的に描画（接続時に繋がって見える）
    """
    model = VoxelModel(16, 16, 2)

    # L字ベルト面
    model.fill_box(1, 0, 0, 14, 14, 0, "belt")   # 入力側（縦）
    model.fill_box(1, 1, 0, 15, 14, 0, "belt")   # 出力側（横）

    # 外側レール（L字の外角）
    model.fill_box(0, 0, 0, 0, 15, 1, "frame")   # 入力左
    model.fill_box(0, 15, 0, 15, 15, 1, "frame") # 外角

    # 内側レール - コーナー部分は開ける
    model.fill_box(15, 0, 0, 15, 0, 1, "frame")  # 入力右端

    # L字カーブ矢印 - 入口から曲がって出口へ連続
    # 入力側から来る矢印（下から上へ）
    for y in range(0, 5):
        model.set_voxel_named(7, y, 0, "arrow")
        model.set_voxel_named(8, y, 0, "arrow")

    # カーブ部分（対角線）
    model.set_voxel_named(7, 5, 0, "arrow")
    model.set_voxel_named(8, 5, 0, "arrow")
    model.set_voxel_named(8, 6, 0, "arrow")
    model.set_voxel_named(9, 6, 0, "arrow")
    model.set_voxel_named(9, 7, 0, "arrow")
    model.set_voxel_named(10, 7, 0, "arrow")
    model.set_voxel_named(10, 8, 0, "arrow")

    # 出力側へ向かう矢印（左から右へ）
    for x in range(11, 16):
        model.set_voxel_named(x, 7, 0, "arrow")
        model.set_voxel_named(x, 8, 0, "arrow")

    return model


def create_conveyor_t_junction() -> VoxelModel:
    """T字合流コンベアを生成（左右から入力、前方へ出力）- Kenney風シームレス

    入力: -X方向と+X方向から
    出力: +Y方向へ
    """
    model = VoxelModel(16, 16, 2)

    # T字ベルト面
    model.fill_box(1, 1, 0, 14, 15, 0, "belt")   # 縦（出力側）
    model.fill_box(0, 1, 0, 15, 14, 0, "belt")   # 横（入力側）

    # 出力方向の山形 - Z=0でフラット
    model.set_voxel_named(7, 13, 0, "arrow")
    model.set_voxel_named(8, 13, 0, "arrow")
    model.set_voxel_named(6, 12, 0, "arrow")
    model.set_voxel_named(9, 12, 0, "arrow")
    model.set_voxel_named(5, 11, 0, "arrow")
    model.set_voxel_named(10, 11, 0, "arrow")

    return model


def create_conveyor_splitter() -> VoxelModel:
    """十字スプリッターを生成（後ろから入力、前・左・右へ出力）- Kenney風シームレス

    入力: -Y方向から
    出力: +Y, -X, +X の3方向へ
    """
    model = VoxelModel(16, 16, 2)

    # 十字ベルト面
    model.fill_box(1, 0, 0, 14, 15, 0, "belt")   # 縦
    model.fill_box(0, 1, 0, 15, 14, 0, "belt")   # 横

    # 4隅のレール端点のみ
    model.fill_box(0, 0, 0, 0, 0, 1, "frame")    # 入力左
    model.fill_box(15, 0, 0, 15, 0, 1, "frame")  # 入力右
    model.fill_box(0, 15, 0, 0, 15, 1, "frame")  # 出力前左
    model.fill_box(15, 15, 0, 15, 15, 1, "frame") # 出力前右

    # 中央分岐マーカー - Z=0でフラット
    model.fill_box(7, 7, 0, 8, 8, 0, "warning")
    model.set_voxel_named(7, 8, 0, "arrow")
    model.set_voxel_named(8, 8, 0, "arrow")

    return model


def create_miner() -> VoxelModel:
    """採掘機モデルを生成 (16x16x16)"""
    model = VoxelModel(16, 16, 16)

    # ベース（底面）
    model.fill_box(1, 1, 0, 14, 14, 1, "dark_steel")

    # メインボディ
    model.fill_box(2, 2, 2, 13, 13, 10, "miner_body")

    # 上部ハウジング
    model.fill_box(3, 3, 10, 12, 12, 13, "iron")

    # ドリルヘッド（下向き、中央）
    model.fill_cylinder(7, 7, 0, 2, 3, "dark_steel")

    # 排気口（上部）
    model.fill_box(6, 6, 13, 9, 9, 15, "iron")

    # 出力口（前面）
    model.fill_box(5, 14, 4, 10, 15, 7, "frame")

    # アクセントライン
    model.fill_box(2, 2, 5, 13, 2, 6, "warning")
    model.fill_box(2, 13, 5, 13, 13, 6, "warning")

    # 稼働インジケータ
    model.fill_box(6, 2, 8, 9, 2, 9, "active")

    return model


def create_furnace() -> VoxelModel:
    """精錬炉モデルを生成 (16x16x16)"""
    model = VoxelModel(16, 16, 16)

    # ベース（底面）
    model.fill_box(1, 1, 0, 14, 14, 1, "dark_steel")

    # 外壁（中空）
    model.fill_box_hollow(1, 1, 2, 14, 14, 14, "furnace_body", 2)

    # 炉口（前面）- 開口部
    model.fill_box(4, 0, 3, 11, 2, 9, "furnace_glow")

    # 煙突（上部）
    model.fill_box(6, 6, 14, 9, 9, 15, "iron")

    # 入力口（後面上部）
    model.fill_box(5, 14, 10, 10, 15, 13, "frame")

    # 出力口（前面下部）
    model.fill_box(5, 0, 2, 10, 1, 4, "frame")

    # 温度計/インジケータ（側面）
    model.fill_box(0, 6, 6, 0, 9, 10, "danger")

    # 装飾リベット
    model.fill_box(1, 1, 14, 2, 2, 14, "brass")
    model.fill_box(13, 1, 14, 14, 2, 14, "brass")
    model.fill_box(1, 13, 14, 2, 14, 14, "brass")
    model.fill_box(13, 13, 14, 14, 14, 14, "brass")

    return model


def create_crusher() -> VoxelModel:
    """粉砕機モデルを生成 (16x16x16)"""
    model = VoxelModel(16, 16, 16)

    # ベース（底面）
    model.fill_box(1, 1, 0, 14, 14, 1, "dark_steel")

    # メインボディ（紫）
    model.fill_box(2, 2, 2, 13, 13, 12, "crusher_body")

    # 入力ホッパー（上部）
    model.fill_box(4, 4, 12, 11, 11, 15, "iron")
    model.fill_box(5, 5, 15, 10, 10, 15, "dark_steel")  # 入力口

    # 出力口（前面下部）
    model.fill_box(5, 14, 2, 10, 15, 5, "frame")

    # 粉砕ローラー（側面から見える）
    model.fill_box(2, 5, 6, 2, 10, 9, "iron")
    model.fill_box(13, 5, 6, 13, 10, 9, "iron")

    # モーターハウジング（後部）
    model.fill_box(4, 1, 5, 11, 1, 10, "iron")

    # アクセントライン
    model.fill_box(2, 2, 7, 13, 2, 8, "power")
    model.fill_box(2, 13, 7, 13, 13, 8, "power")

    # 稼働インジケータ
    model.fill_box(6, 2, 10, 9, 2, 11, "active")

    return model


def create_assembler() -> VoxelModel:
    """組立機モデルを生成 (16x16x16)

    複数の入力を組み合わせて製品を作る機械。
    緑をベースカラーとし、複数の入力口を持つデザイン。
    """
    model = VoxelModel(16, 16, 16)

    # ベース（底面）
    model.fill_box(1, 1, 0, 14, 14, 1, "dark_steel")

    # メインボディ（緑系）
    model.fill_box(2, 2, 2, 13, 13, 11, "active")

    # 上部カバー（鉄）
    model.fill_box(2, 2, 11, 13, 13, 12, "iron")

    # 中央の作業台（上面）
    model.fill_box(4, 4, 12, 11, 11, 13, "brass")

    # ロボットアーム（中央から突出）
    model.fill_box(7, 7, 13, 8, 8, 15, "iron")
    model.fill_box(6, 6, 14, 9, 9, 14, "dark_steel")  # アーム先端

    # 入力口1（左側面）
    model.fill_box(0, 5, 4, 1, 10, 7, "frame")

    # 入力口2（右側面）
    model.fill_box(14, 5, 4, 15, 10, 7, "frame")

    # 出力口（前面）
    model.fill_box(5, 14, 3, 10, 15, 6, "frame")

    # アクセントライン（青）
    model.fill_box(2, 2, 6, 13, 2, 7, "power")
    model.fill_box(2, 13, 6, 13, 13, 7, "power")

    # 稼働インジケータ（前面）
    model.fill_box(6, 2, 9, 9, 2, 10, "warning")

    return model


def create_storage() -> VoxelModel:
    """倉庫モデルを生成 (16x16x16)

    アイテムを保管するコンテナ。
    木箱風のデザインで、前面に扉。
    """
    model = VoxelModel(16, 16, 16)

    # ベース（底面）- 木製パレット
    model.fill_box(0, 0, 0, 15, 15, 1, "wood")

    # 本体（木箱）
    model.fill_box(1, 1, 1, 14, 14, 13, "wood")

    # 上面（蓋）
    model.fill_box(1, 1, 13, 14, 14, 14, "wood")

    # フレーム補強（鉄）- 角
    model.fill_box(1, 1, 1, 2, 2, 13, "iron")
    model.fill_box(13, 1, 1, 14, 2, 13, "iron")
    model.fill_box(1, 13, 1, 2, 14, 13, "iron")
    model.fill_box(13, 13, 1, 14, 14, 13, "iron")

    # 横補強
    model.fill_box(1, 1, 6, 14, 1, 7, "iron")
    model.fill_box(1, 14, 6, 14, 14, 7, "iron")

    # 前面扉（ダークスチール）
    model.fill_box(4, 14, 2, 11, 14, 11, "dark_steel")

    # 扉の取っ手（真鍮）
    model.fill_box(6, 14, 6, 6, 15, 7, "brass")
    model.fill_box(9, 14, 6, 9, 15, 7, "brass")

    # 入力口（上面中央）
    model.fill_box(5, 5, 14, 10, 10, 15, "frame")

    # 出力口（前面下部）
    model.fill_box(5, 15, 2, 10, 15, 4, "frame")

    # 容量インジケータ（側面）
    model.fill_box(0, 5, 3, 0, 10, 11, "active")

    return model


def create_generator() -> VoxelModel:
    """発電機モデルを生成 (16x16x16)

    燃料を燃やして電力を生成する機械。
    煙突とタービンを持つ工業的なデザイン。
    """
    model = VoxelModel(16, 16, 16)

    # ベース（底面）- 頑丈な土台
    model.fill_box(0, 0, 0, 15, 15, 2, "dark_steel")

    # メインボディ（鉄）
    model.fill_box(2, 2, 2, 13, 13, 10, "iron")

    # 燃焼室（前面に見える炎）
    model.fill_box(4, 1, 3, 11, 2, 7, "furnace_glow")

    # 煙突（後部）
    model.fill_box(4, 2, 10, 7, 5, 15, "iron")
    model.fill_box(5, 3, 15, 6, 4, 15, "dark_steel")  # 煙突口

    # タービン部（右側面）
    model.fill_cylinder(13, 7, 4, 9, 3, "copper")
    model.fill_box(14, 5, 5, 15, 9, 8, "iron")

    # 燃料入力口（上面）
    model.fill_box(9, 9, 10, 12, 12, 11, "frame")

    # 電力出力コネクタ（側面）
    model.fill_box(0, 5, 5, 1, 10, 8, "power")
    model.fill_box(0, 6, 6, 0, 9, 7, "warning")  # 警告ライト

    # 排気グリル（後面）
    model.fill_box(8, 14, 4, 12, 15, 8, "dark_steel")

    # 稼働インジケータ（前面）
    model.fill_box(6, 2, 8, 9, 2, 9, "active")

    # 危険マーク（側面）
    model.fill_box(15, 6, 6, 15, 9, 7, "danger")

    return model


def create_inserter() -> VoxelModel:
    """インサーター（アイテム移動アーム）を生成 (16x16x8)

    コンベアや機械間でアイテムを移動するロボットアーム。
    Factorio風の回転アーム。
    """
    model = VoxelModel(16, 16, 8)

    # ベース（底面）
    model.fill_box(4, 4, 0, 11, 11, 1, "dark_steel")

    # 回転台座
    model.fill_cylinder(7, 7, 1, 3, 3, "iron")

    # アーム本体（伸びた状態）
    model.fill_box(6, 7, 3, 8, 14, 4, "iron")  # 水平部分
    model.fill_box(6, 14, 3, 8, 14, 6, "iron")  # 先端（下向き）

    # グリッパー（先端）
    model.fill_box(5, 14, 6, 9, 15, 7, "dark_steel")
    model.fill_box(5, 15, 5, 5, 15, 7, "brass")  # 左爪
    model.fill_box(9, 15, 5, 9, 15, 7, "brass")  # 右爪

    # モーター部（中央上部）
    model.fill_box(6, 6, 4, 9, 9, 6, "copper")

    # 稼働インジケータ
    model.fill_box(7, 4, 2, 8, 4, 2, "active")

    return model


def create_splitter_machine() -> VoxelModel:
    """スプリッター機械を生成 (16x16x10)

    1入力を2出力に分岐する機械。
    コンベアのsplitterとは別に、より高機能な分岐装置。
    """
    model = VoxelModel(16, 16, 10)

    # ベース
    model.fill_box(1, 1, 0, 14, 14, 1, "dark_steel")

    # 本体（三角形っぽい形状）
    model.fill_box(4, 1, 1, 11, 14, 6, "iron")

    # 入力口（後面中央）
    model.fill_box(5, 0, 2, 10, 1, 5, "frame")

    # 出力口1（左前）
    model.fill_box(0, 10, 2, 4, 15, 5, "frame")

    # 出力口2（右前）
    model.fill_box(11, 10, 2, 15, 15, 5, "frame")

    # 分岐ガイド（Y字形状の内部）
    model.fill_box(6, 5, 3, 9, 10, 4, "belt")
    model.fill_box(3, 10, 3, 6, 12, 4, "belt")  # 左分岐
    model.fill_box(9, 10, 3, 12, 12, 4, "belt")  # 右分岐

    # 上部カバー
    model.fill_box(4, 1, 6, 11, 14, 7, "dark_steel")

    # センサー/制御部
    model.fill_box(6, 7, 7, 9, 9, 9, "power")

    # インジケータ
    model.fill_box(7, 2, 5, 8, 3, 5, "warning")

    return model


# =============================================================================
# アイテムモデル
# =============================================================================

def create_item_iron_ore() -> VoxelModel:
    """鉄鉱石アイテムモデル (8x8x8) - ゴツゴツした岩の塊"""
    model = VoxelModel(8, 8, 8)

    # 不規則な岩の形状（ベース石）
    model.fill_box(1, 1, 0, 6, 6, 4, "stone")
    model.fill_box(2, 2, 4, 5, 5, 6, "stone")
    model.fill_box(0, 2, 1, 0, 5, 3, "stone")
    model.fill_box(7, 2, 1, 7, 5, 3, "stone")
    model.fill_box(2, 0, 1, 5, 0, 3, "stone")
    model.fill_box(2, 7, 1, 5, 7, 3, "stone")

    # 鉄鉱脈（キラキラした部分）
    model.set_voxel_named(2, 3, 5, "iron")
    model.set_voxel_named(4, 4, 5, "iron")
    model.set_voxel_named(3, 2, 4, "iron")
    model.set_voxel_named(5, 5, 3, "iron")
    model.set_voxel_named(2, 5, 2, "iron")
    model.set_voxel_named(4, 2, 2, "iron")
    model.set_voxel_named(1, 3, 3, "iron")
    model.set_voxel_named(6, 4, 2, "iron")

    return model


def create_item_copper_ore() -> VoxelModel:
    """銅鉱石アイテムモデル (8x8x8) - 赤みがかった岩"""
    model = VoxelModel(8, 8, 8)

    # 不規則な岩の形状
    model.fill_box(1, 1, 0, 6, 6, 4, "stone")
    model.fill_box(2, 2, 4, 5, 5, 6, "stone")
    model.fill_box(0, 2, 1, 0, 5, 3, "stone")
    model.fill_box(7, 2, 1, 7, 5, 3, "stone")

    # 銅鉱脈（オレンジがかった部分）
    model.set_voxel_named(2, 3, 5, "copper")
    model.set_voxel_named(4, 4, 5, "copper")
    model.set_voxel_named(3, 2, 4, "copper")
    model.set_voxel_named(5, 5, 3, "copper")
    model.set_voxel_named(2, 5, 2, "copper")
    model.set_voxel_named(4, 2, 2, "copper")
    model.set_voxel_named(1, 3, 3, "copper")
    model.set_voxel_named(6, 4, 2, "copper")
    model.set_voxel_named(3, 4, 3, "copper")

    return model


def create_item_coal() -> VoxelModel:
    """石炭アイテムモデル (8x8x8) - 黒い塊"""
    model = VoxelModel(8, 8, 8)

    # 不規則な黒い塊
    model.fill_box(1, 1, 0, 6, 6, 3, "dark_steel")
    model.fill_box(2, 2, 3, 5, 5, 5, "dark_steel")
    model.fill_box(0, 2, 1, 0, 5, 2, "dark_steel")
    model.fill_box(7, 2, 1, 7, 5, 2, "dark_steel")

    # 光沢（少しだけ）
    model.set_voxel_named(3, 3, 5, "frame")
    model.set_voxel_named(4, 4, 4, "frame")

    return model


def create_item_stone() -> VoxelModel:
    """石アイテムモデル (8x8x8) - グレーの岩"""
    model = VoxelModel(8, 8, 8)

    # 丸みのある岩
    model.fill_box(1, 1, 0, 6, 6, 3, "stone")
    model.fill_box(2, 2, 3, 5, 5, 5, "stone")
    model.fill_box(0, 2, 1, 0, 5, 2, "stone")
    model.fill_box(7, 2, 1, 7, 5, 2, "stone")
    model.fill_box(2, 0, 1, 5, 0, 2, "stone")
    model.fill_box(2, 7, 1, 5, 7, 2, "stone")

    return model


def create_item_iron_ingot() -> VoxelModel:
    """鉄インゴットアイテムモデル (8x8x4) - 台形のインゴット"""
    model = VoxelModel(8, 8, 4)

    # インゴット形状（台形）
    model.fill_box(0, 0, 0, 7, 7, 1, "iron")
    model.fill_box(1, 1, 1, 6, 6, 2, "iron")
    model.fill_box(2, 2, 2, 5, 5, 3, "iron")

    # 光沢ハイライト
    model.set_voxel_named(3, 3, 3, "frame")
    model.set_voxel_named(4, 4, 3, "frame")

    return model


def create_item_copper_ingot() -> VoxelModel:
    """銅インゴットアイテムモデル (8x8x4) - 台形のインゴット"""
    model = VoxelModel(8, 8, 4)

    # インゴット形状（台形）
    model.fill_box(0, 0, 0, 7, 7, 1, "copper")
    model.fill_box(1, 1, 1, 6, 6, 2, "copper")
    model.fill_box(2, 2, 2, 5, 5, 3, "copper")

    # 光沢ハイライト
    model.set_voxel_named(3, 3, 3, "brass")
    model.set_voxel_named(4, 4, 3, "brass")

    return model


# =============================================================================
# メイン
# =============================================================================

if __name__ == "__main__":
    import sys

    conveyor_dir = Path("assets/models/machines/conveyor")
    machines_dir = Path("assets/models/machines")
    items_dir = Path("assets/models/items")

    conveyor_models = {
        "straight": create_conveyor_straight,
        "corner_left": create_conveyor_corner_left,
        "corner_right": create_conveyor_corner_right,
        "t_junction": create_conveyor_t_junction,
        "splitter": create_conveyor_splitter,
    }

    machine_models = {
        "miner": create_miner,
        "furnace": create_furnace,
        "crusher": create_crusher,
        "assembler": create_assembler,
        "storage": create_storage,
        "generator": create_generator,
        "inserter": create_inserter,
        "splitter_machine": create_splitter_machine,
    }

    item_models = {
        "iron_ore": create_item_iron_ore,
        "copper_ore": create_item_copper_ore,
        "coal": create_item_coal,
        "stone": create_item_stone,
        "iron_ingot": create_item_iron_ingot,
        "copper_ingot": create_item_copper_ingot,
    }

    all_models = {**conveyor_models, **machine_models, **item_models}

    if len(sys.argv) > 1:
        name = sys.argv[1]
        if name in conveyor_models:
            model = conveyor_models[name]()
            model.save(conveyor_dir / f"{name}.vox")
            print(f"Stats: {model.get_stats()}")
        elif name in machine_models:
            model = machine_models[name]()
            model.save(machines_dir / f"{name}.vox")
            print(f"Stats: {model.get_stats()}")
        elif name in item_models:
            model = item_models[name]()
            model.save(items_dir / f"{name}.vox")
            print(f"Stats: {model.get_stats()}")
        elif name == "all":
            # 全モデル生成
            print("Generating all models...")
            for n, creator in conveyor_models.items():
                model = creator()
                model.save(conveyor_dir / f"{n}.vox")
                print(f"  conveyor/{n}: {model.get_stats()}")
            for n, creator in machine_models.items():
                model = creator()
                model.save(machines_dir / f"{n}.vox")
                print(f"  {n}: {model.get_stats()}")
            for n, creator in item_models.items():
                model = creator()
                model.save(items_dir / f"{n}.vox")
                print(f"  items/{n}: {model.get_stats()}")
            print("Done!")
        else:
            print(f"Unknown model: {name}")
            print(f"Available: {', '.join(all_models.keys())}, all")
    else:
        # 全モデル生成
        print("Generating all models...")
        for n, creator in conveyor_models.items():
            model = creator()
            model.save(conveyor_dir / f"{n}.vox")
            print(f"  conveyor/{n}: {model.get_stats()}")
        for n, creator in machine_models.items():
            model = creator()
            model.save(machines_dir / f"{n}.vox")
            print(f"  {n}: {model.get_stats()}")
        for n, creator in item_models.items():
            model = creator()
            model.save(items_dir / f"{n}.vox")
            print(f"  items/{n}: {model.get_stats()}")
        print("Done!")
