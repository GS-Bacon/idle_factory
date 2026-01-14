//! UI Element specification and registry
//!
//! Provides type-safe dynamic IDs for UI elements, enabling:
//! - Mod-extensible UI (e.g., "mymod:custom_hud")
//! - Automated visibility management based on UIState
//! - Test API for UI element verification

use crate::core::{StringInterner, UIElementId};
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// UI Element specification
///
/// Defines when a UI element should be visible based on the current UIContext.
#[derive(Debug, Clone)]
pub struct UIElementSpec {
    /// Full ID with namespace (e.g., "base:hotbar", "mymod:custom_hud")
    pub id: String,
    /// Display name for debugging
    pub name: String,
    /// List of UIContext names where this element should be visible
    /// e.g., ["Gameplay", "Inventory", "MachineUI"]
    pub show_in: Vec<String>,
    /// Whether this element can be interacted with (buttons, slots, etc.)
    pub interactable: bool,
    /// Whether multiple instances can exist (e.g., inventory slots)
    pub dynamic: bool,
}

/// TOML deserialization structure for ui_elements.toml
#[derive(Debug, Deserialize)]
pub struct UIElementToml {
    pub id: String,
    pub name: Option<String>,
    pub show_in: Vec<String>,
    #[serde(default)]
    pub interactable: bool,
    #[serde(default)]
    pub dynamic: bool,
}

/// TOML file structure
#[derive(Debug, Deserialize)]
pub struct UIElementsFile {
    #[serde(default)]
    pub ui_elements: Vec<UIElementToml>,
}

/// Registry for UI elements
///
/// Manages all registered UI element specifications and provides
/// lookup by UIElementId.
#[derive(Resource, Default)]
pub struct UIElementRegistry {
    /// Specs indexed by UIElementId
    specs: HashMap<UIElementId, UIElementSpec>,
    /// String interner for ID mapping
    interner: StringInterner,
}

impl UIElementRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a UI element from a TOML definition
    pub fn register(&mut self, toml: UIElementToml) -> UIElementId {
        let id = UIElementId::from_string(&toml.id, &mut self.interner);
        let spec = UIElementSpec {
            id: toml.id.clone(),
            name: toml.name.unwrap_or_else(|| toml.id.clone()),
            show_in: toml.show_in,
            interactable: toml.interactable,
            dynamic: toml.dynamic,
        };
        self.specs.insert(id, spec);
        id
    }

    /// Register a UI element directly
    pub fn register_spec(&mut self, spec: UIElementSpec) -> UIElementId {
        let id = UIElementId::from_string(&spec.id, &mut self.interner);
        self.specs.insert(id, spec);
        id
    }

    /// Get the spec for a UI element
    pub fn get(&self, id: UIElementId) -> Option<&UIElementSpec> {
        self.specs.get(&id)
    }

    /// Check if a UI element should be visible in the given context
    pub fn should_show(&self, id: UIElementId, ui_context: &str) -> bool {
        self.specs
            .get(&id)
            .map(|spec| spec.show_in.iter().any(|ctx| ctx == ui_context))
            .unwrap_or(false)
    }

    /// Get the string ID for a UIElementId
    pub fn resolve_id(&self, id: UIElementId) -> Option<&str> {
        id.to_string_id(&self.interner)
    }

    /// Get UIElementId from string (returns None if not registered)
    pub fn get_id(&self, id_str: &str) -> Option<UIElementId> {
        self.interner.get(id_str).map(UIElementId::from_raw)
    }

    /// Get all registered element IDs
    pub fn all_ids(&self) -> impl Iterator<Item = UIElementId> + '_ {
        self.specs.keys().copied()
    }

    /// Get all specs
    pub fn all_specs(&self) -> impl Iterator<Item = (&UIElementId, &UIElementSpec)> {
        self.specs.iter()
    }

    /// Number of registered elements
    pub fn len(&self) -> usize {
        self.specs.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }

    /// Get the interner (for resolving IDs to strings)
    pub fn interner(&self) -> &StringInterner {
        &self.interner
    }
}

/// Component to tag UI entities with their element ID
///
/// Attach this to UI entities so they can be managed by the visibility system.
#[derive(Component, Clone, Copy, Debug)]
pub struct UIElementTag(pub UIElementId);

impl UIElementTag {
    pub fn new(id: UIElementId) -> Self {
        Self(id)
    }

    pub fn id(&self) -> UIElementId {
        self.0
    }
}

/// Load UI elements from a TOML file
pub fn load_ui_elements_from_toml(
    toml_content: &str,
) -> Result<Vec<UIElementToml>, toml::de::Error> {
    let file: UIElementsFile = toml::from_str(toml_content)?;
    Ok(file.ui_elements)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = UIElementRegistry::new();

        let spec = UIElementSpec {
            id: "base:hotbar".to_string(),
            name: "Hotbar".to_string(),
            show_in: vec!["Gameplay".to_string(), "Inventory".to_string()],
            interactable: false,
            dynamic: false,
        };

        let id = registry.register_spec(spec);

        let retrieved = registry.get(id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Hotbar");
    }

    #[test]
    fn test_should_show() {
        let mut registry = UIElementRegistry::new();

        let id = registry.register_spec(UIElementSpec {
            id: "base:pause_menu".to_string(),
            name: "Pause Menu".to_string(),
            show_in: vec!["PauseMenu".to_string()],
            interactable: true,
            dynamic: false,
        });

        assert!(registry.should_show(id, "PauseMenu"));
        assert!(!registry.should_show(id, "Gameplay"));
        assert!(!registry.should_show(id, "Inventory"));
    }

    #[test]
    fn test_toml_parsing() {
        let toml = r#"
[[ui_elements]]
id = "base:hotbar"
name = "Hotbar"
show_in = ["Gameplay", "Inventory"]

[[ui_elements]]
id = "base:pause_menu"
show_in = ["PauseMenu"]
interactable = true
"#;

        let elements = load_ui_elements_from_toml(toml).unwrap();
        assert_eq!(elements.len(), 2);
        assert_eq!(elements[0].id, "base:hotbar");
        assert_eq!(elements[1].interactable, true);
    }

    #[test]
    fn test_registry_from_toml() {
        let mut registry = UIElementRegistry::new();

        let toml_def = UIElementToml {
            id: "base:crosshair".to_string(),
            name: Some("Crosshair".to_string()),
            show_in: vec!["Gameplay".to_string()],
            interactable: false,
            dynamic: false,
        };

        let id = registry.register(toml_def);

        assert!(registry.should_show(id, "Gameplay"));
        assert!(!registry.should_show(id, "PauseMenu"));
        assert_eq!(registry.resolve_id(id), Some("base:crosshair"));
    }
}
