//! Blockstate definitions for state-to-model mapping
//!
//! Based on Minecraft's blockstates JSON format.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Blockstate definition (loaded from JSON)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockstateDefinition {
    /// Variant-based definitions (mutually exclusive states)
    #[serde(default)]
    pub variants: Option<HashMap<String, ModelVariantList>>,
    /// Multipart definitions (combinable parts)
    #[serde(default)]
    pub multipart: Option<Vec<MultipartCase>>,
}

/// A list of model variants (for random selection)
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModelVariantList {
    Single(ModelVariant),
    Multiple(Vec<ModelVariant>),
}

impl ModelVariantList {
    /// Get a variant (first one for now, could add randomization)
    pub fn get_variant(&self) -> &ModelVariant {
        match self {
            ModelVariantList::Single(v) => v,
            ModelVariantList::Multiple(list) => list.first().unwrap_or(&DEFAULT_VARIANT),
        }
    }

    /// Get all variants
    pub fn variants(&self) -> Vec<&ModelVariant> {
        match self {
            ModelVariantList::Single(v) => vec![v],
            ModelVariantList::Multiple(list) => list.iter().collect(),
        }
    }
}

static DEFAULT_VARIANT: ModelVariant = ModelVariant {
    model: String::new(),
    x: None,
    y: None,
    uvlock: None,
    weight: None,
};

/// A single model variant
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelVariant {
    /// Model path (e.g., "block/stone")
    pub model: String,
    /// X-axis rotation (0, 90, 180, 270)
    #[serde(default)]
    pub x: Option<i32>,
    /// Y-axis rotation (0, 90, 180, 270)
    #[serde(default)]
    pub y: Option<i32>,
    /// Lock UV coordinates when rotated
    #[serde(default)]
    pub uvlock: Option<bool>,
    /// Weight for random selection
    #[serde(default)]
    pub weight: Option<u32>,
}

impl ModelVariant {
    pub fn new(model: &str) -> Self {
        Self {
            model: model.to_string(),
            x: None,
            y: None,
            uvlock: None,
            weight: None,
        }
    }

    pub fn with_rotation(mut self, x: i32, y: i32) -> Self {
        self.x = Some(x);
        self.y = Some(y);
        self
    }

    /// Get rotation as a quaternion
    pub fn rotation(&self) -> Quat {
        let x_rot = Quat::from_rotation_x((self.x.unwrap_or(0) as f32).to_radians());
        let y_rot = Quat::from_rotation_y((self.y.unwrap_or(0) as f32).to_radians());
        y_rot * x_rot
    }
}

/// Multipart case (conditional model application)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultipartCase {
    /// Condition for this case
    #[serde(default)]
    pub when: Option<MultipartCondition>,
    /// Model to apply
    pub apply: ModelVariantList,
}

/// Condition for multipart cases
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MultipartCondition {
    /// Simple key-value conditions
    Simple(HashMap<String, String>),
    /// OR condition (any of the sub-conditions)
    Or {
        #[serde(rename = "OR")]
        or: Vec<HashMap<String, String>>,
    },
}

impl MultipartCondition {
    /// Check if condition matches the given state
    pub fn matches(&self, state: &HashMap<String, String>) -> bool {
        match self {
            MultipartCondition::Simple(conditions) => {
                conditions.iter().all(|(k, v)| state.get(k) == Some(v))
            }
            MultipartCondition::Or { or } => or
                .iter()
                .any(|cond| cond.iter().all(|(k, v)| state.get(k) == Some(v))),
        }
    }
}

impl BlockstateDefinition {
    /// Load from a JSON file
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read blockstate file: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse blockstate JSON: {}", e))
    }

    /// Get model variant for a given state
    pub fn get_model(&self, state: &HashMap<String, String>) -> Option<&ModelVariant> {
        // Try variants first
        if let Some(variants) = &self.variants {
            let state_key = Self::state_to_key(state);
            if let Some(variant_list) = variants.get(&state_key) {
                return Some(variant_list.get_variant());
            }
            // Try empty key for default
            if let Some(variant_list) = variants.get("") {
                return Some(variant_list.get_variant());
            }
        }
        None
    }

    /// Get all applicable models for multipart
    pub fn get_multipart_models(&self, state: &HashMap<String, String>) -> Vec<&ModelVariant> {
        let mut models = Vec::new();
        if let Some(multipart) = &self.multipart {
            for case in multipart {
                let applies = match &case.when {
                    None => true,
                    Some(condition) => condition.matches(state),
                };
                if applies {
                    models.push(case.apply.get_variant());
                }
            }
        }
        models
    }

    /// Convert state map to key string (e.g., "facing=north,half=bottom")
    fn state_to_key(state: &HashMap<String, String>) -> String {
        let mut pairs: Vec<_> = state.iter().collect();
        pairs.sort_by_key(|(k, _)| *k);
        pairs
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// Blockstate registry resource
#[derive(Resource, Default)]
pub struct BlockstateRegistry {
    /// Loaded blockstate definitions
    definitions: HashMap<String, BlockstateDefinition>,
}

impl BlockstateRegistry {
    /// Load all blockstate definitions from a directory
    pub fn load_from_directory(&mut self, path: &Path) {
        if !path.exists() {
            warn!("Blockstates directory not found: {:?}", path);
            return;
        }

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(name) = file_path.file_stem().and_then(|s| s.to_str()) {
                        match BlockstateDefinition::load_from_file(&file_path) {
                            Ok(def) => {
                                self.definitions.insert(name.to_string(), def);
                                info!("Loaded blockstate: {}", name);
                            }
                            Err(e) => {
                                warn!("Failed to load blockstate {}: {}", name, e);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get a blockstate definition
    pub fn get(&self, name: &str) -> Option<&BlockstateDefinition> {
        self.definitions.get(name)
    }

    /// Register a blockstate definition
    pub fn register(&mut self, name: &str, definition: BlockstateDefinition) {
        self.definitions.insert(name.to_string(), definition);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockstate_variants() {
        let json = r#"{
            "variants": {
                "facing=north": { "model": "block/furnace", "y": 0 },
                "facing=east": { "model": "block/furnace", "y": 90 },
                "facing=south": { "model": "block/furnace", "y": 180 },
                "facing=west": { "model": "block/furnace", "y": 270 }
            }
        }"#;

        let def: BlockstateDefinition = serde_json::from_str(json).unwrap();

        let mut state = HashMap::new();
        state.insert("facing".to_string(), "east".to_string());

        let model = def.get_model(&state).unwrap();
        assert_eq!(model.model, "block/furnace");
        assert_eq!(model.y, Some(90));
    }

    #[test]
    fn test_multipart_condition() {
        let json = r#"{
            "multipart": [
                { "apply": { "model": "block/fence_post" } },
                { "when": { "north": "true" }, "apply": { "model": "block/fence_side" } },
                { "when": { "east": "true" }, "apply": { "model": "block/fence_side", "y": 90 } }
            ]
        }"#;

        let def: BlockstateDefinition = serde_json::from_str(json).unwrap();

        let mut state = HashMap::new();
        state.insert("north".to_string(), "true".to_string());
        state.insert("east".to_string(), "false".to_string());

        let models = def.get_multipart_models(&state);
        assert_eq!(models.len(), 2); // post + north side
    }
}
