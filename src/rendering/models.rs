use crate::rendering::meshing::Direction;

// メッシュ構築用のヘルパー構造体
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
        // Top
        self.face([x, y+sy, z+sz], [x+sx, y+sy, z+sz], [x+sx, y+sy, z], [x, y+sy, z], [0.0, 1.0, 0.0], color);
        // Bottom
        self.face([x, y, z], [x+sx, y, z], [x+sx, y, z+sz], [x, y, z+sz], [0.0, -1.0, 0.0], color);
        // East
        self.face([x+sx, y, z+sz], [x+sx, y+sy, z+sz], [x+sx, y+sy, z], [x+sx, y, z], [1.0, 0.0, 0.0], color);
        // West
        self.face([x, y, z], [x, y+sy, z], [x, y+sy, z+sz], [x, y, z+sz], [-1.0, 0.0, 0.0], color);
        // South
        self.face([x, y, z+sz], [x+sx, y, z+sz], [x+sx, y+sy, z+sz], [x, y+sy, z+sz], [0.0, 0.0, 1.0], color);
        // North
        self.face([x+sx, y, z], [x, y, z], [x, y+sy, z], [x+sx, y+sy, z], [0.0, 0.0, -1.0], color);
    }

    // 内部用: 1面追加
    fn face(&mut self, v0: [f32;3], v1: [f32;3], v2: [f32;3], v3: [f32;3], normal: [f32;3], color: [f32;4]) {
        self.positions.extend_from_slice(&[v0, v1, v2, v3]);
        self.normals.extend_from_slice(&[normal, normal, normal, normal]);
        self.colors.extend_from_slice(&[color, color, color, color]);
        let i = *self.idx_counter;
        self.indices.extend_from_slice(&[i, i+1, i+2, i+2, i+3, i]);
        *self.idx_counter += 4;
    }
    
    // 通常ブロックの面（Culling対応）
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

// ★ここに追加していく: アイテムごとの形状定義
pub fn mesh_conveyor(builder: &mut MeshBuilder, x: f32, y: f32, z: f32, color: [f32; 4]) {
    // 平たい板
    builder.push_cuboid(x, y, z, 1.0, 0.2, 1.0, color);
    
    // (例) 将来的に脚をつけたり、ガイドレールをつけたりもここで記述
}