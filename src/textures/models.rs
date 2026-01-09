//! Block model definitions
//!
//! Based on Minecraft's models JSON format.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Block model definition (loaded from JSON)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelDefinition {
    /// Parent model to inherit from
    #[serde(default)]
    pub parent: Option<String>,
    /// Texture variable definitions
    #[serde(default)]
    pub textures: HashMap<String, String>,
    /// Model elements (custom geometry)
    #[serde(default)]
    pub elements: Option<Vec<ModelElement>>,
    /// Display transforms
    #[serde(default)]
    pub display: Option<HashMap<String, DisplayTransform>>,
}

/// A single model element (box)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelElement {
    /// Start corner [x, y, z] (0-16 scale)
    pub from: [f32; 3],
    /// End corner [x, y, z] (0-16 scale)
    pub to: [f32; 3],
    /// Rotation of the element
    #[serde(default)]
    pub rotation: Option<ElementRotation>,
    /// Whether to apply ambient occlusion
    #[serde(default = "default_shade")]
    pub shade: bool,
    /// Face definitions
    #[serde(default)]
    pub faces: HashMap<String, ElementFace>,
}

fn default_shade() -> bool {
    true
}

/// Element rotation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ElementRotation {
    /// Rotation origin [x, y, z]
    pub origin: [f32; 3],
    /// Rotation axis ("x", "y", or "z")
    pub axis: String,
    /// Rotation angle (-45, -22.5, 0, 22.5, 45)
    pub angle: f32,
    /// Whether to rescale faces
    #[serde(default)]
    pub rescale: bool,
}

/// Face definition for an element
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ElementFace {
    /// UV coordinates [x1, y1, x2, y2] (0-16 scale)
    #[serde(default)]
    pub uv: Option<[f32; 4]>,
    /// Texture variable reference (e.g., "#top")
    pub texture: String,
    /// Face to cull against
    #[serde(default)]
    pub cullface: Option<String>,
    /// Texture rotation (0, 90, 180, 270)
    #[serde(default)]
    pub rotation: Option<i32>,
    /// Tint index for biome coloring
    #[serde(default)]
    pub tintindex: Option<i32>,
}

/// Display transform for different contexts
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisplayTransform {
    /// Rotation [x, y, z] degrees
    #[serde(default)]
    pub rotation: Option<[f32; 3]>,
    /// Translation [x, y, z]
    #[serde(default)]
    pub translation: Option<[f32; 3]>,
    /// Scale [x, y, z]
    #[serde(default)]
    pub scale: Option<[f32; 3]>,
}

impl ModelDefinition {
    /// Load from a JSON file
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read model file: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse model JSON: {}", e))
    }

    /// Get a texture variable value
    pub fn get_texture(&self, var: &str) -> Option<&str> {
        // Remove leading # if present
        let key = var.strip_prefix('#').unwrap_or(var);
        self.textures.get(key).map(|s| s.as_str())
    }

    /// Resolve texture references (following parent chain)
    pub fn resolve_texture(&self, var: &str, model_registry: &ModelRegistry) -> Option<String> {
        let key = var.strip_prefix('#').unwrap_or(var);

        // Check this model's textures
        if let Some(value) = self.textures.get(key) {
            // If it references another variable, resolve recursively
            if value.starts_with('#') {
                return self.resolve_texture(value, model_registry);
            }
            return Some(value.clone());
        }

        // Check parent model
        if let Some(parent_name) = &self.parent {
            if let Some(parent) = model_registry.get(parent_name) {
                return parent.resolve_texture(var, model_registry);
            }
        }

        None
    }
}

/// Face textures for a block (resolved from model)
#[derive(Clone, Debug, Default)]
pub struct FaceTextures {
    pub top: Option<String>,
    pub bottom: Option<String>,
    pub north: Option<String>,
    pub south: Option<String>,
    pub east: Option<String>,
    pub west: Option<String>,
}

impl FaceTextures {
    /// Create with all faces using the same texture
    pub fn all(texture: &str) -> Self {
        let tex = Some(texture.to_string());
        Self {
            top: tex.clone(),
            bottom: tex.clone(),
            north: tex.clone(),
            south: tex.clone(),
            east: tex.clone(),
            west: tex,
        }
    }

    /// Create with top, side, bottom differentiation
    pub fn top_side_bottom(top: &str, side: &str, bottom: &str) -> Self {
        Self {
            top: Some(top.to_string()),
            bottom: Some(bottom.to_string()),
            north: Some(side.to_string()),
            south: Some(side.to_string()),
            east: Some(side.to_string()),
            west: Some(side.to_string()),
        }
    }
}

/// Built-in model types
#[derive(Clone, Debug)]
pub enum BlockModel {
    /// Standard full cube
    Cube,
    /// Cube with all faces same texture (like stone)
    CubeAll,
    /// Cube with different top (like grass)
    CubeTop,
    /// Cube with different top and bottom
    CubeColumn,
    /// Custom model with elements
    Custom(ModelDefinition),
}

impl BlockModel {
    /// Get face textures for this model
    pub fn get_face_textures(&self, textures: &HashMap<String, String>) -> FaceTextures {
        match self {
            BlockModel::Cube | BlockModel::CubeAll => {
                let all = textures.get("all").or(textures.get("texture"));
                FaceTextures::all(all.map(|s| s.as_str()).unwrap_or("missing"))
            }
            BlockModel::CubeTop => {
                let top = textures.get("top").map(|s| s.as_str()).unwrap_or("missing");
                let side = textures
                    .get("side")
                    .map(|s| s.as_str())
                    .unwrap_or("missing");
                FaceTextures::top_side_bottom(top, side, side)
            }
            BlockModel::CubeColumn => {
                let top = textures.get("top").or(textures.get("end"));
                let side = textures.get("side");
                FaceTextures {
                    top: top.cloned(),
                    bottom: top.cloned(),
                    north: side.cloned(),
                    south: side.cloned(),
                    east: side.cloned(),
                    west: side.cloned(),
                }
            }
            BlockModel::Custom(_) => {
                // For custom models, would need to analyze elements
                FaceTextures::default()
            }
        }
    }
}

/// Model registry resource
#[derive(Resource, Default)]
pub struct ModelRegistry {
    /// Loaded model definitions
    definitions: HashMap<String, ModelDefinition>,
    /// Built-in base models
    base_models: HashMap<String, BlockModel>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();

        // Register built-in base models
        registry
            .base_models
            .insert("block/cube".to_string(), BlockModel::Cube);
        registry
            .base_models
            .insert("block/cube_all".to_string(), BlockModel::CubeAll);
        registry
            .base_models
            .insert("block/cube_top".to_string(), BlockModel::CubeTop);
        registry
            .base_models
            .insert("block/cube_column".to_string(), BlockModel::CubeColumn);

        registry
    }

    /// Load all model definitions from a directory
    pub fn load_from_directory(&mut self, path: &Path) {
        self.load_recursive(path, "");
    }

    fn load_recursive(&mut self, base_path: &Path, prefix: &str) {
        if !base_path.exists() {
            return;
        }

        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let file_path = entry.path();

                if file_path.is_dir() {
                    let dir_name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    let new_prefix = if prefix.is_empty() {
                        dir_name.to_string()
                    } else {
                        format!("{}/{}", prefix, dir_name)
                    };
                    self.load_recursive(&file_path, &new_prefix);
                } else if file_path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(name) = file_path.file_stem().and_then(|s| s.to_str()) {
                        let full_name = if prefix.is_empty() {
                            name.to_string()
                        } else {
                            format!("{}/{}", prefix, name)
                        };

                        match ModelDefinition::load_from_file(&file_path) {
                            Ok(def) => {
                                self.definitions.insert(full_name.clone(), def);
                                debug!("Loaded model: {}", full_name);
                            }
                            Err(e) => {
                                warn!("Failed to load model {}: {}", full_name, e);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get a model definition
    pub fn get(&self, name: &str) -> Option<&ModelDefinition> {
        self.definitions.get(name)
    }

    /// Get a base model type
    pub fn get_base_model(&self, name: &str) -> Option<&BlockModel> {
        self.base_models.get(name)
    }

    /// Register a model definition
    pub fn register(&mut self, name: &str, definition: ModelDefinition) {
        self.definitions.insert(name.to_string(), definition);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_definition_parse() {
        let json = r#"{
            "parent": "block/cube_all",
            "textures": {
                "all": "block/stone"
            }
        }"#;

        let def: ModelDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.parent, Some("block/cube_all".to_string()));
        assert_eq!(def.get_texture("all"), Some("block/stone"));
    }

    #[test]
    fn test_face_textures_all() {
        let ft = FaceTextures::all("stone");
        assert_eq!(ft.top, Some("stone".to_string()));
        assert_eq!(ft.north, Some("stone".to_string()));
    }

    #[test]
    fn test_face_textures_top_side_bottom() {
        let ft = FaceTextures::top_side_bottom("grass_top", "grass_side", "dirt");
        assert_eq!(ft.top, Some("grass_top".to_string()));
        assert_eq!(ft.north, Some("grass_side".to_string()));
        assert_eq!(ft.bottom, Some("dirt".to_string()));
    }
}
