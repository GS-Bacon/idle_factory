use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

// --- Block Definitions ---

#[derive(Debug, Deserialize, Clone)]
pub struct BlockDefinition {
    pub id: String,
    pub name: String,
    pub is_solid: bool,
    #[serde(default = "default_texture")]
    pub texture: String,
    pub collision: Option<Vec<f32>>,
}

fn default_texture() -> String {
    "none".to_string()
}

#[derive(Debug, Clone)]
pub struct BlockProperty {
    pub name: String,
    pub is_solid: bool,
    pub texture: String,
    pub collision_box: [f32; 6],
}

#[derive(Resource, Default)]
pub struct BlockRegistry {
    pub map: HashMap<String, BlockProperty>,
}

// --- Recipe Definitions ---

#[derive(Debug, Deserialize, Clone)]
pub struct RecipeInput {
    pub item: String,
    pub count: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RecipeDefinition {
    pub id: String,
    pub name: String,
    pub inputs: Vec<RecipeInput>,
    pub outputs: Vec<RecipeInput>,
    pub craft_time: f32,
}

#[derive(Resource, Default)]
pub struct RecipeRegistry {
    pub map: HashMap<String, RecipeDefinition>,
}


// --- Plugin ---

pub struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockRegistry>()
            .init_resource::<RecipeRegistry>()
            .add_systems(Startup, (load_blocks, load_recipes));
    }
}

fn load_blocks(mut registry: ResMut<BlockRegistry>) {
    let path = "assets/data/blocks/core.yaml";
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(defs) = serde_yaml::from_str::<Vec<BlockDefinition>>(&content) {
            for def in defs {
                let col = if let Some(c) = def.collision {
                    if c.len() == 6 {
                        [c[0], c[1], c[2], c[3], c[4], c[5]]
                    } else {
                        warn!("Block {} has invalid collision data length.", def.id);
                        [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]
                    }
                } else {
                    [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]
                };

                registry.map.insert(def.id.clone(), BlockProperty {
                    name: def.name,
                    is_solid: def.is_solid,
                    texture: def.texture,
                    collision_box: col,
                });
                info!("Loaded block: {}", def.id);
            }
        } else {
            error!("Failed to parse YAML: {}", path);
        }
    } else {
        error!("Failed to read file: {}", path);
    }
}

fn load_recipes(mut registry: ResMut<RecipeRegistry>) {
    let path = "assets/data/recipes/vanilla.yaml";
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(defs) = serde_yaml::from_str::<Vec<RecipeDefinition>>(&content) {
            for def in defs {
                registry.map.insert(def.id.clone(), def);
                info!("Loaded recipe: {}", registry.map.get(&"ore_to_ingot".to_string()).unwrap().id);
            }
        } else {
            error!("Failed to parse YAML: {}", path);
        }
    } else {
        error!("Failed to read file: {}", path);
    }
}