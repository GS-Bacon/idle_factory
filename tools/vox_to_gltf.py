#!/usr/bin/env python3
"""
VOX to glTF converter using Blender

.voxファイルを読み込み、最適化されたメッシュとしてglTF/glbに変換する。
ボクセルを個別キューブではなく、面を統合した効率的なメッシュに変換。

使い方:
    # Blender経由で実行
    blender --background --python vox_to_gltf.py -- input.vox output.glb

    # または直接実行（Blenderがパスにある場合）
    python3 vox_to_gltf.py input.vox output.glb
"""

import struct
import sys
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass


@dataclass
class VoxModel:
    """VOXファイルから読み込んだモデルデータ"""
    size_x: int
    size_y: int
    size_z: int
    voxels: Dict[Tuple[int, int, int], int]
    palette: List[Tuple[int, int, int, int]]


def read_vox(filepath: str) -> Optional[VoxModel]:
    """VOXファイルを読み込む"""
    with open(filepath, 'rb') as f:
        # Header
        magic = f.read(4)
        if magic != b'VOX ':
            print(f"Invalid VOX file: {magic}")
            return None

        version = struct.unpack('<I', f.read(4))[0]

        # Read MAIN chunk
        main_id = f.read(4)
        if main_id != b'MAIN':
            print(f"Expected MAIN chunk, got: {main_id}")
            return None

        content_size = struct.unpack('<I', f.read(4))[0]
        children_size = struct.unpack('<I', f.read(4))[0]

        # Skip MAIN content (should be empty)
        f.read(content_size)

        size_x, size_y, size_z = 0, 0, 0
        voxels = {}
        palette = [(255, 255, 255, 255)] * 256  # Default white

        # Read child chunks
        end_pos = f.tell() + children_size
        while f.tell() < end_pos:
            chunk_id = f.read(4)
            if len(chunk_id) < 4:
                break

            chunk_content_size = struct.unpack('<I', f.read(4))[0]
            chunk_children_size = struct.unpack('<I', f.read(4))[0]

            if chunk_id == b'SIZE':
                size_x, size_y, size_z = struct.unpack('<III', f.read(12))

            elif chunk_id == b'XYZI':
                num_voxels = struct.unpack('<I', f.read(4))[0]
                for _ in range(num_voxels):
                    x, y, z, i = struct.unpack('<BBBB', f.read(4))
                    voxels[(x, y, z)] = i

            elif chunk_id == b'RGBA':
                for i in range(256):
                    r, g, b, a = struct.unpack('<BBBB', f.read(4))
                    palette[i] = (r, g, b, a)

            else:
                # Skip unknown chunk
                f.read(chunk_content_size + chunk_children_size)

    return VoxModel(size_x, size_y, size_z, voxels, palette)


def generate_optimized_mesh(model: VoxModel, use_greedy: bool = True) -> Tuple[List[Tuple[float, float, float]],
                                                        List[Tuple[int, int, int]],
                                                        List[int],
                                                        Dict[int, Tuple[float, float, float, float]]]:
    """
    ボクセルから最適化されたメッシュを生成

    Args:
        model: VOXモデル
        use_greedy: Trueならグリーディメッシング（面統合）を使用

    Returns:
        vertices: 頂点リスト
        faces: 面リスト（三角形インデックス）
        face_colors: 各面のカラーインデックス
        materials: カラーインデックス→RGBA のマップ
    """
    if use_greedy:
        return _generate_greedy_mesh(model)
    else:
        return _generate_naive_mesh(model)


def _generate_naive_mesh(model: VoxModel) -> Tuple[List, List, List, Dict]:
    """単純なメッシュ生成（各ボクセルの面を個別に生成）"""
    voxels = model.voxels
    palette = model.palette

    vertices = []
    faces = []
    face_colors = []
    materials = {}

    voxel_size = 1.0 / 16.0

    face_directions = [
        (1, 0, 0, [(1, 0, 0), (1, 1, 0), (1, 1, 1), (1, 0, 1)]),
        (-1, 0, 0, [(0, 0, 0), (0, 0, 1), (0, 1, 1), (0, 1, 0)]),
        (0, 1, 0, [(0, 1, 0), (0, 1, 1), (1, 1, 1), (1, 1, 0)]),
        (0, -1, 0, [(0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)]),
        (0, 0, 1, [(0, 0, 1), (1, 0, 1), (1, 1, 1), (0, 1, 1)]),
        (0, 0, -1, [(0, 0, 0), (0, 1, 0), (1, 1, 0), (1, 0, 0)]),
    ]

    for (x, y, z), color_index in voxels.items():
        if color_index not in materials:
            r, g, b, a = palette[color_index - 1]
            materials[color_index] = (r / 255.0, g / 255.0, b / 255.0, a / 255.0)

        for dx, dy, dz, face_verts in face_directions:
            neighbor = (x + dx, y + dy, z + dz)
            if neighbor in voxels:
                continue

            base_idx = len(vertices)
            for vx, vy, vz in face_verts:
                vertices.append((
                    (x + vx - model.size_x / 2) * voxel_size,
                    (y + vy - model.size_y / 2) * voxel_size,
                    (z + vz) * voxel_size
                ))

            faces.append((base_idx, base_idx + 1, base_idx + 2))
            faces.append((base_idx, base_idx + 2, base_idx + 3))
            face_colors.append(color_index)
            face_colors.append(color_index)

    return vertices, faces, face_colors, materials


def _generate_greedy_mesh(model: VoxModel) -> Tuple[List, List, List, Dict]:
    """
    グリーディメッシング - 同じ色の隣接面を統合して大きな四角形に

    アルゴリズム:
    1. 各軸方向のスライスごとに処理
    2. 同じ色で隣接する面を矩形にまとめる
    3. 統合された矩形を頂点・面として出力
    """
    voxels = model.voxels
    palette = model.palette

    vertices = []
    faces = []
    face_colors = []
    materials = {}

    voxel_size = 1.0 / 16.0
    offset_x = model.size_x / 2
    offset_y = model.size_y / 2

    # 6方向それぞれで処理
    # (normal_axis, u_axis, v_axis, normal_dir, flip_winding)
    directions = [
        ('x', 'y', 'z', 1, False),   # +X
        ('x', 'y', 'z', -1, True),   # -X
        ('y', 'x', 'z', 1, True),    # +Y
        ('y', 'x', 'z', -1, False),  # -Y
        ('z', 'x', 'y', 1, False),   # +Z
        ('z', 'x', 'y', -1, True),   # -Z
    ]

    def get_axis_size(axis):
        return {'x': model.size_x, 'y': model.size_y, 'z': model.size_z}[axis]

    def get_coord(pos, axis):
        return {'x': pos[0], 'y': pos[1], 'z': pos[2]}[axis]

    def make_pos(n_val, u_val, v_val, n_axis, u_axis, v_axis):
        pos = [0, 0, 0]
        axes = {'x': 0, 'y': 1, 'z': 2}
        pos[axes[n_axis]] = n_val
        pos[axes[u_axis]] = u_val
        pos[axes[v_axis]] = v_val
        return tuple(pos)

    for n_axis, u_axis, v_axis, n_dir, flip in directions:
        n_size = get_axis_size(n_axis)
        u_size = get_axis_size(u_axis)
        v_size = get_axis_size(v_axis)

        # 各スライスを処理
        for n in range(n_size):
            # このスライスで露出している面を収集
            # mask[u][v] = color_index or 0 (面なし)
            mask = [[0] * v_size for _ in range(u_size)]

            for u in range(u_size):
                for v in range(v_size):
                    pos = make_pos(n, u, v, n_axis, u_axis, v_axis)

                    if pos not in voxels:
                        continue

                    # 隣接チェック
                    if n_dir > 0:
                        neighbor = make_pos(n + 1, u, v, n_axis, u_axis, v_axis)
                    else:
                        neighbor = make_pos(n - 1, u, v, n_axis, u_axis, v_axis)

                    if neighbor in voxels:
                        continue

                    mask[u][v] = voxels[pos]

                    # マテリアル登録
                    color_index = voxels[pos]
                    if color_index not in materials:
                        r, g, b, a = palette[color_index - 1]
                        materials[color_index] = (r / 255.0, g / 255.0, b / 255.0, a / 255.0)

            # グリーディに矩形を抽出
            for u in range(u_size):
                v = 0
                while v < v_size:
                    color = mask[u][v]
                    if color == 0:
                        v += 1
                        continue

                    # 幅を計算（V方向に拡張）
                    width = 1
                    while v + width < v_size and mask[u][v + width] == color:
                        width += 1

                    # 高さを計算（U方向に拡張）
                    height = 1
                    done = False
                    while u + height < u_size and not done:
                        for k in range(width):
                            if mask[u + height][v + k] != color:
                                done = True
                                break
                        if not done:
                            height += 1

                    # マスクをクリア
                    for du in range(height):
                        for dv in range(width):
                            mask[u + du][v + dv] = 0

                    # 四角形を生成
                    # 面の座標を計算
                    if n_dir > 0:
                        n_coord = n + 1
                    else:
                        n_coord = n

                    # 4頂点を生成
                    corners = [
                        make_pos(n_coord, u, v, n_axis, u_axis, v_axis),
                        make_pos(n_coord, u + height, v, n_axis, u_axis, v_axis),
                        make_pos(n_coord, u + height, v + width, n_axis, u_axis, v_axis),
                        make_pos(n_coord, u, v + width, n_axis, u_axis, v_axis),
                    ]

                    # ワールド座標に変換
                    base_idx = len(vertices)
                    for cx, cy, cz in corners:
                        vertices.append((
                            (cx - offset_x) * voxel_size,
                            (cy - offset_y) * voxel_size,
                            cz * voxel_size
                        ))

                    # 面を追加（ワインディング順序に注意）
                    if flip:
                        faces.append((base_idx, base_idx + 3, base_idx + 2))
                        faces.append((base_idx, base_idx + 2, base_idx + 1))
                    else:
                        faces.append((base_idx, base_idx + 1, base_idx + 2))
                        faces.append((base_idx, base_idx + 2, base_idx + 3))

                    face_colors.append(color)
                    face_colors.append(color)

                    v += width

    return vertices, faces, face_colors, materials


def convert_in_blender(vox_path: str, output_path: str):
    """Blender内で変換を実行"""
    import bpy
    import bmesh

    # シーンをクリア
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete()

    # VOXを読み込み
    model = read_vox(vox_path)
    if model is None:
        print(f"Failed to read: {vox_path}")
        return False

    print(f"Loaded: {len(model.voxels)} voxels, size: {model.size_x}x{model.size_y}x{model.size_z}")

    # メッシュ生成
    vertices, faces, face_colors, materials = generate_optimized_mesh(model)
    print(f"Generated mesh: {len(vertices)} vertices, {len(faces)} triangles")

    # Blenderメッシュ作成
    mesh = bpy.data.meshes.new("VoxelMesh")
    obj = bpy.data.objects.new("VoxelModel", mesh)
    bpy.context.collection.objects.link(obj)

    # BMeshで構築
    bm = bmesh.new()

    # 頂点追加
    bm_verts = [bm.verts.new(v) for v in vertices]
    bm.verts.ensure_lookup_table()

    # 面追加
    for i, face in enumerate(faces):
        try:
            f = bm.faces.new([bm_verts[idx] for idx in face])
        except ValueError:
            pass  # 重複面はスキップ

    bm.to_mesh(mesh)
    bm.free()

    # マテリアル作成
    mat_map = {}
    for color_idx, rgba in materials.items():
        mat = bpy.data.materials.new(f"Color_{color_idx}")
        mat.use_nodes = True
        bsdf = mat.node_tree.nodes.get("Principled BSDF")
        if bsdf:
            bsdf.inputs["Base Color"].default_value = rgba
            bsdf.inputs["Metallic"].default_value = 0.3
            bsdf.inputs["Roughness"].default_value = 0.7
        obj.data.materials.append(mat)
        mat_map[color_idx] = len(obj.data.materials) - 1

    # 面にマテリアルを割り当て
    for i, poly in enumerate(mesh.polygons):
        if i < len(face_colors):
            color_idx = face_colors[i]
            if color_idx in mat_map:
                poly.material_index = mat_map[color_idx]

    # スムーズシェーディングをオフ（フラット）
    for poly in mesh.polygons:
        poly.use_smooth = False

    # 重複頂点をマージ
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.mesh.remove_doubles(threshold=0.0001)
    bpy.ops.object.mode_set(mode='OBJECT')

    # glTF/glbエクスポート
    bpy.ops.export_scene.gltf(
        filepath=output_path,
        export_format='GLB' if output_path.endswith('.glb') else 'GLTF_SEPARATE',
        export_apply=True,
        export_materials='EXPORT',
    )

    print(f"Exported: {output_path}")
    return True


def main():
    """メインエントリポイント"""
    # Blender経由の場合、--以降が引数
    if '--' in sys.argv:
        args = sys.argv[sys.argv.index('--') + 1:]
    else:
        args = sys.argv[1:]

    if len(args) < 2:
        print("Usage: blender --background --python vox_to_gltf.py -- input.vox output.glb")
        print("   or: python3 vox_to_gltf.py input.vox output.glb")
        sys.exit(1)

    vox_path = args[0]
    output_path = args[1]

    # Blenderモジュールがあるか確認
    try:
        import bpy
        convert_in_blender(vox_path, output_path)
    except ImportError:
        # Blenderなしで実行された場合、Blenderを呼び出す
        import subprocess
        script_path = Path(__file__).resolve()
        cmd = [
            'blender', '--background', '--python', str(script_path),
            '--', vox_path, output_path
        ]
        subprocess.run(cmd)


if __name__ == "__main__":
    main()
