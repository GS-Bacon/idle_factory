use crate::rendering::meshing::Direction;

pub struct MeshBuilder<'a> {
    pub positions: &'a mut Vec<[f32; 3]>,
    pub normals: &'a mut Vec<[f32; 3]>,
    pub indices: &'a mut Vec<u32>,
    pub colors: &'a mut Vec<[f32; 4]>,
    pub idx_counter: &'a mut u32,
}

impl<'a> MeshBuilder<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn push_cuboid(&mut self, x: f32, y: f32, z: f32, sx: f32, sy: f32, sz: f32, color: [f32; 4]) {
        self.face([x, y+sy, z+sz], [x+sx, y+sy, z+sz], [x+sx, y+sy, z], [x, y+sy, z], [0.0, 1.0, 0.0], color); // Top
        self.face([x, y, z], [x+sx, y, z], [x+sx, y, z+sz], [x, y, z+sz], [0.0, -1.0, 0.0], color); // Bottom
        self.face([x+sx, y, z+sz], [x+sx, y+sy, z+sz], [x+sx, y+sy, z], [x+sx, y, z], [1.0, 0.0, 0.0], color); // East
        self.face([x, y, z], [x, y+sy, z], [x, y+sy, z+sz], [x, y, z+sz], [-1.0, 0.0, 0.0], color); // West
        self.face([x, y, z+sz], [x+sx, y, z+sz], [x+sx, y+sy, z+sz], [x, y+sy, z+sz], [0.0, 0.0, 1.0], color); // South
        self.face([x+sx, y, z], [x, y, z], [x, y+sy, z], [x+sx, y+sy, z], [0.0, 0.0, -1.0], color); // North
    }

    fn face(&mut self, v0: [f32;3], v1: [f32;3], v2: [f32;3], v3: [f32;3], normal: [f32;3], color: [f32;4]) {
        self.positions.extend_from_slice(&[v0, v1, v2, v3]);
        self.normals.extend_from_slice(&[normal, normal, normal, normal]);
        self.colors.extend_from_slice(&[color, color, color, color]);
        let i = *self.idx_counter;
        self.indices.extend_from_slice(&[i, i+1, i+2, i+2, i+3, i]);
        *self.idx_counter += 4;
    }
    
    pub fn push_face_by_dir(&mut self, x: f32, y: f32, z: f32, dir: Direction, color: [f32; 4]) {
        let (v0, v1, v2, v3, normal) = match dir {
            Direction::YPos => ([x, y + 1.0, z + 1.0], [x + 1.0, y + 1.0, z + 1.0], [x + 1.0, y + 1.0, z], [x, y + 1.0, z], [0.0, 1.0, 0.0]),
            Direction::YNeg => ([x, y, z], [x + 1.0, y, z], [x + 1.0, y, z + 1.0], [x, y, z + 1.0], [0.0, -1.0, 0.0]),
            Direction::XPos => ([x + 1.0, y, z + 1.0], [x + 1.0, y, z], [x + 1.0, y + 1.0, z], [x + 1.0, y + 1.0, z + 1.0], [1.0, 0.0, 0.0]),
            Direction::XNeg => ([x, y, z], [x, y, z + 1.0], [x, y + 1.0, z + 1.0], [x, y + 1.0, z], [-1.0, 0.0, 0.0]),
            Direction::ZPos => ([x, y, z + 1.0], [x + 1.0, y, z + 1.0], [x + 1.0, y + 1.0, z + 1.0], [x, y + 1.0, z + 1.0], [0.0, 0.0, 1.0]),
            Direction::ZNeg => ([x + 1.0, y, z], [x, y, z], [x, y + 1.0, z], [x + 1.0, y + 1.0, z], [0.0, 0.0, -1.0]),
        };
        self.face(v0, v1, v2, v3, normal, color);
    }
}

// ★修正: 関数ポインタをやめ、モデルIDを持つように変更
pub enum MeshType {
    Cube,
    VoxModel(String), // "miner" などのID
}

pub struct BlockVisual {
    pub color: [f32; 4],
    pub mesh_type: MeshType,
    pub is_transparent: bool,
}

// ★修正: VoxModelを指定するように変更
pub fn get_block_visual(id: &str) -> BlockVisual {
    match id {
        // === 機械ブロック ===
        "conveyor" => BlockVisual {
            color: [0.2, 0.2, 0.2, 1.0],
            mesh_type: MeshType::VoxModel("conveyor".to_string()),
            is_transparent: true,
        },
        "miner" => BlockVisual {
            color: [0.8, 0.4, 0.0, 1.0],
            mesh_type: MeshType::VoxModel("miner".to_string()),
            is_transparent: true,
        },

        // === 基本地形ブロック ===
        "dirt" => BlockVisual {
            color: [0.4, 0.25, 0.1, 1.0],
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
        "grass" => BlockVisual {
            color: [0.3, 0.6, 0.2, 1.0],  // 緑
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
        "stone" => BlockVisual {
            color: [0.5, 0.5, 0.5, 1.0],
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
        "sand" => BlockVisual {
            color: [0.9, 0.85, 0.6, 1.0],  // 砂色
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
        "sandstone" => BlockVisual {
            color: [0.85, 0.75, 0.5, 1.0],  // 砂岩色
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
        "gravel" => BlockVisual {
            color: [0.55, 0.5, 0.45, 1.0],  // 砂利色
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
        "deepslate" => BlockVisual {
            color: [0.25, 0.25, 0.3, 1.0],  // 深層岩（暗いグレー）
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
        "bedrock" => BlockVisual {
            color: [0.15, 0.15, 0.15, 1.0],  // 岩盤（ほぼ黒）
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },

        // === 鉱石ブロック ===
        "coal_ore" => BlockVisual {
            color: [0.3, 0.3, 0.3, 1.0],  // 石炭（黒っぽい）
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
        "iron_ore" => BlockVisual {
            color: [0.6, 0.5, 0.45, 1.0],  // 鉄（ベージュがかったグレー）
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
        "copper_ore" => BlockVisual {
            color: [0.7, 0.45, 0.3, 1.0],  // 銅（オレンジがかった茶色）
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
        "gold_ore" => BlockVisual {
            color: [0.9, 0.75, 0.3, 1.0],  // 金（黄色）
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
        // 深層鉱石
        "deepslate_coal_ore" => BlockVisual {
            color: [0.2, 0.2, 0.25, 1.0],
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
        "deepslate_iron_ore" => BlockVisual {
            color: [0.45, 0.4, 0.4, 1.0],
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
        "deepslate_copper_ore" => BlockVisual {
            color: [0.55, 0.35, 0.28, 1.0],
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
        "deepslate_gold_ore" => BlockVisual {
            color: [0.7, 0.6, 0.25, 1.0],
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },

        // === 液体・特殊ブロック ===
        "water" => BlockVisual {
            color: [0.2, 0.4, 0.8, 0.6],  // 半透明の青
            mesh_type: MeshType::Cube,
            is_transparent: true,
        },
        "lava" => BlockVisual {
            color: [0.9, 0.3, 0.1, 0.9],  // オレンジ
            mesh_type: MeshType::Cube,
            is_transparent: true,
        },

        // === 空気 ===
        "air" => BlockVisual {
            color: [0.0, 0.0, 0.0, 0.0],
            mesh_type: MeshType::Cube,
            is_transparent: true,
        },

        // === フォールバック ===
        _ => BlockVisual {
            color: [1.0, 0.0, 1.0, 1.0],  // マゼンタ（未定義ブロック）
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
    }
}