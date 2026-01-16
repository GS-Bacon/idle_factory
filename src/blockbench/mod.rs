//! Blockbench .bbmodel file loader
//!
//! Loads Blockbench project files (.bbmodel) and converts them to Bevy meshes.
//! Supports animations via keyframe data.

use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use std::fmt;

mod animation;
mod loader;
mod mesh;
mod raw;
mod texture;

// Re-exports
pub use animation::{
    Animation, AnimationChannel, Bone, BoneChild, Interpolation, Keyframe, LoopMode,
};
pub use loader::BlockbenchLoader;
pub use texture::TextureData;

/// Plugin for loading Blockbench .bbmodel files
pub struct BlockbenchPlugin;

impl Plugin for BlockbenchPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<BlockbenchModel>()
            .init_asset_loader::<BlockbenchLoader>();
    }
}

/// Blockbench model asset
#[derive(Asset, TypePath, Debug)]
pub struct BlockbenchModel {
    /// Model name
    pub name: String,
    /// Texture resolution (width, height)
    pub resolution: UVec2,
    /// Generated mesh from elements
    pub mesh: Mesh,
    /// Embedded texture data (raw PNG/JPEG bytes)
    /// Use `create_texture()` to convert to a Bevy Image
    pub texture_data: Option<TextureData>,
    /// Bone hierarchy from outliner
    pub bones: Vec<Bone>,
    /// Animations
    pub animations: Vec<Animation>,
}

impl BlockbenchModel {
    /// Create a Bevy Image from embedded texture data
    ///
    /// Returns `None` if no texture data is embedded in the model.
    pub fn create_texture(&self) -> Option<Image> {
        let texture_data = self.texture_data.as_ref()?;

        // Determine image format from PNG magic bytes
        let image_type = if texture_data
            .raw_bytes
            .starts_with(&[0x89, 0x50, 0x4E, 0x47])
        {
            ImageType::Extension("png")
        } else if texture_data.raw_bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            ImageType::Extension("jpg")
        } else {
            tracing::warn!("Unknown embedded texture format in bbmodel");
            return None;
        };

        match Image::from_buffer(
            &texture_data.raw_bytes,
            image_type,
            CompressedImageFormats::NONE,
            true, // is_srgb
            ImageSampler::Default,
            RenderAssetUsages::RENDER_WORLD,
        ) {
            Ok(image) => Some(image),
            Err(e) => {
                tracing::warn!("Failed to decode bbmodel texture: {:?}", e);
                None
            }
        }
    }
}

/// Error type for Blockbench loading
#[derive(Debug)]
pub enum BlockbenchLoadError {
    Io(String),
    Json(String),
    #[allow(dead_code)]
    Invalid(String),
}

impl fmt::Display for BlockbenchLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockbenchLoadError::Io(msg) => write!(f, "IO error: {}", msg),
            BlockbenchLoadError::Json(msg) => write!(f, "JSON parse error: {}", msg),
            BlockbenchLoadError::Invalid(msg) => write!(f, "Invalid model: {}", msg),
        }
    }
}

impl std::error::Error for BlockbenchLoadError {}

/// Helper function to spawn a BlockbenchModel as an entity with a simple material
pub fn spawn_bbmodel(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    model: &BlockbenchModel,
    transform: Transform,
) -> Entity {
    let mesh_handle = meshes.add(model.mesh.clone());

    // Create material with embedded texture if available
    let material = if let Some(image) = model.create_texture() {
        let texture_handle = images.add(image);
        StandardMaterial {
            base_color_texture: Some(texture_handle),
            ..default()
        }
    } else {
        StandardMaterial {
            base_color: Color::WHITE,
            ..default()
        }
    };
    let material_handle = materials.add(material);

    commands
        .spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            transform,
            Name::new(model.name.clone()),
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockbench::raw::RawBbmodel;
    use crate::blockbench::texture::base64_decode;
    use bevy::render::mesh::PrimitiveTopology;

    #[test]
    fn test_parse_simple_bbmodel() {
        let json = r#"{
            "name": "test_cube",
            "meta": {
                "format_version": "4.10",
                "model_format": "free"
            },
            "resolution": {
                "width": 16,
                "height": 16
            },
            "elements": [
                {
                    "uuid": "abc123",
                    "name": "cube1",
                    "type": "cube",
                    "from": [0, 0, 0],
                    "to": [16, 16, 16],
                    "origin": [8, 8, 8],
                    "rotation": [0, 0, 0],
                    "faces": {
                        "north": {"uv": [0, 0, 16, 16], "texture": 0},
                        "south": {"uv": [0, 0, 16, 16], "texture": 0},
                        "east": {"uv": [0, 0, 16, 16], "texture": 0},
                        "west": {"uv": [0, 0, 16, 16], "texture": 0},
                        "up": {"uv": [0, 0, 16, 16], "texture": 0},
                        "down": {"uv": [0, 0, 16, 16], "texture": 0}
                    }
                }
            ],
            "textures": []
        }"#;

        let raw: RawBbmodel = serde_json::from_str(json).unwrap();
        assert_eq!(raw.name, "test_cube");
        assert_eq!(raw.elements.len(), 1);
        assert_eq!(raw.elements[0].from, [0.0, 0.0, 0.0]);
        assert_eq!(raw.elements[0].to, [16.0, 16.0, 16.0]);
    }

    #[test]
    fn test_parse_test_cube_file() {
        let path = std::path::Path::new("assets/models/blockbench/test_cube.bbmodel");
        if !path.exists() {
            // Skip if test file doesn't exist
            return;
        }
        let bytes = std::fs::read(path).unwrap();
        let raw: RawBbmodel = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(raw.name, "test_cube");
        assert_eq!(raw.elements.len(), 1);
        assert_eq!(raw.resolution.width, 16);
        assert_eq!(raw.resolution.height, 16);

        // Generate mesh and verify
        let mesh = mesh::generate_mesh(
            &raw.elements,
            UVec2::new(raw.resolution.width, raw.resolution.height),
        )
        .unwrap();
        let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let pos_count = match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => 0,
        };
        assert_eq!(pos_count, 24); // 6 faces * 4 vertices
    }

    #[test]
    fn test_parse_test_machine_file() {
        let path = std::path::Path::new("assets/models/blockbench/test_machine.bbmodel");
        if !path.exists() {
            return;
        }
        let bytes = std::fs::read(path).unwrap();
        let raw: RawBbmodel = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(raw.name, "test_machine");
        assert_eq!(raw.elements.len(), 3); // base, body, arm
        assert_eq!(raw.resolution.width, 32);
        assert_eq!(raw.resolution.height, 32);

        // Generate mesh and verify
        let mesh = mesh::generate_mesh(
            &raw.elements,
            UVec2::new(raw.resolution.width, raw.resolution.height),
        )
        .unwrap();
        let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let pos_count = match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => 0,
        };
        assert_eq!(pos_count, 72); // 3 elements * 6 faces * 4 vertices
    }

    #[test]
    fn test_create_texture_from_png() {
        // 1x1 red pixel PNG encoded as base64
        let png_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
        let raw_bytes = base64_decode(png_base64).unwrap();

        let model = BlockbenchModel {
            name: "test".to_string(),
            resolution: UVec2::new(16, 16),
            mesh: Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            ),
            texture_data: Some(TextureData {
                raw_bytes,
                uv_width: Some(1),
                uv_height: Some(1),
            }),
            bones: vec![],
            animations: vec![],
        };

        let image = model.create_texture();
        assert!(image.is_some());

        let image = image.unwrap();
        // 1x1 RGBA image = 4 bytes
        assert_eq!(image.width(), 1);
        assert_eq!(image.height(), 1);
    }

    #[test]
    fn test_create_texture_no_data() {
        let model = BlockbenchModel {
            name: "test".to_string(),
            resolution: UVec2::new(16, 16),
            mesh: Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            ),
            texture_data: None,
            bones: vec![],
            animations: vec![],
        };

        let image = model.create_texture();
        assert!(image.is_none());
    }

    #[test]
    fn test_create_texture_invalid_format() {
        let model = BlockbenchModel {
            name: "test".to_string(),
            resolution: UVec2::new(16, 16),
            mesh: Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            ),
            texture_data: Some(TextureData {
                raw_bytes: vec![0x00, 0x01, 0x02, 0x03], // Invalid image data
                uv_width: Some(1),
                uv_height: Some(1),
            }),
            bones: vec![],
            animations: vec![],
        };

        let image = model.create_texture();
        assert!(image.is_none());
    }

    #[test]
    fn test_load_bbmodel_integration() {
        // Integration test: full JSON with elements, bones, and animations
        let json = r#"{
            "name": "animated_machine",
            "meta": {
                "format_version": "4.10",
                "model_format": "free"
            },
            "resolution": {
                "width": 16,
                "height": 16
            },
            "elements": [
                {
                    "uuid": "base-uuid",
                    "name": "base",
                    "type": "cube",
                    "from": [0, 0, 0],
                    "to": [16, 4, 16],
                    "origin": [8, 2, 8],
                    "rotation": [0, 0, 0],
                    "faces": {
                        "north": {"uv": [0, 0, 16, 4], "texture": 0},
                        "south": {"uv": [0, 0, 16, 4], "texture": 0},
                        "east": {"uv": [0, 0, 16, 4], "texture": 0},
                        "west": {"uv": [0, 0, 16, 4], "texture": 0},
                        "up": {"uv": [0, 0, 16, 16], "texture": 0},
                        "down": {"uv": [0, 0, 16, 16], "texture": 0}
                    }
                },
                {
                    "uuid": "arm-uuid",
                    "name": "arm",
                    "type": "cube",
                    "from": [6, 4, 6],
                    "to": [10, 12, 10],
                    "origin": [8, 4, 8],
                    "rotation": [0, 0, 0],
                    "faces": {
                        "north": {"uv": [0, 0, 4, 8], "texture": 0},
                        "south": {"uv": [0, 0, 4, 8], "texture": 0},
                        "east": {"uv": [0, 0, 4, 8], "texture": 0},
                        "west": {"uv": [0, 0, 4, 8], "texture": 0},
                        "up": {"uv": [0, 0, 4, 4], "texture": 0},
                        "down": {"uv": [0, 0, 4, 4], "texture": 0}
                    }
                }
            ],
            "outliner": [
                {
                    "name": "body",
                    "origin": [8, 0, 8],
                    "children": [
                        "base-uuid",
                        {
                            "name": "arm_bone",
                            "origin": [8, 4, 8],
                            "children": ["arm-uuid"]
                        }
                    ]
                }
            ],
            "animations": [
                {
                    "name": "rotate",
                    "loop": "loop",
                    "length": 2.0,
                    "animators": {
                        "arm_bone": {
                            "rotation": [
                                {
                                    "time": 0,
                                    "data_points": [{"x": 0, "y": 0, "z": 0}],
                                    "interpolation": "linear"
                                },
                                {
                                    "time": 1.0,
                                    "data_points": [{"x": 0, "y": 180, "z": 0}],
                                    "interpolation": "linear"
                                },
                                {
                                    "time": 2.0,
                                    "data_points": [{"x": 0, "y": 360, "z": 0}],
                                    "interpolation": "linear"
                                }
                            ]
                        }
                    }
                }
            ],
            "textures": []
        }"#;

        // Parse raw bbmodel
        let raw: RawBbmodel = serde_json::from_str(json).unwrap();

        // Generate mesh
        let mesh = mesh::generate_mesh(
            &raw.elements,
            UVec2::new(raw.resolution.width, raw.resolution.height),
        )
        .unwrap();

        // Parse bones
        let bones = animation::parse_outliner(&raw.outliner, None);

        // Parse animations
        let animations = animation::parse_animations(&raw.animations);

        // Verify mesh (2 elements * 6 faces * 4 vertices = 48)
        let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let pos_count = match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => 0,
        };
        assert_eq!(pos_count, 48);

        // Verify bones
        assert_eq!(bones.len(), 1);
        assert_eq!(bones[0].name, "body");
        assert_eq!(bones[0].children.len(), 2);

        // Verify nested bone
        if let BoneChild::Bone(arm_bone) = &bones[0].children[1] {
            assert_eq!(arm_bone.name, "arm_bone");
            assert_eq!(arm_bone.parent, Some("body".to_string()));
            assert_eq!(arm_bone.origin, Vec3::new(8.0, 4.0, 8.0));
        } else {
            panic!("Expected nested bone");
        }

        // Verify animations
        assert_eq!(animations.len(), 1);
        assert_eq!(animations[0].name, "rotate");
        assert_eq!(animations[0].loop_mode, LoopMode::Loop);
        assert_eq!(animations[0].length, 2.0);

        // Verify keyframes (arm_bone is the animator key in JSON)
        let keyframes = animations[0].bone_keyframes.get("arm_bone").unwrap();
        assert_eq!(keyframes.len(), 3);
        assert_eq!(keyframes[0].channel, AnimationChannel::Rotation);
        assert_eq!(keyframes[0].time, 0.0);
        assert_eq!(keyframes[1].time, 1.0);
        assert_eq!(keyframes[2].time, 2.0);
        assert_eq!(keyframes[1].value, Vec3::new(0.0, 180.0, 0.0));
    }
}
