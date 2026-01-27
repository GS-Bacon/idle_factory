//! Asset loader for Blockbench .bbmodel files

use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;

use super::animation::parse_animations;
use super::animation::parse_outliner;
use super::mesh::generate_mesh;
use super::raw::RawBbmodel;
use super::texture::extract_texture_data;
use super::BlockbenchLoadError;
use super::BlockbenchModel;

/// Blockbench asset loader
#[derive(Default, TypePath)]
pub struct BlockbenchLoader;

impl AssetLoader for BlockbenchLoader {
    type Asset = BlockbenchModel;
    type Settings = ();
    type Error = BlockbenchLoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|e| BlockbenchLoadError::Io(e.to_string()))?;

        let raw: RawBbmodel =
            serde_json::from_slice(&bytes).map_err(|e| BlockbenchLoadError::Json(e.to_string()))?;

        let name = if raw.name.is_empty() {
            load_context
                .path()
                .path()
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unnamed")
                .to_string()
        } else {
            raw.name
        };

        let resolution = UVec2::new(raw.resolution.width, raw.resolution.height);

        // Generate mesh from elements
        let mesh = generate_mesh(&raw.elements, resolution)?;

        // Try to extract embedded texture data
        let texture_data = extract_texture_data(&raw.textures);

        // Parse bone hierarchy from outliner
        let bones = parse_outliner(&raw.outliner, None);

        // Parse animations
        let animations = parse_animations(&raw.animations);

        tracing::info!(
            "Loaded bbmodel: {} ({} elements, {} bones, {} animations, {}x{} resolution)",
            name,
            raw.elements.len(),
            bones.len(),
            animations.len(),
            resolution.x,
            resolution.y
        );

        Ok(BlockbenchModel {
            name,
            resolution,
            mesh,
            texture_data,
            bones,
            animations,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bbmodel"]
    }
}
