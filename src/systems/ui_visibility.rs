//! UI Visibility System
//!
//! Manages UI element visibility based on the current UIState.
//! Uses UIElementTag components and UIElementRegistry to determine
//! which elements should be visible.

use crate::components::{UIContext, UIState};
use crate::game_spec::{UIElementRegistry, UIElementTag};
use bevy::prelude::*;

/// Convert UIContext to string for registry lookup
fn ui_context_to_string(context: &UIContext) -> &'static str {
    match context {
        UIContext::Gameplay => "Gameplay",
        UIContext::Inventory => "Inventory",
        UIContext::GlobalInventory => "GlobalInventory",
        UIContext::CommandInput => "CommandInput",
        UIContext::PauseMenu => "PauseMenu",
        UIContext::Settings => "Settings",
        UIContext::Machine(_) => "MachineUI",
    }
}

/// System to update UI visibility based on UIState
///
/// This system checks each entity with a UIElementTag and updates
/// its Visibility based on the current UIContext and the element's
/// show_in configuration in the registry.
pub fn update_ui_visibility(
    ui_state: Res<UIState>,
    registry: Res<UIElementRegistry>,
    mut query: Query<(&UIElementTag, &mut Visibility)>,
) {
    // Only update when UIState changes
    if !ui_state.is_changed() {
        return;
    }

    let current_context = ui_context_to_string(&ui_state.current());

    for (tag, mut visibility) in query.iter_mut() {
        let should_show = registry.should_show(tag.0, current_context);
        *visibility = if should_show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Collect information about all UI elements for the test API
#[derive(Debug, Clone)]
pub struct UIElementInfo {
    /// Element ID string (e.g., "base:hotbar")
    pub id: String,
    /// Whether the element is currently visible
    pub visible: bool,
    /// Whether the element can be interacted with
    pub interactable: bool,
}

/// Query all UI element states for the test API
///
/// Returns information about all registered UI elements that have
/// been spawned with UIElementTag.
pub fn collect_ui_element_states(
    registry: &UIElementRegistry,
    query: &Query<(&UIElementTag, &Visibility, Option<&Interaction>)>,
) -> Vec<UIElementInfo> {
    query
        .iter()
        .filter_map(|(tag, visibility, interaction)| {
            let id_str = registry.resolve_id(tag.0)?;
            Some(UIElementInfo {
                id: id_str.to_string(),
                visible: *visibility != Visibility::Hidden,
                interactable: interaction.is_some(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_spec::{UIElementRegistry, UIElementSpec};

    #[test]
    fn test_ui_context_to_string() {
        assert_eq!(ui_context_to_string(&UIContext::Gameplay), "Gameplay");
        assert_eq!(ui_context_to_string(&UIContext::Inventory), "Inventory");
        assert_eq!(ui_context_to_string(&UIContext::PauseMenu), "PauseMenu");
        assert_eq!(
            ui_context_to_string(&UIContext::Machine(Entity::PLACEHOLDER)),
            "MachineUI"
        );
    }

    #[test]
    fn test_collect_ui_element_states() {
        // This test requires a full ECS setup, so we just test the helper functions
        let mut registry = UIElementRegistry::new();

        let spec = UIElementSpec {
            id: "base:test".to_string(),
            name: "Test".to_string(),
            show_in: vec!["Gameplay".to_string()],
            interactable: false,
            dynamic: false,
        };

        let id = registry.register_spec(spec);

        // Verify registry lookup works
        assert!(registry.should_show(id, "Gameplay"));
        assert!(!registry.should_show(id, "PauseMenu"));
    }
}
