//! Texture atlas management
//!
//! Handles runtime atlas generation and UV coordinate mapping.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::HashMap;

use crate::core::{items, ItemId};

/// UV rectangle in normalized coordinates (0.0-1.0)
#[derive(Clone, Copy, Debug, Default)]
pub struct UVRect {
    pub min: Vec2,
    pub max: Vec2,
}

impl UVRect {
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
        }
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// Get UV coordinates for a quad, accounting for tiling
    /// Returns [bottom-left, bottom-right, top-right, top-left]
    pub fn get_uvs(&self, tile_x: f32, tile_y: f32) -> [[f32; 2]; 4] {
        let w = self.width() * tile_x;
        let h = self.height() * tile_y;
        [
            [self.min.x, self.min.y],
            [self.min.x + w, self.min.y],
            [self.min.x + w, self.min.y + h],
            [self.min.x, self.min.y + h],
        ]
    }
}

/// Block face enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockFace {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

impl BlockFace {
    pub fn all() -> [BlockFace; 6] {
        [
            BlockFace::Top,
            BlockFace::Bottom,
            BlockFace::North,
            BlockFace::South,
            BlockFace::East,
            BlockFace::West,
        ]
    }

    pub fn is_side(&self) -> bool {
        matches!(
            self,
            BlockFace::North | BlockFace::South | BlockFace::East | BlockFace::West
        )
    }
}

/// Texture configuration for a single block
#[derive(Clone, Debug)]
pub enum BlockTextureConfig {
    /// All faces use the same texture
    All(String),
    /// Different textures for top, sides, and bottom
    TopSideBottom {
        top: String,
        side: String,
        bottom: String,
    },
    /// Each face has its own texture (for directional blocks)
    PerFace {
        top: String,
        bottom: String,
        north: String,
        south: String,
        east: String,
        west: String,
    },
}

impl BlockTextureConfig {
    pub fn get_texture(&self, face: BlockFace) -> &str {
        match self {
            BlockTextureConfig::All(tex) => tex,
            BlockTextureConfig::TopSideBottom { top, side, bottom } => match face {
                BlockFace::Top => top,
                BlockFace::Bottom => bottom,
                _ => side,
            },
            BlockTextureConfig::PerFace {
                top,
                bottom,
                north,
                south,
                east,
                west,
            } => match face {
                BlockFace::Top => top,
                BlockFace::Bottom => bottom,
                BlockFace::North => north,
                BlockFace::South => south,
                BlockFace::East => east,
                BlockFace::West => west,
            },
        }
    }

    /// Get all unique texture names used by this config
    pub fn texture_names(&self) -> Vec<&str> {
        match self {
            BlockTextureConfig::All(tex) => vec![tex.as_str()],
            BlockTextureConfig::TopSideBottom { top, side, bottom } => {
                let mut names = vec![top.as_str(), side.as_str(), bottom.as_str()];
                names.sort();
                names.dedup();
                names
            }
            BlockTextureConfig::PerFace {
                top,
                bottom,
                north,
                south,
                east,
                west,
            } => {
                let mut names = vec![
                    top.as_str(),
                    bottom.as_str(),
                    north.as_str(),
                    south.as_str(),
                    east.as_str(),
                    west.as_str(),
                ];
                names.sort();
                names.dedup();
                names
            }
        }
    }
}

/// Texture atlas with UV mapping
#[derive(Clone, Debug)]
pub struct TextureAtlas {
    /// Handle to the atlas image
    pub image: Handle<Image>,
    /// UV regions for each texture name
    pub regions: HashMap<String, UVRect>,
    /// Atlas size in pixels
    pub size: UVec2,
    /// Individual texture size (typically 16x16)
    pub tile_size: u32,
}

impl TextureAtlas {
    pub fn new(image: Handle<Image>, size: UVec2, tile_size: u32) -> Self {
        Self {
            image,
            regions: HashMap::new(),
            size,
            tile_size,
        }
    }

    /// Get UV rect for a texture name
    pub fn get_uv(&self, name: &str) -> Option<UVRect> {
        self.regions.get(name).copied()
    }

    /// Add a texture region
    pub fn add_region(&mut self, name: String, grid_x: u32, grid_y: u32) {
        let tile_uv = self.tile_size as f32 / self.size.x as f32;
        let rect = UVRect::new(
            grid_x as f32 * tile_uv,
            grid_y as f32 * tile_uv,
            (grid_x + 1) as f32 * tile_uv,
            (grid_y + 1) as f32 * tile_uv,
        );
        self.regions.insert(name, rect);
    }
}

/// Cached UV data for async mesh generation (Send + Sync safe)
#[derive(Clone, Debug, Default)]
pub struct UVCache {
    /// Map from (ItemId, face_index) to UVRect
    /// face_index: 0=Top, 1=Bottom, 2=North, 3=South, 4=East, 5=West
    pub uvs: HashMap<(ItemId, u8), UVRect>,
}

impl UVCache {
    /// Get UV rect for a block face
    pub fn get(&self, item_id: ItemId, face_idx: u8) -> Option<UVRect> {
        self.uvs.get(&(item_id, face_idx)).copied()
    }

    /// Get UV rect with default fallback
    pub fn get_or_default(&self, item_id: ItemId, face_idx: u8) -> UVRect {
        self.get(item_id, face_idx).unwrap_or_default()
    }

    /// Check if the cache has any entries
    pub fn is_empty(&self) -> bool {
        self.uvs.is_empty()
    }
}

/// Main texture registry resource
#[derive(Resource, Default)]
pub struct TextureRegistry {
    /// Block texture atlas
    pub block_atlas: Option<TextureAtlas>,
    /// Block texture configurations (ItemId -> config)
    pub block_textures: HashMap<ItemId, BlockTextureConfig>,
    /// Individual texture handles for atlas building
    texture_handles: HashMap<String, Handle<Image>>,
    /// Source image IDs for change detection
    source_images: Vec<AssetId<Image>>,
    /// Whether atlas needs rebuilding
    pub needs_rebuild: bool,
}

impl TextureRegistry {
    /// Load base game textures
    pub fn load_base_textures(&mut self, asset_server: &AssetServer, images: &mut Assets<Image>) {
        // Define block texture configurations
        self.register_block_texture(items::stone(), BlockTextureConfig::All("stone".to_string()));
        self.register_block_texture(
            items::grass(),
            BlockTextureConfig::TopSideBottom {
                top: "grass_top".to_string(),
                side: "grass_side".to_string(),
                bottom: "dirt".to_string(),
            },
        );
        self.register_block_texture(
            items::iron_ore(),
            BlockTextureConfig::All("iron_ore".to_string()),
        );
        self.register_block_texture(
            items::copper_ore(),
            BlockTextureConfig::All("copper_ore".to_string()),
        );
        self.register_block_texture(
            items::coal(),
            BlockTextureConfig::All("coal_ore".to_string()),
        );

        // Load texture files
        let texture_files = [
            "stone",
            "grass_top",
            "grass_side",
            "dirt",
            "iron_ore",
            "copper_ore",
            "coal_ore",
            "cobblestone",
            "gravel",
            "sandstone",
            "sand",
            "bedrock",
        ];

        for name in texture_files {
            let path = format!("textures/blocks/{}.png", name);
            let handle: Handle<Image> = asset_server.load(&path);
            self.texture_handles.insert(name.to_string(), handle);
        }

        // Build initial atlas
        self.build_atlas(images);
    }

    /// Register a block's texture configuration
    pub fn register_block_texture(&mut self, item_id: ItemId, config: BlockTextureConfig) {
        self.block_textures.insert(item_id, config);
        self.needs_rebuild = true;
    }

    /// Get texture config for a block
    pub fn get_block_config(&self, item_id: ItemId) -> Option<&BlockTextureConfig> {
        self.block_textures.get(&item_id)
    }

    /// Get UV rect for a block face
    pub fn get_block_uv(&self, item_id: ItemId, face: BlockFace) -> Option<UVRect> {
        let config = self.block_textures.get(&item_id)?;
        let texture_name = config.get_texture(face);
        self.block_atlas.as_ref()?.get_uv(texture_name)
    }

    /// Get the atlas image handle
    pub fn atlas_image(&self) -> Option<Handle<Image>> {
        self.block_atlas.as_ref().map(|a| a.image.clone())
    }

    /// Number of registered blocks
    pub fn block_count(&self) -> usize {
        self.block_textures.len()
    }

    /// Check if an image change requires atlas rebuild
    pub fn needs_rebuild(&self, id: AssetId<Image>) -> bool {
        self.source_images.contains(&id)
    }

    /// Rebuild the atlas from source images
    pub fn rebuild_atlas(&mut self, images: &mut Assets<Image>) {
        self.build_atlas(images);
    }

    /// Build the texture atlas from loaded images
    fn build_atlas(&mut self, images: &mut Assets<Image>) {
        const ATLAS_SIZE: u32 = 256;
        const TILE_SIZE: u32 = 16;
        const TILES_PER_ROW: u32 = ATLAS_SIZE / TILE_SIZE;

        // Create atlas image
        let mut atlas_data = vec![0u8; (ATLAS_SIZE * ATLAS_SIZE * 4) as usize];

        // Collect all unique texture names
        let mut texture_names: Vec<String> = self
            .block_textures
            .values()
            .flat_map(|c| c.texture_names())
            .map(|s| s.to_string())
            .collect();
        texture_names.sort();
        texture_names.dedup();

        // Create atlas with placeholder colors for now
        let mut atlas = TextureAtlas::new(Handle::default(), UVec2::splat(ATLAS_SIZE), TILE_SIZE);

        for (idx, name) in texture_names.iter().enumerate() {
            let grid_x = (idx as u32) % TILES_PER_ROW;
            let grid_y = (idx as u32) / TILES_PER_ROW;

            // Try to copy actual texture data, otherwise use fallback color
            if let Some(handle) = self.texture_handles.get(name) {
                if let Some(source_image) = images.get(handle) {
                    // Copy texture to atlas
                    self.copy_texture_to_atlas(
                        source_image,
                        &mut atlas_data,
                        ATLAS_SIZE,
                        grid_x * TILE_SIZE,
                        grid_y * TILE_SIZE,
                        TILE_SIZE,
                    );
                } else {
                    // Texture not loaded yet, use fallback
                    self.fill_tile_with_color(
                        &mut atlas_data,
                        ATLAS_SIZE,
                        grid_x * TILE_SIZE,
                        grid_y * TILE_SIZE,
                        TILE_SIZE,
                        self.get_fallback_color(name),
                    );
                }
            } else {
                // No handle, use fallback
                self.fill_tile_with_color(
                    &mut atlas_data,
                    ATLAS_SIZE,
                    grid_x * TILE_SIZE,
                    grid_y * TILE_SIZE,
                    TILE_SIZE,
                    self.get_fallback_color(name),
                );
            }

            atlas.add_region(name.clone(), grid_x, grid_y);
        }

        // Create the atlas image
        let atlas_image = Image::new(
            Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            atlas_data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );

        atlas.image = images.add(atlas_image);
        self.block_atlas = Some(atlas);
        self.needs_rebuild = false;

        info!("Built texture atlas with {} textures", texture_names.len());
    }

    /// Copy a source texture to the atlas
    fn copy_texture_to_atlas(
        &self,
        source: &Image,
        atlas_data: &mut [u8],
        atlas_size: u32,
        dest_x: u32,
        dest_y: u32,
        tile_size: u32,
    ) {
        let src_width = source.width();
        let src_height = source.height();

        for y in 0..tile_size.min(src_height) {
            for x in 0..tile_size.min(src_width) {
                let src_idx = ((y * src_width + x) * 4) as usize;
                let dest_idx = (((dest_y + y) * atlas_size + dest_x + x) * 4) as usize;

                if src_idx + 3 < source.data.len() && dest_idx + 3 < atlas_data.len() {
                    atlas_data[dest_idx] = source.data[src_idx];
                    atlas_data[dest_idx + 1] = source.data[src_idx + 1];
                    atlas_data[dest_idx + 2] = source.data[src_idx + 2];
                    atlas_data[dest_idx + 3] = source.data[src_idx + 3];
                }
            }
        }
    }

    /// Fill a tile with a solid color
    fn fill_tile_with_color(
        &self,
        atlas_data: &mut [u8],
        atlas_size: u32,
        dest_x: u32,
        dest_y: u32,
        tile_size: u32,
        color: [u8; 4],
    ) {
        for y in 0..tile_size {
            for x in 0..tile_size {
                let idx = (((dest_y + y) * atlas_size + dest_x + x) * 4) as usize;
                if idx + 3 < atlas_data.len() {
                    atlas_data[idx] = color[0];
                    atlas_data[idx + 1] = color[1];
                    atlas_data[idx + 2] = color[2];
                    atlas_data[idx + 3] = color[3];
                }
            }
        }
    }

    /// Export UV cache for use in async mesh generation
    /// Returns a HashMap where key is (ItemId, BlockFace index) and value is UVRect
    pub fn export_uv_cache(&self) -> UVCache {
        let mut cache = HashMap::new();
        if let Some(atlas) = &self.block_atlas {
            for (&item_id, config) in &self.block_textures {
                for (face_idx, face) in BlockFace::all().iter().enumerate() {
                    let texture_name = config.get_texture(*face);
                    if let Some(uv) = atlas.get_uv(texture_name) {
                        cache.insert((item_id, face_idx as u8), uv);
                    }
                }
            }
        }
        UVCache { uvs: cache }
    }

    /// Get fallback color for a texture name
    fn get_fallback_color(&self, name: &str) -> [u8; 4] {
        match name {
            "stone" => [128, 128, 128, 255],
            "grass_top" => [50, 200, 50, 255],
            "grass_side" => [100, 150, 70, 255],
            "dirt" => [139, 90, 43, 255],
            "iron_ore" => [150, 130, 100, 255],
            "copper_ore" => [180, 100, 70, 255],
            "coal_ore" => [40, 40, 40, 255],
            "cobblestone" => [100, 100, 100, 255],
            "gravel" => [130, 130, 130, 255],
            "sandstone" => [220, 200, 150, 255],
            "sand" => [240, 220, 180, 255],
            "bedrock" => [50, 50, 50, 255],
            _ => [255, 0, 255, 255], // Magenta for missing
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_texture_config_all() {
        let config = BlockTextureConfig::All("stone".to_string());
        assert_eq!(config.get_texture(BlockFace::Top), "stone");
        assert_eq!(config.get_texture(BlockFace::North), "stone");
    }

    #[test]
    fn test_block_texture_config_top_side_bottom() {
        let config = BlockTextureConfig::TopSideBottom {
            top: "grass_top".to_string(),
            side: "grass_side".to_string(),
            bottom: "dirt".to_string(),
        };
        assert_eq!(config.get_texture(BlockFace::Top), "grass_top");
        assert_eq!(config.get_texture(BlockFace::North), "grass_side");
        assert_eq!(config.get_texture(BlockFace::Bottom), "dirt");
    }

    #[test]
    fn test_uv_rect_get_uvs() {
        let rect = UVRect::new(0.0, 0.0, 0.0625, 0.0625);
        let uvs = rect.get_uvs(1.0, 1.0);
        assert_eq!(uvs[0], [0.0, 0.0]);
        assert_eq!(uvs[2], [0.0625, 0.0625]);
    }
}
