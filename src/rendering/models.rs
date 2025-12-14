use crate::rendering::meshing::Direction;

// メッシュ構築用のヘルパー
pub struct MeshBuilder<'a> {
    pub positions: &'a mut Vec<[f32; 3]>,
    pub normals: &'a mut Vec<[f32; 3]>,
    pub indices: &'a mut Vec<u32>,
    pub colors: &'a mut Vec<[f32; 4]>,
    pub idx_counter: &'a mut u32,
}

impl<'a> MeshBuilder<'a> {
    // 汎用: 直方体を追加
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

// ★追加: ブロックの描画タイプ
pub enum MeshType {
    Cube, // 通常の1x1x1ブロック (Cullingあり)
    Custom(fn(&mut MeshBuilder, f32, f32, f32, [f32; 4])), // カスタム形状関数
}

// ★追加: ブロックの見た目情報まとめ
pub struct BlockVisual {
    pub color: [f32; 4],
    pub mesh_type: MeshType,
    pub is_transparent: bool, // これがtrueなら、隣接ブロックの面を描画させる(Cullingされない)
}

// ★追加: IDから見た目情報を引く関数 (ここを一元管理する)
pub fn get_block_visual(id: &str) -> BlockVisual {
    match id {
        "conveyor" => BlockVisual {
            color: [0.2, 0.2, 0.2, 1.0],
            mesh_type: MeshType::Custom(mesh_conveyor),
            is_transparent: true, // 背が低いので隣が見える
        },
        "miner" => BlockVisual {
            color: [0.8, 0.4, 0.0, 1.0],
            mesh_type: MeshType::Custom(mesh_miner),
            is_transparent: true,
        },
        "dirt" => BlockVisual {
            color: [0.4, 0.25, 0.1, 1.0],
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
        "stone" => BlockVisual {
            color: [0.5, 0.5, 0.5, 1.0],
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
        "air" => BlockVisual {
            color: [0.0, 0.0, 0.0, 0.0],
            mesh_type: MeshType::Cube,
            is_transparent: true,
        },
        _ => BlockVisual { // 未定義ブロック
            color: [1.0, 0.0, 1.0, 1.0], // マゼンタ
            mesh_type: MeshType::Cube,
            is_transparent: false,
        },
    }
}

// 個別のカスタムメッシュ関数
fn mesh_conveyor(builder: &mut MeshBuilder, x: f32, y: f32, z: f32, color: [f32; 4]) {
    builder.push_cuboid(x, y, z, 1.0, 0.2, 1.0, color);
}

fn mesh_miner(builder: &mut MeshBuilder, x: f32, y: f32, z: f32, color: [f32; 4]) {
    builder.push_cuboid(x, y, z, 1.0, 0.2, 1.0, color); // 土台
    builder.push_cuboid(x + 0.2, y + 0.2, z + 0.2, 0.6, 0.6, 0.6, [color[0]*0.8, color[1]*0.8, color[2]*0.8, 1.0]); // 本体
    builder.push_cuboid(x + 0.4, y + 0.8, z + 0.4, 0.2, 0.2, 0.2, [0.3, 0.3, 0.3, 1.0]); // ドリル
}