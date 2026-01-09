//! UI condition events for visibility control
//!
//! These events allow decoupled UI visibility management.
//! When a condition changes (tutorial completed, UI opened, etc.),
//! an event is fired and the UIVisibilityController updates accordingly.

use crate::ui::visibility::{UIId, VisibilityRules};
use bevy::prelude::*;

/// Event fired when a UI-related condition changes
#[derive(Event, Clone, Debug)]
pub enum UIConditionChanged {
    /// Tutorial was completed
    TutorialCompleted,
    /// Tutorial was reset (for testing/debug)
    TutorialReset,
    /// Custom condition changed (for Mod support)
    Custom { name: String, value: bool },
}

/// Event to register a new UI's visibility rules (from Mod API)
#[derive(Event, Clone, Debug)]
pub struct UIRegistration {
    pub id: UIId,
    pub rules: VisibilityRules,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_condition_changed_clone() {
        let event = UIConditionChanged::TutorialCompleted;
        let cloned = event.clone();
        assert!(matches!(cloned, UIConditionChanged::TutorialCompleted));
    }

    #[test]
    fn test_ui_condition_changed_custom() {
        let event = UIConditionChanged::Custom {
            name: "my_mod:unlocked".to_string(),
            value: true,
        };
        if let UIConditionChanged::Custom { name, value } = event {
            assert_eq!(name, "my_mod:unlocked");
            assert!(value);
        } else {
            panic!("Expected Custom variant");
        }
    }
}
