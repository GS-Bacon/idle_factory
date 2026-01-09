//! Scenario-based E2E test framework
//!
//! Scenarios are defined in TOML format and describe
//! a sequence of actions and assertions.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// A test scenario
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Scenario {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<Step>,
}

/// A single step in a scenario
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "action")]
pub enum Step {
    /// Send a key/action input
    #[serde(rename = "input")]
    Input { key: String },

    /// Wait for a duration
    #[serde(rename = "wait")]
    Wait { ms: u64 },

    /// Assert a condition
    #[serde(rename = "assert")]
    Assert { condition: String },

    /// Save current state to a variable
    #[serde(rename = "save")]
    Save { variable: String, field: String },
}

impl Scenario {
    /// Load a scenario from a TOML file
    pub fn load(path: &Path) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        toml::from_str(&content).map_err(|e| format!("Failed to parse TOML: {}", e))
    }

    /// Load a scenario from a TOML string
    pub fn from_toml(toml: &str) -> Result<Self, String> {
        toml::from_str(toml).map_err(|e| format!("Failed to parse TOML: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scenario() {
        let toml = r#"
name = "inventory_toggle"
description = "Test that E key toggles inventory"

[[steps]]
action = "assert"
condition = "ui_state == Gameplay"

[[steps]]
action = "input"
key = "ToggleInventory"

[[steps]]
action = "wait"
ms = 100

[[steps]]
action = "assert"
condition = "ui_state == Inventory"
"#;
        let scenario = Scenario::from_toml(toml);
        assert!(scenario.is_ok(), "Failed to parse: {:?}", scenario.err());
        let scenario = scenario.unwrap();
        assert_eq!(scenario.name, "inventory_toggle");
        assert_eq!(scenario.steps.len(), 4);
    }

    #[test]
    fn test_parse_save_step() {
        let toml = r#"
name = "save_test"

[[steps]]
action = "save"
variable = "pos_before"
field = "player_position"
"#;
        let scenario = Scenario::from_toml(toml);
        assert!(scenario.is_ok(), "Failed to parse: {:?}", scenario.err());
        let scenario = scenario.unwrap();
        assert_eq!(scenario.steps.len(), 1);
        match &scenario.steps[0] {
            Step::Save { variable, field } => {
                assert_eq!(variable, "pos_before");
                assert_eq!(field, "player_position");
            }
            _ => panic!("Expected Save step"),
        }
    }

    #[test]
    fn test_load_from_file() {
        // Test loading from the actual scenario file
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("scenarios")
            .join("inventory_toggle.toml");
        if path.exists() {
            let scenario = Scenario::load(&path);
            assert!(scenario.is_ok(), "Failed to load: {:?}", scenario.err());
        }
    }
}
