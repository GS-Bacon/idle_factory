//! Texture handling for Blockbench models

use super::raw::RawTexture;

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

/// Extract embedded texture data from bbmodel (raw bytes, not decoded image)
pub(crate) fn extract_texture_data(textures: &[RawTexture]) -> Option<TextureData> {
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
pub(crate) fn base64_decode(input: &str) -> Option<Vec<u8>> {
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
}
