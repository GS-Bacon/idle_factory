use bevy::prelude::*;
use bevy::utils::HashMap;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct BlockDefinition {
    pub id: String,
    pub name: String,
    pub texture: String,
    pub is_solid: bool,
}

#[derive(Resource, Default)]
pub struct BlockRegistry {
    pub map: HashMap<String, BlockDefinition>,
}

pub fn load_block_registry(mut registry: ResMut<BlockRegistry>) {
    // 読み込むフォルダのパス
    let folder_path = "assets/data/blocks";
    
    info!("Loading blocks from: {}", folder_path);

    // フォルダを開く
    let entries = match fs::read_dir(folder_path) {
        Ok(e) => e,
        Err(e) => {
            error!("Failed to read directory {}: {}", folder_path, e);
            return;
        }
    };

    // フォルダ内のファイルを1つずつ処理
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            
            // 拡張子が .yaml または .yml の場合のみ読み込む
            if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                load_blocks_from_file(&path, &mut registry);
            }
        }
    }
}

// 1つのファイルから読み込む補助関数
fn load_blocks_from_file(path: &Path, registry: &mut ResMut<BlockRegistry>) {
    match fs::read_to_string(path) {
        Ok(content) => {
            // YAMLパース
            let blocks: Result<Vec<BlockDefinition>, _> = serde_yaml::from_str(&content);
            
            match blocks {
                Ok(blocks) => {
                    for block in blocks {
                        info!("Loaded Block: {} ({}) from {:?}", block.name, block.id, path);
                        registry.map.insert(block.id.clone(), block);
                    }
                }
                Err(e) => {
                    error!("Failed to parse YAML {:?}: {}", path, e);
                }
            }
        }
        Err(e) => {
            error!("Failed to read file {:?}: {}", path, e);
        }
    }
}