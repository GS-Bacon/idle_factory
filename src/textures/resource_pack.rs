//! Resource pack management
//!
//! Allows users and MODs to override textures without modifying base files.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::atlas::TextureRegistry;

/// Resource pack metadata (loaded from pack.toml)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackMetadata {
    /// Pack identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Version string
    pub version: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Author(s)
    #[serde(default)]
    pub authors: Vec<String>,
    /// Minimum game version required
    #[serde(default)]
    pub min_game_version: Option<String>,
}

/// A loaded resource pack
#[derive(Clone, Debug)]
pub struct ResourcePack {
    /// Pack metadata
    pub metadata: PackMetadata,
    /// Root directory of the pack
    pub root_path: PathBuf,
    /// Whether the pack is enabled
    pub enabled: bool,
    /// Load order priority (higher = loaded later, overrides earlier)
    pub priority: i32,
}

impl ResourcePack {
    /// Load a resource pack from a directory
    pub fn load(path: &Path) -> Result<Self, String> {
        let metadata_path = path.join("pack.toml");
        if !metadata_path.exists() {
            return Err(format!("pack.toml not found in {:?}", path));
        }

        let content = fs::read_to_string(&metadata_path)
            .map_err(|e| format!("Failed to read pack.toml: {}", e))?;

        let metadata: PackMetadata =
            toml::from_str(&content).map_err(|e| format!("Failed to parse pack.toml: {}", e))?;

        Ok(Self {
            metadata,
            root_path: path.to_path_buf(),
            enabled: true,
            priority: 0,
        })
    }

    /// Find a texture file in this pack
    pub fn find_texture(&self, name: &str) -> Option<PathBuf> {
        // Try different texture locations
        let candidates = [
            self.root_path
                .join(format!("assets/textures/blocks/{}.png", name)),
            self.root_path.join(format!("textures/blocks/{}.png", name)),
            self.root_path.join(format!("textures/{}.png", name)),
        ];

        candidates.into_iter().find(|p| p.exists())
    }

    /// Find a model file in this pack
    pub fn find_model(&self, name: &str) -> Option<PathBuf> {
        let candidates = [
            self.root_path.join(format!("assets/models/{}.json", name)),
            self.root_path.join(format!("models/{}.json", name)),
        ];

        candidates.into_iter().find(|p| p.exists())
    }

    /// Find a blockstate file in this pack
    pub fn find_blockstate(&self, name: &str) -> Option<PathBuf> {
        let candidates = [
            self.root_path
                .join(format!("assets/blockstates/{}.json", name)),
            self.root_path.join(format!("blockstates/{}.json", name)),
        ];

        candidates.into_iter().find(|p| p.exists())
    }

    /// List all textures in this pack
    pub fn list_textures(&self) -> Vec<(String, PathBuf)> {
        let mut textures = Vec::new();
        let texture_dirs = [
            self.root_path.join("assets/textures/blocks"),
            self.root_path.join("textures/blocks"),
            self.root_path.join("textures"),
        ];

        for dir in texture_dirs {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("png") {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            textures.push((name.to_string(), path));
                        }
                    }
                }
            }
        }

        textures
    }
}

/// Resource pack manager
#[derive(Resource, Default)]
pub struct ResourcePackManager {
    /// Loaded resource packs (in priority order)
    packs: Vec<ResourcePack>,
    /// Base resource packs directory
    resource_packs_dir: PathBuf,
    /// Texture overrides (texture name -> pack providing it)
    texture_overrides: HashMap<String, String>,
}

impl ResourcePackManager {
    pub fn new() -> Self {
        Self {
            packs: Vec::new(),
            resource_packs_dir: PathBuf::from("resource_packs"),
            texture_overrides: HashMap::new(),
        }
    }

    /// Set the resource packs directory
    pub fn set_directory(&mut self, path: PathBuf) {
        self.resource_packs_dir = path;
    }

    /// Scan for resource packs in the directory
    pub fn scan_resource_packs(&mut self) {
        self.packs.clear();

        if !self.resource_packs_dir.exists() {
            info!(
                "Resource packs directory not found, creating: {:?}",
                self.resource_packs_dir
            );
            if let Err(e) = fs::create_dir_all(&self.resource_packs_dir) {
                warn!("Failed to create resource packs directory: {}", e);
            }
            return;
        }

        if let Ok(entries) = fs::read_dir(&self.resource_packs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    match ResourcePack::load(&path) {
                        Ok(pack) => {
                            info!(
                                "Found resource pack: {} v{}",
                                pack.metadata.name, pack.metadata.version
                            );
                            self.packs.push(pack);
                        }
                        Err(e) => {
                            debug!("Skipping {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        // Sort by priority
        self.packs.sort_by(|a, b| a.priority.cmp(&b.priority));
    }

    /// Apply resource pack overrides to the texture registry
    pub fn apply_to_registry(
        &mut self,
        registry: &mut TextureRegistry,
        asset_server: &AssetServer,
        _images: &mut Assets<Image>,
    ) {
        self.texture_overrides.clear();

        for pack in &self.packs {
            if !pack.enabled {
                continue;
            }

            for (name, path) in pack.list_textures() {
                // Load the texture
                let handle: Handle<Image> = asset_server.load(path.clone());

                // Store override info
                self.texture_overrides
                    .insert(name.clone(), pack.metadata.id.clone());

                debug!(
                    "Texture override: {} from pack '{}'",
                    name, pack.metadata.name
                );

                // The texture will be picked up when the atlas is rebuilt
                let _ = handle;
            }
        }

        // Rebuild atlas if we have overrides
        if !self.texture_overrides.is_empty() {
            info!("Applied {} texture overrides", self.texture_overrides.len());
            registry.needs_rebuild = true;
        }
    }

    /// Resolve a texture path (checking packs first, then base)
    pub fn resolve_texture(&self, name: &str) -> Option<PathBuf> {
        // Check packs in reverse priority order (highest priority last wins)
        for pack in self.packs.iter().rev() {
            if pack.enabled {
                if let Some(path) = pack.find_texture(name) {
                    return Some(path);
                }
            }
        }

        // Fall back to base game texture
        let base_path = PathBuf::from(format!("assets/textures/blocks/{}.png", name));
        if base_path.exists() {
            return Some(base_path);
        }

        None
    }

    /// Get the number of loaded packs
    pub fn pack_count(&self) -> usize {
        self.packs.len()
    }

    /// Get pack info for UI display
    pub fn get_packs(&self) -> &[ResourcePack] {
        &self.packs
    }

    /// Enable/disable a pack by ID
    pub fn set_pack_enabled(&mut self, pack_id: &str, enabled: bool) {
        for pack in &mut self.packs {
            if pack.metadata.id == pack_id {
                pack.enabled = enabled;
                break;
            }
        }
    }

    /// Set pack priority
    pub fn set_pack_priority(&mut self, pack_id: &str, priority: i32) {
        for pack in &mut self.packs {
            if pack.metadata.id == pack_id {
                pack.priority = priority;
                break;
            }
        }
        // Re-sort
        self.packs.sort_by(|a, b| a.priority.cmp(&b.priority));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_pack_metadata_parse() {
        let toml = r#"
            id = "test_pack"
            name = "Test Pack"
            version = "1.0.0"
            description = "A test resource pack"
            authors = ["Test Author"]
        "#;

        let metadata: PackMetadata = toml::from_str(toml).unwrap();
        assert_eq!(metadata.id, "test_pack");
        assert_eq!(metadata.name, "Test Pack");
        assert_eq!(metadata.version, "1.0.0");
    }

    #[test]
    fn test_resource_pack_load() {
        let dir = tempdir().unwrap();
        let pack_path = dir.path().join("test_pack");
        fs::create_dir(&pack_path).unwrap();

        let mut file = fs::File::create(pack_path.join("pack.toml")).unwrap();
        write!(
            file,
            r#"
            id = "test"
            name = "Test"
            version = "1.0.0"
        "#
        )
        .unwrap();

        let pack = ResourcePack::load(&pack_path).unwrap();
        assert_eq!(pack.metadata.id, "test");
    }
}
