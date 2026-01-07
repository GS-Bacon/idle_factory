//! Blockbench .bbmodel file loader
//!
//! Loads Blockbench project files (.bbmodel) and converts them to Bevy meshes.
//! Supports animations via keyframe data.

use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
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
    /// Embedded texture data (raw PNG/JPEG bytes)
    /// Use `create_texture()` to convert to a Bevy Image
    pub texture_data: Option<TextureData>,
    /// Bone hierarchy from outliner
    pub bones: Vec<Bone>,
    /// Animations
    pub animations: Vec<Animation>,
}

/// Animation from Blockbench
#[derive(Debug, Clone)]
pub struct Animation {
    /// Animation name
    pub name: String,
    /// Loop mode
    pub loop_mode: LoopMode,
    /// Total animation length in seconds
    pub length: f32,
    /// Keyframes per bone (bone UUID -> keyframes)
    pub bone_keyframes: std::collections::HashMap<String, Vec<Keyframe>>,
}

/// Animation loop mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoopMode {
    /// Play once
    #[default]
    Once,
    /// Loop continuously
    Loop,
    /// Hold on last frame
    Hold,
}

/// Single animation keyframe
#[derive(Debug, Clone)]
pub struct Keyframe {
    /// Which property is being animated
    pub channel: AnimationChannel,
    /// Time in seconds
    pub time: f32,
    /// Value (position/scale as Vec3, rotation as Euler angles in degrees)
    pub value: Vec3,
    /// Interpolation method
    pub interpolation: Interpolation,
}

/// Animation channel type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationChannel {
    /// Position offset
    Position,
    /// Rotation in Euler angles (degrees)
    Rotation,
    /// Scale factor
    Scale,
}

/// Keyframe interpolation method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Interpolation {
    /// Linear interpolation
    #[default]
    Linear,
    /// Catmull-Rom spline
    CatmullRom,
    /// Bezier curve
    Bezier,
    /// Step (no interpolation)
    Step,
}

/// Bone (skeletal hierarchy node) from Blockbench outliner
#[derive(Debug, Clone)]
pub struct Bone {
    /// Bone name
    pub name: String,
    /// Parent bone name (None for root bones)
    pub parent: Option<String>,
    /// Pivot point / origin
    pub origin: Vec3,
    /// Child elements (nested bones or element UUIDs)
    pub children: Vec<BoneChild>,
}

/// Child of a bone - either another bone or an element reference
#[derive(Debug, Clone)]
pub enum BoneChild {
    /// Nested bone
    Bone(Bone),
    /// Element UUID reference
    Element(String),
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
    /// Outliner (bone hierarchy)
    #[serde(default)]
    outliner: Vec<serde_json::Value>,
    /// Animations
    #[serde(default)]
    animations: Vec<RawAnimation>,
}

/// Raw animation from JSON
#[derive(Debug, Deserialize)]
struct RawAnimation {
    /// Animation name
    #[serde(default)]
    name: String,
    /// Loop mode: "once", "loop", or "hold"
    #[serde(rename = "loop", default)]
    loop_mode: String,
    /// Animation length in seconds
    #[serde(default)]
    length: f32,
    /// Animators by bone UUID
    #[serde(default)]
    animators: std::collections::HashMap<String, RawAnimator>,
}

/// Raw animator for a single bone
#[derive(Debug, Deserialize)]
struct RawAnimator {
    /// Position keyframes
    #[serde(default)]
    position: Vec<RawKeyframe>,
    /// Rotation keyframes
    #[serde(default)]
    rotation: Vec<RawKeyframe>,
    /// Scale keyframes
    #[serde(default)]
    scale: Vec<RawKeyframe>,
}

/// Raw keyframe from JSON
#[derive(Debug, Deserialize)]
struct RawKeyframe {
    /// Time in seconds (can be number or string)
    #[serde(default, deserialize_with = "deserialize_time")]
    time: f32,
    /// Value as [x, y, z] or single values
    #[serde(default, rename = "data_points")]
    data_points: Vec<RawDataPoint>,
    /// Interpolation mode
    #[serde(default)]
    interpolation: String,
}

/// Raw data point (value at keyframe)
#[derive(Debug, Deserialize)]
struct RawDataPoint {
    #[serde(default, deserialize_with = "deserialize_value")]
    x: f32,
    #[serde(default, deserialize_with = "deserialize_value")]
    y: f32,
    #[serde(default, deserialize_with = "deserialize_value")]
    z: f32,
}

/// Deserialize time which can be a number or string
fn deserialize_time<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(n) => n
            .as_f64()
            .map(|v| v as f32)
            .ok_or_else(|| D::Error::custom("expected number")),
        serde_json::Value::String(s) => s
            .parse::<f32>()
            .map_err(|_| D::Error::custom(format!("invalid time string: {}", s))),
        _ => Ok(0.0),
    }
}

/// Deserialize value which can be a number or string (for expressions)
fn deserialize_value<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(n) => Ok(n.as_f64().unwrap_or(0.0) as f32),
        serde_json::Value::String(s) => s.parse::<f32>().unwrap_or(0.0).pipe(Ok),
        _ => Ok(0.0),
    }
}

/// Helper trait for pipe syntax
trait Pipe: Sized {
    fn pipe<T, F: FnOnce(Self) -> T>(self, f: F) -> T {
        f(self)
    }
}

impl<T> Pipe for T {}

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

/// Parse outliner array into bone hierarchy
fn parse_outliner(outliner: &[serde_json::Value], parent_name: Option<&str>) -> Vec<Bone> {
    let mut bones = Vec::new();

    for item in outliner {
        match item {
            // String = element UUID reference (not a bone, skip at root level)
            serde_json::Value::String(_) => {
                // Element references at root level are orphaned elements
                // They don't belong to any bone, so we skip them here
            }
            // Object = bone definition
            serde_json::Value::Object(obj) => {
                if let Some(bone) = parse_bone_object(obj, parent_name) {
                    bones.push(bone);
                }
            }
            _ => {}
        }
    }

    bones
}

/// Parse a single bone object from JSON
fn parse_bone_object(
    obj: &serde_json::Map<String, serde_json::Value>,
    parent_name: Option<&str>,
) -> Option<Bone> {
    // Get bone name (required)
    let name = obj.get("name")?.as_str()?.to_string();

    // Get origin (pivot point), default to [0, 0, 0]
    let origin = if let Some(serde_json::Value::Array(arr)) = obj.get("origin") {
        Vec3::new(
            arr.first().and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
            arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
            arr.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
        )
    } else {
        Vec3::ZERO
    };

    // Parse children
    let mut children = Vec::new();
    if let Some(serde_json::Value::Array(child_arr)) = obj.get("children") {
        for child in child_arr {
            match child {
                // String = element UUID
                serde_json::Value::String(uuid) => {
                    children.push(BoneChild::Element(uuid.clone()));
                }
                // Object = nested bone
                serde_json::Value::Object(child_obj) => {
                    if let Some(child_bone) = parse_bone_object(child_obj, Some(&name)) {
                        children.push(BoneChild::Bone(child_bone));
                    }
                }
                _ => {}
            }
        }
    }

    Some(Bone {
        name,
        parent: parent_name.map(|s| s.to_string()),
        origin,
        children,
    })
}

/// Parse animations from raw animation data
fn parse_animations(raw_anims: &[RawAnimation]) -> Vec<Animation> {
    raw_anims.iter().map(parse_animation).collect()
}

/// Parse a single animation
fn parse_animation(raw: &RawAnimation) -> Animation {
    let loop_mode = match raw.loop_mode.as_str() {
        "loop" => LoopMode::Loop,
        "hold" => LoopMode::Hold,
        _ => LoopMode::Once,
    };

    let mut bone_keyframes = std::collections::HashMap::new();

    for (bone_uuid, animator) in &raw.animators {
        let mut keyframes = Vec::new();

        // Parse position keyframes
        for kf in &animator.position {
            if let Some(keyframe) = parse_keyframe(kf, AnimationChannel::Position) {
                keyframes.push(keyframe);
            }
        }

        // Parse rotation keyframes
        for kf in &animator.rotation {
            if let Some(keyframe) = parse_keyframe(kf, AnimationChannel::Rotation) {
                keyframes.push(keyframe);
            }
        }

        // Parse scale keyframes
        for kf in &animator.scale {
            if let Some(keyframe) = parse_keyframe(kf, AnimationChannel::Scale) {
                keyframes.push(keyframe);
            }
        }

        // Sort keyframes by time
        keyframes.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if !keyframes.is_empty() {
            bone_keyframes.insert(bone_uuid.clone(), keyframes);
        }
    }

    Animation {
        name: raw.name.clone(),
        loop_mode,
        length: raw.length,
        bone_keyframes,
    }
}

/// Parse a single keyframe
fn parse_keyframe(raw: &RawKeyframe, channel: AnimationChannel) -> Option<Keyframe> {
    // Get value from first data point
    let value = if let Some(dp) = raw.data_points.first() {
        Vec3::new(dp.x, dp.y, dp.z)
    } else {
        return None;
    };

    let interpolation = match raw.interpolation.as_str() {
        "catmullrom" => Interpolation::CatmullRom,
        "bezier" => Interpolation::Bezier,
        "step" => Interpolation::Step,
        _ => Interpolation::Linear,
    };

    Some(Keyframe {
        channel,
        time: raw.time,
        value,
        interpolation,
    })
}

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
    fn test_parse_outliner_simple() {
        // Simple outliner with one bone containing one element
        let outliner_json = r#"[
            {
                "name": "root",
                "origin": [0, 0, 0],
                "children": ["element-uuid-1"]
            }
        ]"#;

        let outliner: Vec<serde_json::Value> = serde_json::from_str(outliner_json).unwrap();
        let bones = parse_outliner(&outliner, None);

        assert_eq!(bones.len(), 1);
        assert_eq!(bones[0].name, "root");
        assert!(bones[0].parent.is_none());
        assert_eq!(bones[0].origin, Vec3::ZERO);
        assert_eq!(bones[0].children.len(), 1);

        match &bones[0].children[0] {
            BoneChild::Element(uuid) => assert_eq!(uuid, "element-uuid-1"),
            BoneChild::Bone(_) => panic!("Expected Element, got Bone"),
        }
    }

    #[test]
    fn test_parse_outliner_nested() {
        // Nested outliner with parent and child bones
        let outliner_json = r#"[
            {
                "name": "root",
                "origin": [0, 0, 0],
                "children": [
                    "element-uuid-1",
                    {
                        "name": "arm",
                        "origin": [4, 8, 0],
                        "children": ["element-uuid-2"]
                    }
                ]
            }
        ]"#;

        let outliner: Vec<serde_json::Value> = serde_json::from_str(outliner_json).unwrap();
        let bones = parse_outliner(&outliner, None);

        assert_eq!(bones.len(), 1);
        let root = &bones[0];
        assert_eq!(root.name, "root");
        assert!(root.parent.is_none());
        assert_eq!(root.children.len(), 2);

        // First child is an element
        match &root.children[0] {
            BoneChild::Element(uuid) => assert_eq!(uuid, "element-uuid-1"),
            BoneChild::Bone(_) => panic!("Expected Element"),
        }

        // Second child is a nested bone
        match &root.children[1] {
            BoneChild::Bone(arm) => {
                assert_eq!(arm.name, "arm");
                assert_eq!(arm.parent, Some("root".to_string()));
                assert_eq!(arm.origin, Vec3::new(4.0, 8.0, 0.0));
                assert_eq!(arm.children.len(), 1);

                match &arm.children[0] {
                    BoneChild::Element(uuid) => assert_eq!(uuid, "element-uuid-2"),
                    BoneChild::Bone(_) => panic!("Expected Element in arm"),
                }
            }
            BoneChild::Element(_) => panic!("Expected Bone"),
        }
    }

    #[test]
    fn test_parse_outliner_empty() {
        let outliner: Vec<serde_json::Value> = vec![];
        let bones = parse_outliner(&outliner, None);
        assert!(bones.is_empty());
    }

    #[test]
    fn test_parse_outliner_root_element_only() {
        // Outliner with only element UUIDs at root level (no bones)
        let outliner_json = r#"["element-uuid-1", "element-uuid-2"]"#;
        let outliner: Vec<serde_json::Value> = serde_json::from_str(outliner_json).unwrap();
        let bones = parse_outliner(&outliner, None);

        // Root-level elements are not bones, so bones should be empty
        assert!(bones.is_empty());
    }

    #[test]
    fn test_parse_animation_simple() {
        // Simple animation with position and rotation keyframes
        let raw = RawAnimation {
            name: "walk".to_string(),
            loop_mode: "loop".to_string(),
            length: 1.0,
            animators: {
                let mut map = std::collections::HashMap::new();
                map.insert(
                    "bone-uuid-1".to_string(),
                    RawAnimator {
                        position: vec![
                            RawKeyframe {
                                time: 0.0,
                                data_points: vec![RawDataPoint {
                                    x: 0.0,
                                    y: 0.0,
                                    z: 0.0,
                                }],
                                interpolation: "linear".to_string(),
                            },
                            RawKeyframe {
                                time: 0.5,
                                data_points: vec![RawDataPoint {
                                    x: 0.0,
                                    y: 1.0,
                                    z: 0.0,
                                }],
                                interpolation: "linear".to_string(),
                            },
                        ],
                        rotation: vec![RawKeyframe {
                            time: 0.0,
                            data_points: vec![RawDataPoint {
                                x: 0.0,
                                y: 45.0,
                                z: 0.0,
                            }],
                            interpolation: "catmullrom".to_string(),
                        }],
                        scale: vec![],
                    },
                );
                map
            },
        };

        let anim = parse_animation(&raw);

        assert_eq!(anim.name, "walk");
        assert_eq!(anim.loop_mode, LoopMode::Loop);
        assert_eq!(anim.length, 1.0);
        assert_eq!(anim.bone_keyframes.len(), 1);

        let keyframes = anim.bone_keyframes.get("bone-uuid-1").unwrap();
        assert_eq!(keyframes.len(), 3); // 2 position + 1 rotation

        // Keyframes should be sorted by time
        assert_eq!(keyframes[0].time, 0.0);
        assert_eq!(keyframes[1].time, 0.0);
        assert_eq!(keyframes[2].time, 0.5);
    }

    #[test]
    fn test_parse_animation_all_interpolation_types() {
        // Test all interpolation types
        let raw = RawAnimation {
            name: "test".to_string(),
            loop_mode: "once".to_string(),
            length: 2.0,
            animators: {
                let mut map = std::collections::HashMap::new();
                map.insert(
                    "bone-1".to_string(),
                    RawAnimator {
                        position: vec![
                            RawKeyframe {
                                time: 0.0,
                                data_points: vec![RawDataPoint {
                                    x: 0.0,
                                    y: 0.0,
                                    z: 0.0,
                                }],
                                interpolation: "linear".to_string(),
                            },
                            RawKeyframe {
                                time: 0.5,
                                data_points: vec![RawDataPoint {
                                    x: 1.0,
                                    y: 0.0,
                                    z: 0.0,
                                }],
                                interpolation: "catmullrom".to_string(),
                            },
                            RawKeyframe {
                                time: 1.0,
                                data_points: vec![RawDataPoint {
                                    x: 2.0,
                                    y: 0.0,
                                    z: 0.0,
                                }],
                                interpolation: "bezier".to_string(),
                            },
                            RawKeyframe {
                                time: 1.5,
                                data_points: vec![RawDataPoint {
                                    x: 3.0,
                                    y: 0.0,
                                    z: 0.0,
                                }],
                                interpolation: "step".to_string(),
                            },
                        ],
                        rotation: vec![],
                        scale: vec![RawKeyframe {
                            time: 0.0,
                            data_points: vec![RawDataPoint {
                                x: 1.0,
                                y: 1.0,
                                z: 1.0,
                            }],
                            interpolation: "linear".to_string(),
                        }],
                    },
                );
                map
            },
        };

        let anim = parse_animation(&raw);

        assert_eq!(anim.name, "test");
        assert_eq!(anim.loop_mode, LoopMode::Once);
        assert_eq!(anim.length, 2.0);

        let keyframes = anim.bone_keyframes.get("bone-1").unwrap();
        assert_eq!(keyframes.len(), 5); // 4 position + 1 scale

        // Check interpolation types (keyframes are sorted by time)
        assert_eq!(keyframes[0].interpolation, Interpolation::Linear); // position at 0.0
        assert_eq!(keyframes[1].interpolation, Interpolation::Linear); // scale at 0.0
        assert_eq!(keyframes[2].interpolation, Interpolation::CatmullRom); // position at 0.5
        assert_eq!(keyframes[3].interpolation, Interpolation::Bezier); // position at 1.0
        assert_eq!(keyframes[4].interpolation, Interpolation::Step); // position at 1.5

        // Check channels
        assert_eq!(keyframes[0].channel, AnimationChannel::Position);
        assert_eq!(keyframes[1].channel, AnimationChannel::Scale);
        assert_eq!(keyframes[2].channel, AnimationChannel::Position);

        // Check values
        assert_eq!(keyframes[0].value, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(keyframes[1].value, Vec3::new(1.0, 1.0, 1.0)); // scale
        assert_eq!(keyframes[2].value, Vec3::new(1.0, 0.0, 0.0));
    }
}
