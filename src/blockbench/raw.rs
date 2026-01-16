//! Raw JSON structures for .bbmodel files

use serde::Deserialize;

/// Raw .bbmodel JSON structure (top level)
#[derive(Debug, Deserialize)]
pub(crate) struct RawBbmodel {
    /// Model name
    #[serde(default)]
    pub name: String,
    /// Format version (e.g., "4.10")
    #[serde(default)]
    #[allow(dead_code)]
    pub meta: RawMeta,
    /// Texture resolution
    #[serde(default)]
    pub resolution: RawResolution,
    /// Elements (cubes, planes, etc.)
    #[serde(default)]
    pub elements: Vec<RawElement>,
    /// Textures array
    #[serde(default)]
    pub textures: Vec<RawTexture>,
    /// Outliner (bone hierarchy)
    #[serde(default)]
    pub outliner: Vec<serde_json::Value>,
    /// Animations
    #[serde(default)]
    pub animations: Vec<RawAnimation>,
}

/// Raw animation from JSON
#[derive(Debug, Deserialize)]
pub(crate) struct RawAnimation {
    /// Animation name
    #[serde(default)]
    pub name: String,
    /// Loop mode: "once", "loop", or "hold"
    #[serde(rename = "loop", default)]
    pub loop_mode: String,
    /// Animation length in seconds
    #[serde(default)]
    pub length: f32,
    /// Animators by bone UUID
    #[serde(default)]
    pub animators: std::collections::HashMap<String, RawAnimator>,
}

/// Raw animator for a single bone
#[derive(Debug, Deserialize)]
pub(crate) struct RawAnimator {
    /// Position keyframes
    #[serde(default)]
    pub position: Vec<RawKeyframe>,
    /// Rotation keyframes
    #[serde(default)]
    pub rotation: Vec<RawKeyframe>,
    /// Scale keyframes
    #[serde(default)]
    pub scale: Vec<RawKeyframe>,
}

/// Raw keyframe from JSON
#[derive(Debug, Deserialize)]
pub(crate) struct RawKeyframe {
    /// Time in seconds (can be number or string)
    #[serde(default, deserialize_with = "deserialize_time")]
    pub time: f32,
    /// Value as [x, y, z] or single values
    #[serde(default, rename = "data_points")]
    pub data_points: Vec<RawDataPoint>,
    /// Interpolation mode
    #[serde(default)]
    pub interpolation: String,
}

/// Raw data point (value at keyframe)
#[derive(Debug, Deserialize)]
pub(crate) struct RawDataPoint {
    #[serde(default, deserialize_with = "deserialize_value")]
    pub x: f32,
    #[serde(default, deserialize_with = "deserialize_value")]
    pub y: f32,
    #[serde(default, deserialize_with = "deserialize_value")]
    pub z: f32,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct RawMeta {
    #[serde(default)]
    #[allow(dead_code)]
    pub format_version: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub model_format: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawResolution {
    pub width: u32,
    pub height: u32,
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
pub(crate) struct RawElement {
    /// Unique identifier
    #[serde(default)]
    #[allow(dead_code)]
    pub uuid: String,
    /// Element type (usually "cube")
    #[serde(rename = "type", default)]
    #[allow(dead_code)]
    pub element_type: String,
    /// Element name
    #[serde(default)]
    #[allow(dead_code)]
    pub name: String,
    /// Box UV mode
    #[serde(default)]
    #[allow(dead_code)]
    pub box_uv: bool,
    /// Starting corner position [x, y, z]
    #[serde(default)]
    pub from: [f32; 3],
    /// Ending corner position [x, y, z]
    #[serde(default)]
    pub to: [f32; 3],
    /// Rotation origin [x, y, z]
    #[serde(default)]
    pub origin: [f32; 3],
    /// Rotation angles [x, y, z] in degrees
    #[serde(default)]
    pub rotation: [f32; 3],
    /// Face definitions
    #[serde(default)]
    pub faces: RawFaces,
}

/// Faces of an element
#[derive(Debug, Deserialize, Default)]
pub(crate) struct RawFaces {
    #[serde(default)]
    pub north: Option<RawFace>,
    #[serde(default)]
    pub south: Option<RawFace>,
    #[serde(default)]
    pub east: Option<RawFace>,
    #[serde(default)]
    pub west: Option<RawFace>,
    #[serde(default)]
    pub up: Option<RawFace>,
    #[serde(default)]
    pub down: Option<RawFace>,
}

/// Single face definition
#[derive(Debug, Deserialize)]
pub(crate) struct RawFace {
    /// UV coordinates [u1, v1, u2, v2] in pixels
    #[serde(default)]
    pub uv: [f32; 4],
    /// Texture index or null
    #[serde(default)]
    #[allow(dead_code)]
    pub texture: Option<i32>,
    /// Face rotation (0, 90, 180, 270)
    #[serde(default)]
    pub rotation: i32,
}

/// Texture definition
#[derive(Debug, Deserialize)]
pub(crate) struct RawTexture {
    #[serde(default)]
    #[allow(dead_code)]
    pub uuid: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub name: String,
    /// Base64 encoded image data (data:image/png;base64,...)
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub uv_width: Option<u32>,
    #[serde(default)]
    pub uv_height: Option<u32>,
}

/// Deserialize time which can be a number or string
pub(crate) fn deserialize_time<'de, D>(deserializer: D) -> Result<f32, D::Error>
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
pub(crate) fn deserialize_value<'de, D>(deserializer: D) -> Result<f32, D::Error>
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
