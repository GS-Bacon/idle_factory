//! Centralized UI visibility control system
//!
//! This module provides a type-safe, event-driven approach to managing UI visibility.
//! Instead of scattered visibility logic across multiple systems, all UI visibility
//! rules are defined here and evaluated centrally.
//!
//! ## Architecture
//!
//! 1. **UIVisibilityController**: Central resource holding all visibility rules
//! 2. **UIConditionChanged**: Events that trigger condition state updates
//! 3. **ConditionKey**: Keys for the condition state cache
//! 4. **VisibilityCondition**: Individual conditions that must be met for UI visibility
//!
//! ## Example
//!
//! ```ignore
//! // Register a rule
//! controller.register(UIId::QuestPanel, VisibilityRules {
//!     conditions: vec![VisibilityCondition::TutorialCompleted],
//!     layer: UILayer::Gameplay,
//! });
//!
//! // When tutorial completes, fire event
//! events.send(UIConditionChanged::TutorialCompleted);
//!
//! // Controller updates condition state, then evaluate() returns correct visibility
//! ```

use crate::InputState;
use bevy::prelude::*;
use std::collections::HashMap;

/// UI identifier - supports both built-in and Mod UIs
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum UIId {
    // Built-in UIs
    Inventory,
    InventoryOverlay,
    GlobalInventory,
    Machine,
    PauseMenu,
    Settings,
    CommandInput,
    QuestPanel,
    TutorialPanel,
    Hotbar,
    BreakingProgress,
    // Mod UIs
    Mod { namespace: String, name: String },
}

/// Marker component for UI visibility control
///
/// Add this to any UI entity that should be controlled by UIVisibilityController.
/// The controller will automatically set Visibility based on the registered rules.
#[derive(Component, Clone, Debug)]
pub struct UIVisibilityTarget {
    pub id: UIId,
}

impl UIVisibilityTarget {
    pub fn new(id: UIId) -> Self {
        Self { id }
    }
}

impl UIId {
    /// Create a Mod UI identifier
    pub fn mod_ui(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        UIId::Mod {
            namespace: namespace.into(),
            name: name.into(),
        }
    }

    /// Convert to string identifier (for serialization/API)
    pub fn to_string_id(&self) -> String {
        match self {
            UIId::Inventory => "inventory".to_string(),
            UIId::InventoryOverlay => "inventory_overlay".to_string(),
            UIId::GlobalInventory => "global_inventory".to_string(),
            UIId::Machine => "machine".to_string(),
            UIId::PauseMenu => "pause_menu".to_string(),
            UIId::Settings => "settings".to_string(),
            UIId::CommandInput => "command_input".to_string(),
            UIId::QuestPanel => "quest_panel".to_string(),
            UIId::TutorialPanel => "tutorial_panel".to_string(),
            UIId::Hotbar => "hotbar".to_string(),
            UIId::BreakingProgress => "breaking_progress".to_string(),
            UIId::Mod { namespace, name } => format!("{}:{}", namespace, name),
        }
    }

    /// Parse from string identifier
    pub fn from_string_id(s: &str) -> Option<Self> {
        match s {
            "inventory" => Some(UIId::Inventory),
            "inventory_overlay" => Some(UIId::InventoryOverlay),
            "global_inventory" => Some(UIId::GlobalInventory),
            "machine" => Some(UIId::Machine),
            "pause_menu" => Some(UIId::PauseMenu),
            "settings" => Some(UIId::Settings),
            "command_input" => Some(UIId::CommandInput),
            "quest_panel" => Some(UIId::QuestPanel),
            "tutorial_panel" => Some(UIId::TutorialPanel),
            "hotbar" => Some(UIId::Hotbar),
            "breaking_progress" => Some(UIId::BreakingProgress),
            other => {
                let parts: Vec<&str> = other.splitn(2, ':').collect();
                if parts.len() == 2 {
                    Some(UIId::Mod {
                        namespace: parts[0].to_string(),
                        name: parts[1].to_string(),
                    })
                } else {
                    None
                }
            }
        }
    }
}

/// Key for condition state cache
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum ConditionKey {
    /// Tutorial completed condition
    TutorialCompleted,
    /// Current InputState matches
    InputStateIs(InputState),
    /// Custom condition (for Mods)
    Custom(String),
}

/// Individual visibility condition
#[derive(Clone, Debug)]
pub enum VisibilityCondition {
    /// InputState must be this value
    InputStateIs(InputState),
    /// InputState must NOT be this value
    InputStateIsNot(InputState),
    /// Tutorial must be completed
    TutorialCompleted,
    /// Tutorial must NOT be completed
    TutorialNotCompleted,
    /// Custom condition (for Mods)
    Custom(String),
    /// Custom condition must be false (for Mods)
    CustomNot(String),
    /// Always visible (no conditions)
    Always,
}

/// Z-index layer for UI stacking
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub enum UILayer {
    /// Background elements
    #[default]
    Background = 0,
    /// Gameplay HUD (hotbar, quest panel)
    Gameplay = 10,
    /// Content panels (inventory, machine UI)
    Content = 50,
    /// Overlay (darkening overlay)
    Overlay = 80,
    /// Modal dialogs (pause menu)
    Modal = 100,
    /// Notifications (update banner)
    Notification = 110,
}

/// Visibility rules for a single UI
#[derive(Clone, Debug)]
pub struct VisibilityRules {
    /// All conditions must be met for the UI to be visible
    pub conditions: Vec<VisibilityCondition>,
    /// Z-index layer
    pub layer: UILayer,
}

impl Default for VisibilityRules {
    fn default() -> Self {
        Self {
            conditions: vec![],
            layer: UILayer::Background,
        }
    }
}

/// Central controller for UI visibility
#[derive(Resource, Default)]
pub struct UIVisibilityController {
    /// Visibility rules for each UI
    rules: HashMap<UIId, VisibilityRules>,
    /// Cached condition states (updated via events)
    condition_states: HashMap<ConditionKey, bool>,
    /// Whether condition states have changed this frame
    dirty: bool,
}

impl UIVisibilityController {
    /// Create a new controller with default rules
    pub fn new() -> Self {
        Self::default()
    }

    /// Register visibility rules for a UI
    pub fn register(&mut self, id: UIId, rules: VisibilityRules) {
        self.rules.insert(id, rules);
    }

    /// Unregister visibility rules for a UI (for Mod cleanup)
    pub fn unregister(&mut self, id: &UIId) {
        self.rules.remove(id);
    }

    /// Set a condition state (marks controller as dirty)
    pub fn set_condition(&mut self, key: ConditionKey, value: bool) {
        let current = self.condition_states.get(&key).copied();
        if current != Some(value) {
            self.condition_states.insert(key, value);
            self.dirty = true;
        }
    }

    /// Get a condition state
    pub fn get_condition(&self, key: &ConditionKey) -> bool {
        self.condition_states.get(key).copied().unwrap_or(false)
    }

    /// Check if controller has been modified this frame
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear dirty flag (called after visibility update)
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Evaluate visibility for a specific UI
    pub fn evaluate(&self, id: &UIId) -> Visibility {
        let Some(rules) = self.rules.get(id) else {
            return Visibility::Hidden;
        };

        // Empty conditions = always visible
        if rules.conditions.is_empty() {
            return Visibility::Inherited;
        }

        let all_met = rules
            .conditions
            .iter()
            .all(|cond| self.evaluate_condition(cond));

        if all_met {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    }

    /// Evaluate a single condition
    fn evaluate_condition(&self, cond: &VisibilityCondition) -> bool {
        match cond {
            VisibilityCondition::InputStateIs(state) => {
                self.get_condition(&ConditionKey::InputStateIs(*state))
            }
            VisibilityCondition::InputStateIsNot(state) => {
                !self.get_condition(&ConditionKey::InputStateIs(*state))
            }
            VisibilityCondition::TutorialCompleted => {
                self.get_condition(&ConditionKey::TutorialCompleted)
            }
            VisibilityCondition::TutorialNotCompleted => {
                !self.get_condition(&ConditionKey::TutorialCompleted)
            }
            VisibilityCondition::Custom(name) => {
                self.get_condition(&ConditionKey::Custom(name.clone()))
            }
            VisibilityCondition::CustomNot(name) => {
                !self.get_condition(&ConditionKey::Custom(name.clone()))
            }
            VisibilityCondition::Always => true,
        }
    }

    /// Get the layer for a UI (for z-index ordering)
    pub fn get_layer(&self, id: &UIId) -> UILayer {
        self.rules
            .get(id)
            .map(|r| r.layer)
            .unwrap_or(UILayer::Background)
    }

    /// Get all registered UI IDs
    pub fn registered_uis(&self) -> Vec<UIId> {
        self.rules.keys().cloned().collect()
    }
}

/// Create default rules for built-in UIs
pub fn create_default_rules() -> UIVisibilityController {
    let mut controller = UIVisibilityController::new();

    // Quest Panel: visible when tutorial is completed AND in gameplay
    controller.register(
        UIId::QuestPanel,
        VisibilityRules {
            conditions: vec![
                VisibilityCondition::TutorialCompleted,
                VisibilityCondition::InputStateIs(InputState::Gameplay),
            ],
            layer: UILayer::Gameplay,
        },
    );

    // Tutorial Panel: visible when tutorial NOT completed AND in gameplay
    controller.register(
        UIId::TutorialPanel,
        VisibilityRules {
            conditions: vec![
                VisibilityCondition::TutorialNotCompleted,
                VisibilityCondition::InputStateIs(InputState::Gameplay),
            ],
            layer: UILayer::Gameplay,
        },
    );

    // Inventory: visible when in inventory state
    controller.register(
        UIId::Inventory,
        VisibilityRules {
            conditions: vec![VisibilityCondition::InputStateIs(InputState::Inventory)],
            layer: UILayer::Content,
        },
    );

    // Global Inventory: visible when in inventory state (part of inventory UI)
    controller.register(
        UIId::GlobalInventory,
        VisibilityRules {
            conditions: vec![VisibilityCondition::InputStateIs(InputState::Inventory)],
            layer: UILayer::Content,
        },
    );

    // Inventory Overlay: visible when in inventory state
    controller.register(
        UIId::InventoryOverlay,
        VisibilityRules {
            conditions: vec![VisibilityCondition::InputStateIs(InputState::Inventory)],
            layer: UILayer::Overlay,
        },
    );

    // Machine UI: visible when in MachineUI state
    // Note: The specific machine UI shown is determined by update_machine_ui_visibility
    controller.register(
        UIId::Machine,
        VisibilityRules {
            conditions: vec![VisibilityCondition::InputStateIs(InputState::MachineUI)],
            layer: UILayer::Content,
        },
    );

    // Machine UI: was NOT registered here before UI.1.4
    // Visibility is controlled directly by generic_machine_interact
    // because multiple machine types exist, each with their own UI

    // Pause Menu: visible when paused
    controller.register(
        UIId::PauseMenu,
        VisibilityRules {
            conditions: vec![VisibilityCondition::InputStateIs(InputState::Paused)],
            layer: UILayer::Modal,
        },
    );

    // Command Input: visible when command input is open
    controller.register(
        UIId::CommandInput,
        VisibilityRules {
            conditions: vec![VisibilityCondition::InputStateIs(InputState::Command)],
            layer: UILayer::Modal,
        },
    );

    // Hotbar: visible when in gameplay (not in any UI)
    controller.register(
        UIId::Hotbar,
        VisibilityRules {
            conditions: vec![VisibilityCondition::InputStateIs(InputState::Gameplay)],
            layer: UILayer::Gameplay,
        },
    );

    // Breaking Progress: always visible (controlled by content)
    controller.register(
        UIId::BreakingProgress,
        VisibilityRules {
            conditions: vec![VisibilityCondition::Always],
            layer: UILayer::Gameplay,
        },
    );

    controller
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_id_string_roundtrip() {
        let ids = vec![
            UIId::Inventory,
            UIId::QuestPanel,
            UIId::Mod {
                namespace: "test".to_string(),
                name: "panel".to_string(),
            },
        ];

        for id in ids {
            let s = id.to_string_id();
            let parsed = UIId::from_string_id(&s);
            assert_eq!(parsed, Some(id));
        }
    }

    #[test]
    fn test_condition_set_and_get() {
        let mut controller = UIVisibilityController::new();

        assert!(!controller.get_condition(&ConditionKey::TutorialCompleted));

        controller.set_condition(ConditionKey::TutorialCompleted, true);
        assert!(controller.get_condition(&ConditionKey::TutorialCompleted));
        assert!(controller.is_dirty());

        controller.clear_dirty();
        assert!(!controller.is_dirty());
    }

    #[test]
    fn test_evaluate_visibility() {
        let mut controller = UIVisibilityController::new();

        // Register quest panel with tutorial completed condition
        controller.register(
            UIId::QuestPanel,
            VisibilityRules {
                conditions: vec![VisibilityCondition::TutorialCompleted],
                layer: UILayer::Gameplay,
            },
        );

        // Initially hidden
        assert_eq!(controller.evaluate(&UIId::QuestPanel), Visibility::Hidden);

        // After tutorial completion
        controller.set_condition(ConditionKey::TutorialCompleted, true);
        assert_eq!(
            controller.evaluate(&UIId::QuestPanel),
            Visibility::Inherited
        );
    }

    #[test]
    fn test_evaluate_multiple_conditions() {
        let mut controller = UIVisibilityController::new();

        controller.register(
            UIId::QuestPanel,
            VisibilityRules {
                conditions: vec![
                    VisibilityCondition::TutorialCompleted,
                    VisibilityCondition::InputStateIs(InputState::Gameplay),
                ],
                layer: UILayer::Gameplay,
            },
        );

        // Neither condition met
        assert_eq!(controller.evaluate(&UIId::QuestPanel), Visibility::Hidden);

        // Only tutorial completed
        controller.set_condition(ConditionKey::TutorialCompleted, true);
        assert_eq!(controller.evaluate(&UIId::QuestPanel), Visibility::Hidden);

        // Both conditions met
        controller.set_condition(ConditionKey::InputStateIs(InputState::Gameplay), true);
        assert_eq!(
            controller.evaluate(&UIId::QuestPanel),
            Visibility::Inherited
        );
    }

    #[test]
    fn test_unregistered_ui_hidden() {
        let controller = UIVisibilityController::new();
        assert_eq!(controller.evaluate(&UIId::QuestPanel), Visibility::Hidden);
    }

    #[test]
    fn test_mod_ui_registration() {
        let mut controller = UIVisibilityController::new();

        let mod_ui = UIId::mod_ui("my_mod", "custom_panel");
        controller.register(
            mod_ui.clone(),
            VisibilityRules {
                conditions: vec![VisibilityCondition::Custom("my_mod:unlocked".to_string())],
                layer: UILayer::Content,
            },
        );

        assert_eq!(controller.evaluate(&mod_ui), Visibility::Hidden);

        controller.set_condition(ConditionKey::Custom("my_mod:unlocked".to_string()), true);
        assert_eq!(controller.evaluate(&mod_ui), Visibility::Inherited);
    }

    #[test]
    fn test_default_rules() {
        let controller = create_default_rules();

        // Check that all built-in UIs are registered
        assert!(controller.rules.contains_key(&UIId::QuestPanel));
        assert!(controller.rules.contains_key(&UIId::TutorialPanel));
        assert!(controller.rules.contains_key(&UIId::Inventory));
        assert!(controller.rules.contains_key(&UIId::PauseMenu));
        assert!(controller.rules.contains_key(&UIId::Hotbar));
    }

    #[test]
    fn test_empty_conditions_always_visible() {
        let mut controller = UIVisibilityController::new();

        controller.register(
            UIId::BreakingProgress,
            VisibilityRules {
                conditions: vec![],
                layer: UILayer::Gameplay,
            },
        );

        assert_eq!(
            controller.evaluate(&UIId::BreakingProgress),
            Visibility::Inherited
        );
    }
}
