//! Blockbench .bbmodel file loader
//!
//! Loads Blockbench project files (.bbmodel) and converts them to Bevy meshes.
//! Currently supports static models only (no animations).

use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use serde::Deserialize;
use std::fmt;

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
    /// Embedded texture data (raw RGBA bytes, if parsed)
    /// Note: Texture loading is currently not implemented (requires image crate)
    pub texture_data: Option<TextureData>,
}

/// Embedded texture data from bbmodel
#[derive(Debug, Clone)]
pub struct TextureData {
    /// Raw decoded bytes (e.g., PNG data)
    pub raw_bytes: Vec<u8>,
    /// Width from UV settings (if available)
    pub uv_width: Option<u32>,
    /// Height from UV settings (if available)
    pub uv_height: Option<u32>,
}

/// Blockbench asset loader
#[derive(Default)]
pub struct BlockbenchLoader;

/// Raw .bbmodel JSON structure (top level)
#[derive(Debug, Deserialize)]
struct RawBbmodel {
    /// Model name
    #[serde(default)]
    name: String,
    /// Format version (e.g., "4.10")
    #[serde(default)]
    #[allow(dead_code)]
    meta: RawMeta,
    /// Texture resolution
    #[serde(default)]
    resolution: RawResolution,
    /// Elements (cubes, planes, etc.)
    #[serde(default)]
    elements: Vec<RawElement>,
    /// Textures array
    #[serde(default)]
    textures: Vec<RawTexture>,
    /// Outliner (bone hierarchy) - unused for static models
    #[serde(default)]
    #[allow(dead_code)]
    outliner: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize, Default)]
struct RawMeta {
    #[serde(default)]
    #[allow(dead_code)]
    format_version: String,
    #[serde(default)]
    #[allow(dead_code)]
    model_format: String,
}

#[derive(Debug, Deserialize)]
struct RawResolution {
    width: u32,
    height: u32,
}

impl Default for RawResolution {
    fn default() -> Self {
        Self {
            width: 16,
            height: 16,
        }
    }
}

/// Element (cube) in the model
#[derive(Debug, Deserialize)]
struct RawElement {
    /// Unique identifier
    #[serde(default)]
    #[allow(dead_code)]
    uuid: String,
    /// Element type (usually "cube")
    #[serde(rename = "type", default)]
    #[allow(dead_code)]
    element_type: String,
    /// Element name
    #[serde(default)]
    #[allow(dead_code)]
    name: String,
    /// Box UV mode
    #[serde(default)]
    #[allow(dead_code)]
    box_uv: bool,
    /// Starting corner position [x, y, z]
    #[serde(default)]
    from: [f32; 3],
    /// Ending corner position [x, y, z]
    #[serde(default)]
    to: [f32; 3],
    /// Rotation origin [x, y, z]
    #[serde(default)]
    origin: [f32; 3],
    /// Rotation angles [x, y, z] in degrees
    #[serde(default)]
    rotation: [f32; 3],
    /// Face definitions
    #[serde(default)]
    faces: RawFaces,
}

/// Faces of an element
#[derive(Debug, Deserialize, Default)]
struct RawFaces {
    #[serde(default)]
    north: Option<RawFace>,
    #[serde(default)]
    south: Option<RawFace>,
    #[serde(default)]
    east: Option<RawFace>,
    #[serde(default)]
    west: Option<RawFace>,
    #[serde(default)]
    up: Option<RawFace>,
    #[serde(default)]
    down: Option<RawFace>,
}

/// Single face definition
#[derive(Debug, Deserialize)]
struct RawFace {
    /// UV coordinates [u1, v1, u2, v2] in pixels
    #[serde(default)]
    uv: [f32; 4],
    /// Texture index or null
    #[serde(default)]
    #[allow(dead_code)]
    texture: Option<i32>,
    /// Face rotation (0, 90, 180, 270)
    #[serde(default)]
    rotation: i32,
}

/// Texture definition
#[derive(Debug, Deserialize)]
struct RawTexture {
    #[serde(default)]
    #[allow(dead_code)]
    uuid: String,
    #[serde(default)]
    #[allow(dead_code)]
    name: String,
    /// Base64 encoded image data (data:image/png;base64,...)
    #[serde(default)]
    source: String,
    #[serde(default)]
    uv_width: Option<u32>,
    #[serde(default)]
    uv_height: Option<u32>,
}

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

        tracing::info!(
            "Loaded bbmodel: {} ({} elements, {}x{} resolution)",
            name,
            raw.elements.len(),
            resolution.x,
            resolution.y
        );

        Ok(BlockbenchModel {
            name,
            resolution,
            mesh,
            texture_data,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bbmodel"]
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

/// Generate Bevy Mesh from bbmodel elements
fn generate_mesh(elements: &[RawElement], resolution: UVec2) -> Result<Mesh, BlockbenchLoadError> {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Scale factor: 16 pixels = 1 game unit (Minecraft convention)
    let scale = 1.0 / 16.0;

    for element in elements {
        add_element_mesh(
            element,
            resolution,
            scale,
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
        );
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    Ok(mesh)
}

/// Add a single element's geometry to the mesh
fn add_element_mesh(
    element: &RawElement,
    resolution: UVec2,
    scale: f32,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
) {
    let from = element.from;
    let to = element.to;
    let origin = element.origin;
    let rotation = element.rotation;

    // Calculate the 8 corners of the cube
    // Blockbench uses Y-up, same as Bevy
    let corners = [
        [from[0], from[1], from[2]], // 0: ---
        [to[0], from[1], from[2]],   // 1: +--
        [to[0], to[1], from[2]],     // 2: ++-
        [from[0], to[1], from[2]],   // 3: -+-
        [from[0], from[1], to[2]],   // 4: --+
        [to[0], from[1], to[2]],     // 5: +-+
        [to[0], to[1], to[2]],       // 6: +++
        [from[0], to[1], to[2]],     // 7: -++
    ];

    // Apply rotation around origin if needed
    let corners: Vec<[f32; 3]> = if rotation[0] != 0.0 || rotation[1] != 0.0 || rotation[2] != 0.0 {
        let rot = Quat::from_euler(
            EulerRot::YXZ,
            rotation[1].to_radians(),
            rotation[0].to_radians(),
            rotation[2].to_radians(),
        );
        corners
            .iter()
            .map(|c| {
                let p = Vec3::from(*c) - Vec3::from(origin);
                let rotated = rot * p + Vec3::from(origin);
                [rotated.x, rotated.y, rotated.z]
            })
            .collect()
    } else {
        corners.to_vec()
    };

    // Transform corners to game space (scale and center)
    let transformed: Vec<[f32; 3]> = corners
        .iter()
        .map(|c| {
            [
                (c[0] - 8.0) * scale, // Center at 0 (Blockbench default is 0-16)
                (c[1] - 8.0) * scale,
                (c[2] - 8.0) * scale,
            ]
        })
        .collect();

    // Face definitions: (corner indices, normal, face data accessor)
    let face_defs: [([usize; 4], [f32; 3], &Option<RawFace>); 6] = [
        ([0, 1, 2, 3], [0.0, 0.0, -1.0], &element.faces.north),
        ([5, 4, 7, 6], [0.0, 0.0, 1.0], &element.faces.south),
        ([1, 5, 6, 2], [1.0, 0.0, 0.0], &element.faces.east),
        ([4, 0, 3, 7], [-1.0, 0.0, 0.0], &element.faces.west),
        ([3, 2, 6, 7], [0.0, 1.0, 0.0], &element.faces.up),
        ([4, 5, 1, 0], [0.0, -1.0, 0.0], &element.faces.down),
    ];

    for (corner_indices, normal, face_opt) in &face_defs {
        let Some(face) = face_opt else { continue };

        let base_idx = positions.len() as u32;

        // Add vertices for this face
        for &ci in corner_indices {
            positions.push(transformed[ci]);
            normals.push(*normal);
        }

        // Calculate UV coordinates (convert from pixel to 0-1 range)
        let uv = face.uv;
        let u1 = uv[0] / resolution.x as f32;
        let v1 = uv[1] / resolution.y as f32;
        let u2 = uv[2] / resolution.x as f32;
        let v2 = uv[3] / resolution.y as f32;

        // Apply UV based on face rotation
        let face_uvs = match face.rotation {
            0 => [[u1, v2], [u2, v2], [u2, v1], [u1, v1]],
            90 => [[u1, v1], [u1, v2], [u2, v2], [u2, v1]],
            180 => [[u2, v1], [u1, v1], [u1, v2], [u2, v2]],
            270 => [[u2, v2], [u2, v1], [u1, v1], [u1, v2]],
            _ => [[u1, v2], [u2, v2], [u2, v1], [u1, v1]],
        };

        for uv_coord in &face_uvs {
            uvs.push(*uv_coord);
        }

        // Add indices (two triangles)
        indices.extend_from_slice(&[
            base_idx,
            base_idx + 1,
            base_idx + 2,
            base_idx,
            base_idx + 2,
            base_idx + 3,
        ]);
    }
}

/// Extract embedded texture data from bbmodel (raw bytes, not decoded image)
fn extract_texture_data(textures: &[RawTexture]) -> Option<TextureData> {
    // Get the first texture with embedded data
    let texture = textures
        .iter()
        .find(|t| t.source.starts_with("data:image/"))?;

    // Parse data URL: data:image/png;base64,<data>
    let source = &texture.source;
    let base64_start = source.find(',')? + 1;
    let base64_data = &source[base64_start..];

    // Decode base64
    let raw_bytes = base64_decode(base64_data)?;

    Some(TextureData {
        raw_bytes,
        uv_width: texture.uv_width,
        uv_height: texture.uv_height,
    })
}

/// Simple base64 decoder
fn base64_decode(input: &str) -> Option<Vec<u8>> {
    const DECODE_TABLE: [i8; 128] = [
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 62, -1, -1,
        -1, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4,
        5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, -1, -1, -1,
        -1, -1, -1, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
        46, 47, 48, 49, 50, 51, -1, -1, -1, -1, -1,
    ];

    let input = input.trim_end_matches('=');
    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut buffer = 0u32;
    let mut bits = 0;

    for c in input.bytes() {
        if c >= 128 {
            return None;
        }
        let val = DECODE_TABLE[c as usize];
        if val < 0 {
            continue; // Skip whitespace
        }
        buffer = (buffer << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push((buffer >> bits) as u8);
        }
    }

    Some(output)
}

/// Helper function to spawn a BlockbenchModel as an entity with a simple material
pub fn spawn_bbmodel(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    model: &BlockbenchModel,
    transform: Transform,
) -> Entity {
    let mesh_handle = meshes.add(model.mesh.clone());

    // Note: Texture loading is not yet implemented
    // Would need image crate as a regular dependency to decode PNG data
    let material = StandardMaterial {
        base_color: Color::WHITE,
        ..default()
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

    #[test]
    fn test_base64_decode() {
        let encoded = "SGVsbG8gV29ybGQ="; // "Hello World"
        let decoded = base64_decode(encoded).unwrap();
        assert_eq!(decoded, b"Hello World");
    }

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
    fn test_generate_mesh_single_cube() {
        let elements = vec![RawElement {
            uuid: "test".to_string(),
            element_type: "cube".to_string(),
            name: "cube".to_string(),
            box_uv: false,
            from: [0.0, 0.0, 0.0],
            to: [16.0, 16.0, 16.0],
            origin: [8.0, 8.0, 8.0],
            rotation: [0.0, 0.0, 0.0],
            faces: RawFaces {
                north: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
                south: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
                east: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
                west: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
                up: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
                down: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
            },
        }];

        let mesh = generate_mesh(&elements, UVec2::new(16, 16)).unwrap();

        // 6 faces * 4 vertices = 24 vertices
        let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let pos_count = match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => 0,
        };
        assert_eq!(pos_count, 24);

        // 6 faces * 2 triangles * 3 indices = 36 indices
        let indices = mesh.indices().unwrap();
        assert_eq!(indices.len(), 36);
    }

    #[test]
    fn test_generate_mesh_rotated_cube() {
        let elements = vec![RawElement {
            uuid: "test".to_string(),
            element_type: "cube".to_string(),
            name: "cube".to_string(),
            box_uv: false,
            from: [0.0, 0.0, 0.0],
            to: [16.0, 16.0, 16.0],
            origin: [8.0, 8.0, 8.0],
            rotation: [0.0, 45.0, 0.0], // 45 degree rotation on Y axis
            faces: RawFaces {
                north: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
                south: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
                east: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
                west: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
                up: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
                down: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
            },
        }];

        let mesh = generate_mesh(&elements, UVec2::new(16, 16)).unwrap();

        // Should still have 24 vertices even with rotation
        let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let pos_count = match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => 0,
        };
        assert_eq!(pos_count, 24);
    }

    #[test]
    fn test_extract_texture_data() {
        // Test with a minimal PNG (1x1 red pixel) encoded as base64
        let textures = vec![RawTexture {
            uuid: "test".to_string(),
            name: "test.png".to_string(),
            source: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==".to_string(),
            uv_width: Some(16),
            uv_height: Some(16),
        }];

        let data = extract_texture_data(&textures).unwrap();
        assert!(!data.raw_bytes.is_empty());
        assert_eq!(data.uv_width, Some(16));
        assert_eq!(data.uv_height, Some(16));
        // PNG magic bytes
        assert_eq!(&data.raw_bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]);
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
        let mesh = generate_mesh(
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
        let mesh = generate_mesh(
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
}
