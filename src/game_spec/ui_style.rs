//! Declarative UI Style Definitions
//!
//! This module provides data structures for loading CSS-like style definitions
//! from TOML files, enabling declarative UI styling.

use serde::Deserialize;
use std::collections::HashMap;

/// Border specification for UI elements
#[derive(Debug, Clone, Deserialize, Default)]
pub struct BorderSpec {
    /// Border width (e.g., "2px")
    pub width: Option<String>,
    /// Border color (e.g., "#ff8700")
    pub color: Option<String>,
    /// Border radius (e.g., "6px")
    pub radius: Option<String>,
}

/// CSS-like style specification for UI elements
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UIStyleSpec {
    /// Width (e.g., "60px", "100%")
    pub width: Option<String>,
    /// Height (e.g., "60px", "auto")
    pub height: Option<String>,
    /// Background color (e.g., "#1a1a1a", "#1a1a1ae6" with alpha)
    pub background: Option<String>,
    /// Text color (e.g., "#ffffff")
    pub color: Option<String>,
    /// Border specification
    pub border: Option<BorderSpec>,
    /// Border radius shorthand (alternative to border.radius)
    pub border_radius: Option<String>,
    /// Padding (e.g., "16px", "8px 16px")
    pub padding: Option<String>,
    /// Margin (e.g., "8px", "8px 16px")
    pub margin: Option<String>,
    /// Bottom margin shorthand
    pub margin_bottom: Option<String>,
    /// Font size (e.g., "24px", "1.5em")
    pub font_size: Option<String>,
}

impl UIStyleSpec {
    /// Get effective border radius (from border.radius or border_radius)
    pub fn get_border_radius(&self) -> Option<&str> {
        self.border
            .as_ref()
            .and_then(|b| b.radius.as_deref())
            .or(self.border_radius.as_deref())
    }

    /// Parse a CSS-like size value to pixels (f32)
    /// Supports: "60px", "1.5em" (assumes 16px base), plain numbers
    pub fn parse_size(value: &str) -> Option<f32> {
        let value = value.trim();
        if value.ends_with("px") {
            value.trim_end_matches("px").parse().ok()
        } else if value.ends_with("em") {
            value
                .trim_end_matches("em")
                .parse::<f32>()
                .ok()
                .map(|v| v * 16.0)
        } else {
            value.parse().ok()
        }
    }

    /// Parse a CSS color string to RGBA tuple
    /// Supports: "#rgb", "#rrggbb", "#rrggbbaa"
    pub fn parse_color(value: &str) -> Option<(f32, f32, f32, f32)> {
        let value = value.trim().trim_start_matches('#');
        match value.len() {
            3 => {
                // #rgb -> #rrggbb
                let r = u8::from_str_radix(&value[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&value[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&value[2..3], 16).ok()? * 17;
                Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0))
            }
            6 => {
                // #rrggbb
                let r = u8::from_str_radix(&value[0..2], 16).ok()?;
                let g = u8::from_str_radix(&value[2..4], 16).ok()?;
                let b = u8::from_str_radix(&value[4..6], 16).ok()?;
                Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0))
            }
            8 => {
                // #rrggbbaa
                let r = u8::from_str_radix(&value[0..2], 16).ok()?;
                let g = u8::from_str_radix(&value[2..4], 16).ok()?;
                let b = u8::from_str_radix(&value[4..6], 16).ok()?;
                let a = u8::from_str_radix(&value[6..8], 16).ok()?;
                Some((
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    a as f32 / 255.0,
                ))
            }
            _ => None,
        }
    }

    /// Parse padding string to (top, right, bottom, left)
    /// Supports: "16px", "8px 16px", "8px 16px 8px 16px"
    pub fn parse_padding(value: &str) -> Option<(f32, f32, f32, f32)> {
        let parts: Vec<&str> = value.split_whitespace().collect();
        match parts.len() {
            1 => {
                let v = Self::parse_size(parts[0])?;
                Some((v, v, v, v))
            }
            2 => {
                let v = Self::parse_size(parts[0])?;
                let h = Self::parse_size(parts[1])?;
                Some((v, h, v, h))
            }
            4 => {
                let top = Self::parse_size(parts[0])?;
                let right = Self::parse_size(parts[1])?;
                let bottom = Self::parse_size(parts[2])?;
                let left = Self::parse_size(parts[3])?;
                Some((top, right, bottom, left))
            }
            _ => None,
        }
    }
}

/// Root structure for styles.toml file
#[derive(Debug, Clone, Deserialize)]
pub struct UIStylesFile {
    /// Map of class names to style specifications
    pub classes: HashMap<String, UIStyleSpec>,
}

impl UIStylesFile {
    /// Get a style class by name
    pub fn get_class(&self, name: &str) -> Option<&UIStyleSpec> {
        self.classes.get(name)
    }

    /// Get all class names
    pub fn class_names(&self) -> impl Iterator<Item = &String> {
        self.classes.keys()
    }
}

/// Load styles from TOML content string
pub fn load_styles_from_toml(content: &str) -> Result<UIStylesFile, toml::de::Error> {
    toml::from_str(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(UIStyleSpec::parse_size("60px"), Some(60.0));
        assert_eq!(UIStyleSpec::parse_size("1.5em"), Some(24.0));
        assert_eq!(UIStyleSpec::parse_size("100"), Some(100.0));
        assert_eq!(UIStyleSpec::parse_size("invalid"), None);
    }

    #[test]
    fn test_parse_color() {
        // Test rrggbb format
        let color = UIStyleSpec::parse_color("#ff8700");
        assert!(color.is_some());
        let (r, g, b, a) = color.unwrap();
        assert!((r - 1.0).abs() < 0.01);
        assert!((g - 0.529).abs() < 0.01);
        assert!((b - 0.0).abs() < 0.01);
        assert!((a - 1.0).abs() < 0.01);

        // Test rrggbbaa format
        let color = UIStyleSpec::parse_color("#1a1a1ae6");
        assert!(color.is_some());
        let (_, _, _, a) = color.unwrap();
        assert!((a - 0.9).abs() < 0.02);

        // Test rgb shorthand
        let color = UIStyleSpec::parse_color("#fff");
        assert!(color.is_some());
        let (r, g, b, _) = color.unwrap();
        assert!((r - 1.0).abs() < 0.01);
        assert!((g - 1.0).abs() < 0.01);
        assert!((b - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_padding() {
        // Single value
        assert_eq!(
            UIStyleSpec::parse_padding("16px"),
            Some((16.0, 16.0, 16.0, 16.0))
        );

        // Two values (vertical, horizontal)
        assert_eq!(
            UIStyleSpec::parse_padding("8px 16px"),
            Some((8.0, 16.0, 8.0, 16.0))
        );

        // Four values
        assert_eq!(
            UIStyleSpec::parse_padding("1px 2px 3px 4px"),
            Some((1.0, 2.0, 3.0, 4.0))
        );
    }

    #[test]
    fn test_load_styles_from_toml() {
        let toml_content = r##"
[classes.inventory-slot]
width = "60px"
height = "60px"
background = "#1a1a1a"
border = { width = "2px", color = "#ff8700", radius = "6px" }

[classes.title]
font_size = "24px"
color = "#ffffff"
margin_bottom = "12px"
"##;

        let styles = load_styles_from_toml(toml_content).unwrap();
        assert_eq!(styles.classes.len(), 2);

        let slot = styles.get_class("inventory-slot").unwrap();
        assert_eq!(slot.width.as_deref(), Some("60px"));
        assert_eq!(slot.height.as_deref(), Some("60px"));
        assert!(slot.border.is_some());

        let title = styles.get_class("title").unwrap();
        assert_eq!(title.font_size.as_deref(), Some("24px"));
        assert_eq!(title.color.as_deref(), Some("#ffffff"));
    }

    #[test]
    fn test_get_border_radius() {
        let style_with_border = UIStyleSpec {
            border: Some(BorderSpec {
                radius: Some("6px".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(style_with_border.get_border_radius(), Some("6px"));

        let style_with_shorthand = UIStyleSpec {
            border_radius: Some("4px".to_string()),
            ..Default::default()
        };
        assert_eq!(style_with_shorthand.get_border_radius(), Some("4px"));

        // border.radius takes precedence
        let style_with_both = UIStyleSpec {
            border: Some(BorderSpec {
                radius: Some("6px".to_string()),
                ..Default::default()
            }),
            border_radius: Some("4px".to_string()),
            ..Default::default()
        };
        assert_eq!(style_with_both.get_border_radius(), Some("6px"));
    }
}
