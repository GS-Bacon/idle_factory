use bevy::prelude::*;
use std::collections::HashMap;
use dot_vox::load;

// 1つの小さなボクセルのデータ
#[derive(Clone)]
pub struct VoxelData {
    pub pos: Vec3,
    pub color: [f32; 4],
}

// 全てのモデルデータを保持するリソース
#[derive(Resource, Default)]
pub struct VoxelAssets {
    // モデルID ("miner" など) -> ボクセルのリスト
    pub models: HashMap<String, Vec<VoxelData>>,
}

// 起動時に .vox をロードするシステム
pub fn load_vox_assets(mut voxel_assets: ResMut<VoxelAssets>) {
    // 読み込みたいモデルのリスト (ID, ファイルパス)
    // ファイルが存在しない場合でもクラッシュせず、警告ログだけ出してスキップします
    let targets = vec![
        ("conveyor", "assets/models/conveyor.vox"),
        ("miner", "assets/models/miner.vox"),
    ];

    for (id, path) in targets {
        match load(path) {
            Ok(data) => {
                let mut voxels = Vec::new();
                
                // MagicaVoxelは複数のモデルを持てるが、最初の1つ(index 0)を使う
                if let Some(model) = data.models.first() {
                    for v in &model.voxels {
                        // パレットから色を取得
                        let color = if let Some(c) = data.palette.get(v.i as usize) {
                            // ★修正: dot_vox 5.1 では c は構造体なので、直接 r, g, b を取得する
                            let r = c.r as f32 / 255.0;
                            let g = c.g as f32 / 255.0;
                            let b = c.b as f32 / 255.0;
                            // alphaも必要なら c.a を使えますが、通常は不透明(1.0)でOK
                            [r, g, b, 1.0]
                        } else {
                            [1.0, 1.0, 1.0, 1.0]
                        };

                        // 座標変換 
                        // MagicaVoxel: X=Right, Y=Back, Z=Up (Z-up)
                        // Bevy: X=Right, Y=Up, Z=Back (Y-up)
                        // ここでは、MagicaVoxelのZをBevyのY(高さ)に、YをZ(奥行き)にマッピングします
                        voxels.push(VoxelData {
                            pos: Vec3::new(v.x as f32, v.z as f32, v.y as f32),
                            color,
                        });
                    }
                }
                
                debug!("📦 Loaded .vox model: {} ({} voxels)", id, voxels.len());
                voxel_assets.models.insert(id.to_string(), voxels);
            }
            Err(e) => {
                // ファイルがない場合は警告を出して、フォールバックとしてcubeを生成
                warn!("⚠️ Failed to load .vox model: {} ({}) - Using fallback cube mesh.", path, e);

                // フォールバックcubeを生成（8x8x8のシンプルなcube）
                let mut voxels = Vec::new();
                for x in 0..8 {
                    for y in 0..8 {
                        for z in 0..8 {
                            // 中空のcube（外側のみ）
                            if x == 0 || x == 7 || y == 0 || y == 7 || z == 0 || z == 7 {
                                voxels.push(VoxelData {
                                    pos: Vec3::new(x as f32, y as f32, z as f32),
                                    color: [0.7, 0.7, 0.7, 1.0], // グレー
                                });
                            }
                        }
                    }
                }

                debug!("📦 Generated fallback cube for: {} ({} voxels)", id, voxels.len());
                voxel_assets.models.insert(id.to_string(), voxels);
            }
        }
    }
}