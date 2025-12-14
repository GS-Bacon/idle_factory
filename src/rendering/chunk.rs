use bevy::prelude::*;

// チャンクのサイズ定数 (32x32x32)
pub const CHUNK_SIZE: usize = 32;

#[derive(Component)]
pub struct Chunk {
    // ブロックIDを格納する1次元配列 (32*32*32 = 32768個)
    // StringのIDを入れると重いので、本来は数値ID(u16)に変換すべきですが、
    // まずは分かりやすくStringで実装します（後で最適化対象）。
    pub blocks: Vec<String>, 
    pub position: IVec3, // ワールド内のチャンク座標 (0,0,0), (1,0,0) など
}

impl Chunk {
    pub fn new(position: IVec3) -> Self {
        // 全て "air" で埋めて初期化
        let size = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
        Self {
            blocks: vec!["air".to_string(); size],
            position,
        }
    }

    // 3次元座標 (x, y, z) を 1次元インデックスに変換
    pub fn xyz_to_index(x: usize, y: usize, z: usize) -> usize {
        (y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x
    }

    // ブロックをセットする関数
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, id: &str) {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return;
        }
        let idx = Self::xyz_to_index(x, y, z);
        self.blocks[idx] = id.to_string();
    }

    // ブロックを取得する関数
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&String> {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return None;
        }
        let idx = Self::xyz_to_index(x, y, z);
        Some(&self.blocks[idx])
    }
}