//! Texture system for block rendering
//!
//! This module provides:
//! - `TextureAtlas`: Runtime texture atlas with UV region mapping
//! - `BlockTextures`: Per-block texture configuration (all faces, per-face, directional)
//! - `BlockstateDefinition`: JSON-based state-to-model mapping
//! - `ResourcePackManager`: Resource pack loading and override system
//! - `TextureResolver`: Trait for MOD-extensible texture resolution

mod atlas;
mod blockstates;
mod models;
mod resolver;
mod resource_pack;

pub use atlas::{BlockFace, BlockTextureConfig, TextureAtlas, TextureRegistry, UVCache, UVRect};
pub use blockstates::{BlockstateDefinition, BlockstateRegistry, ModelVariant, MultipartCase};
pub use models::{BlockModel, FaceTextures, ModelDefinition};
pub use resolver::{NeighborInfo, TextureResolver, TextureResult};
pub use resource_pack::{ResourcePack, ResourcePackManager};

use bevy::prelude::*;
use std::path::Path;

/// Plugin for the texture system
pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextureRegistry>()
            .init_resource::<ResourcePackManager>()
            .init_resource::<BlockstateRegistry>()
            .add_systems(
                Startup,
                setup_texture_system.in_set(TextureSystemSet::Setup),
            )
            .add_systems(
                Update,
                update_texture_atlas.in_set(TextureSystemSet::Update),
            );
    }
}

/// System set for texture system ordering
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextureSystemSet {
    Setup,
    Update,
}

/// Setup the texture system
fn setup_texture_system(
    mut registry: ResMut<TextureRegistry>,
    mut pack_manager: ResMut<ResourcePackManager>,
    mut blockstate_registry: ResMut<BlockstateRegistry>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    // Load base textures
    registry.load_base_textures(&asset_server, &mut images);

    // Load blockstates from JSON files
    let blockstates_path = Path::new("assets/blockstates");
    blockstate_registry.load_from_directory(blockstates_path);

    // Load resource packs (if any)
    pack_manager.scan_resource_packs();
    pack_manager.apply_to_registry(&mut registry, &asset_server, &mut images);

    info!(
        "Texture system initialized: {} block textures, {} resource packs",
        registry.block_count(),
        pack_manager.pack_count()
    );
}

/// Update texture atlas when resources change or finish loading
fn update_texture_atlas(
    mut registry: ResMut<TextureRegistry>,
    mut images: ResMut<Assets<Image>>,
    mut ev_asset: EventReader<AssetEvent<Image>>,
) {
    let mut needs_rebuild = false;

    for ev in ev_asset.read() {
        match ev {
            // Detect when textures finish loading (initial load)
            AssetEvent::LoadedWithDependencies { id } => {
                if registry.has_pending_texture(*id) {
                    needs_rebuild = true;
                    info!("Texture loaded: {:?}", id);
                }
            }
            // Detect when textures are modified (hot reload)
            AssetEvent::Modified { id } => {
                if registry.needs_rebuild(*id) {
                    needs_rebuild = true;
                }
            }
            _ => {}
        }
    }

    if needs_rebuild {
        info!("Rebuilding texture atlas...");
        registry.rebuild_atlas(&mut images);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uv_rect() {
        let rect = UVRect::new(0.0, 0.0, 0.0625, 0.0625); // 16x16 in 256x256 atlas
        assert!((rect.width() - 0.0625).abs() < 0.0001);
        assert!((rect.height() - 0.0625).abs() < 0.0001);
    }
}
