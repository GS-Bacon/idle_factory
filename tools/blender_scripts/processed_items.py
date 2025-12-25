"""
パイプ・その他加工品モデル生成
カテゴリ: item (dropped_item: 0.4x0.4x0.4)
"""

import bpy
from mathutils import Vector
from math import pi

# _base.pyの関数を使用（先に_base.pyを実行済み前提）
# create_pipe, create_chamfered_cube, create_octagon
# create_material, apply_preset_material
# finalize_model, export_gltf

# =============================================================================
# パイプ類
# =============================================================================

def create_iron_pipe():
    """鉄パイプ - 灰色の八角形パイプ"""
    clear_scene()

    pipe = create_pipe(
        radius=0.15,
        length=0.35,
        wall=0.02,
        location=(0, 0, 0),
        name="iron_pipe"
    )

    # 鉄マテリアル（暗めのメタル）
    mat = create_material(
        "iron_pipe_mat",
        color=(0.29, 0.29, 0.29, 1),
        metallic=1.0,
        roughness=0.5
    )
    apply_material(pipe, mat)

    finalize_model(pipe, category="item")
    export_gltf("iron_pipe.gltf", export_animations=False)
    print("✓ iron_pipe created")

def create_copper_pipe():
    """銅パイプ - 銅色の八角形パイプ"""
    clear_scene()

    pipe = create_pipe(
        radius=0.15,
        length=0.35,
        wall=0.02,
        location=(0, 0, 0),
        name="copper_pipe"
    )

    # 銅マテリアル
    mat = create_material(
        "copper_pipe_mat",
        preset="copper"
    )
    apply_material(pipe, mat)

    finalize_model(pipe, category="item")
    export_gltf("copper_pipe.gltf", export_animations=False)
    print("✓ copper_pipe created")

def create_steel_pipe():
    """鋼鉄パイプ - より暗い灰色のパイプ"""
    clear_scene()

    pipe = create_pipe(
        radius=0.15,
        length=0.35,
        wall=0.02,
        location=(0, 0, 0),
        name="steel_pipe"
    )

    # 鋼鉄マテリアル（鉄より暗い）
    mat = create_material(
        "steel_pipe_mat",
        preset="dark_steel"
    )
    apply_material(pipe, mat)

    finalize_model(pipe, category="item")
    export_gltf("steel_pipe.gltf", export_animations=False)
    print("✓ steel_pipe created")

# =============================================================================
# プラスチック・ゴム系
# =============================================================================

def create_rubber():
    """ゴムのかたまり - 黒い丸みを帯びた形状"""
    clear_scene()

    # 面取り大きめで丸みを表現
    rubber = create_chamfered_cube(
        size=(0.3, 0.25, 0.2),
        chamfer=0.05,
        location=(0, 0, 0),
        name="rubber"
    )

    # 黒いゴムマテリアル
    mat = create_material(
        "rubber_mat",
        color=(0.08, 0.08, 0.08, 1),
        metallic=0.0,
        roughness=0.6
    )
    apply_material(rubber, mat)

    finalize_model(rubber, category="item")
    export_gltf("rubber.gltf", export_animations=False)
    print("✓ rubber created")

def create_plastic():
    """プラスチック片 - 半透明っぽい明るい色"""
    clear_scene()

    # やや不規則な形状
    plastic = create_chamfered_cube(
        size=(0.28, 0.22, 0.15),
        chamfer=0.04,
        location=(0, 0, 0),
        name="plastic"
    )

    # 半透明風の白っぽいプラスチック
    mat = create_material(
        "plastic_mat",
        color=(0.85, 0.85, 0.88, 1),
        metallic=0.0,
        roughness=0.3
    )
    apply_material(plastic, mat)

    finalize_model(plastic, category="item")
    export_gltf("plastic.gltf", export_animations=False)
    print("✓ plastic created")

# =============================================================================
# ガラス・建材系
# =============================================================================

def create_glass():
    """ガラス板 - 透明な薄い板"""
    clear_scene()

    # 薄い板状
    glass = create_chamfered_cube(
        size=(0.3, 0.3, 0.05),
        chamfer=0.008,
        location=(0, 0, 0),
        name="glass"
    )

    # 透明ガラスマテリアル
    mat = create_material(
        "glass_mat",
        color=(0.85, 0.92, 0.95, 1),
        metallic=0.0,
        roughness=0.05
    )
    apply_material(glass, mat)

    finalize_model(glass, category="item")
    export_gltf("glass.gltf", export_animations=False)
    print("✓ glass created")

def create_brick():
    """レンガブロック - 赤茶色の長方形"""
    clear_scene()

    # レンガ形状（横長）
    brick = create_chamfered_cube(
        size=(0.35, 0.18, 0.15),
        chamfer=0.015,
        location=(0, 0, 0),
        name="brick"
    )

    # 赤茶色のレンガマテリアル
    mat = create_material(
        "brick_mat",
        color=(0.65, 0.28, 0.18, 1),
        metallic=0.0,
        roughness=0.85
    )
    apply_material(brick, mat)

    finalize_model(brick, category="item")
    export_gltf("brick.gltf", export_animations=False)
    print("✓ brick created")

def create_concrete():
    """コンクリートブロック - 灰色のブロック"""
    clear_scene()

    # コンクリート形状（立方体に近い）
    concrete = create_chamfered_cube(
        size=(0.3, 0.3, 0.25),
        chamfer=0.02,
        location=(0, 0, 0),
        name="concrete"
    )

    # 灰色のコンクリートマテリアル
    mat = create_material(
        "concrete_mat",
        color=(0.48, 0.48, 0.48, 1),
        metallic=0.0,
        roughness=0.9
    )
    apply_material(concrete, mat)

    finalize_model(concrete, category="item")
    export_gltf("concrete.gltf", export_animations=False)
    print("✓ concrete created")

# =============================================================================
# 鉱石・副産物系
# =============================================================================

def create_coal():
    """石炭のかたまり - 黒いブロック"""
    clear_scene()

    # やや不規則な形状
    coal = create_chamfered_cube(
        size=(0.25, 0.28, 0.22),
        chamfer=0.04,
        location=(0, 0, 0),
        name="coal"
    )

    # 黒い石炭マテリアル
    mat = create_material(
        "coal_mat",
        color=(0.12, 0.12, 0.12, 1),
        metallic=0.0,
        roughness=0.8
    )
    apply_material(coal, mat)

    finalize_model(coal, category="item")
    export_gltf("coal.gltf", export_animations=False)
    print("✓ coal created")

def create_stone_dust():
    """石粉 - 灰色の粉状（小さな塊）"""
    clear_scene()

    # 小さめの不規則な形状
    dust = create_chamfered_cube(
        size=(0.2, 0.18, 0.12),
        chamfer=0.035,
        location=(0, 0, 0),
        name="stone_dust"
    )

    # 灰色の粉マテリアル
    mat = create_material(
        "stone_dust_mat",
        color=(0.55, 0.55, 0.55, 1),
        metallic=0.0,
        roughness=1.0
    )
    apply_material(dust, mat)

    finalize_model(dust, category="item")
    export_gltf("stone_dust.gltf", export_animations=False)
    print("✓ stone_dust created")

def create_slag():
    """スラグ（鉱滓） - 黒っぽい副産物"""
    clear_scene()

    # 不規則な塊
    slag = create_chamfered_cube(
        size=(0.22, 0.25, 0.18),
        chamfer=0.04,
        location=(0, 0, 0),
        name="slag"
    )

    # 黒っぽいスラグマテリアル
    mat = create_material(
        "slag_mat",
        color=(0.15, 0.12, 0.10, 1),
        metallic=0.1,
        roughness=0.85
    )
    apply_material(slag, mat)

    finalize_model(slag, category="item")
    export_gltf("slag.gltf", export_animations=False)
    print("✓ slag created")

# =============================================================================
# 天然素材系
# =============================================================================

def create_rubber_sap():
    """樹液のかたまり - 琥珀色の丸い塊"""
    clear_scene()

    # 丸みを帯びた形状
    sap = create_chamfered_cube(
        size=(0.2, 0.2, 0.18),
        chamfer=0.06,
        location=(0, 0, 0),
        name="rubber_sap"
    )

    # 琥珀色の樹液マテリアル
    mat = create_material(
        "rubber_sap_mat",
        color=(0.75, 0.55, 0.15, 1),
        metallic=0.0,
        roughness=0.2
    )
    apply_material(sap, mat)

    finalize_model(sap, category="item")
    export_gltf("rubber_sap.gltf", export_animations=False)
    print("✓ rubber_sap created")

def create_raw_quartz():
    """粗石英 - 結晶形状（八角柱ベース）"""
    clear_scene()

    # 八角柱で結晶を表現
    quartz = create_octagonal_prism(
        radius=0.12,
        height=0.35,
        location=(0, 0, 0),
        name="raw_quartz"
    )

    # 回転させて斜めに
    quartz.rotation_euler = (pi / 6, 0, pi / 8)

    # 白っぽい半透明風の石英マテリアル
    mat = create_material(
        "raw_quartz_mat",
        color=(0.92, 0.92, 0.95, 1),
        metallic=0.0,
        roughness=0.15
    )
    apply_material(quartz, mat)

    finalize_model(quartz, category="item")
    export_gltf("raw_quartz.gltf", export_animations=False)
    print("✓ raw_quartz created")

# =============================================================================
# 一括生成
# =============================================================================

def create_all():
    """全アイテムを生成"""
    print("\n=== Creating Processed Items ===\n")

    # パイプ類
    create_iron_pipe()
    create_copper_pipe()
    create_steel_pipe()

    # プラスチック・ゴム系
    create_rubber()
    create_plastic()

    # ガラス・建材系
    create_glass()
    create_brick()
    create_concrete()

    # 鉱石・副産物系
    create_coal()
    create_stone_dust()
    create_slag()

    # 天然素材系
    create_rubber_sap()
    create_raw_quartz()

    print("\n=== All Processed Items Created ===")
    print("Total: 13 items")

# 実行
if __name__ == "__main__":
    create_all()
