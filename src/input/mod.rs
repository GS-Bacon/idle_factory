//! Input Manager - Semantic action-based input handling
//!
//! This module provides a configurable input system that maps physical inputs
//! (keyboard keys, mouse buttons) to semantic game actions.

use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// Semantic game actions that can be triggered by input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameAction {
    // Movement
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    Jump,
    Descend,

    // Camera
    LookUp,
    LookDown,
    LookLeft,
    LookRight,

    // UI
    ToggleInventory,
    TogglePause,
    ToggleGlobalInventory,
    ToggleQuest,
    OpenCommand,
    CloseUI,
    Confirm,
    Cancel,

    // Hotbar
    Hotbar1,
    Hotbar2,
    Hotbar3,
    Hotbar4,
    Hotbar5,
    Hotbar6,
    Hotbar7,
    Hotbar8,
    Hotbar9,

    // Block operations
    PrimaryAction,
    SecondaryAction,
    RotateBlock,

    // Modifier keys
    ModifierShift,

    // Debug
    ToggleDebug,

    // Command input
    DeleteChar,
}

/// Physical input binding (key or mouse button)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputBinding {
    Key(KeyCode),
    Mouse(MouseButton),
}

/// Input Manager resource that handles input mapping and state
#[derive(Resource)]
pub struct InputManager {
    /// Mapping from actions to their physical bindings
    bindings: HashMap<GameAction, Vec<InputBinding>>,

    /// Actions that were just pressed this frame
    just_pressed: HashSet<GameAction>,

    /// Actions that are currently held down
    pressed: HashSet<GameAction>,

    /// Actions that were just released this frame
    just_released: HashSet<GameAction>,

    /// Virtual just pressed actions (for testing)
    virtual_just_pressed: HashSet<GameAction>,

    /// Virtual pressed actions (for testing)
    virtual_pressed: HashSet<GameAction>,
}

impl InputManager {
    /// Create a new InputManager with the given bindings
    pub fn new(bindings: HashMap<GameAction, Vec<InputBinding>>) -> Self {
        Self {
            bindings,
            just_pressed: HashSet::new(),
            pressed: HashSet::new(),
            just_released: HashSet::new(),
            virtual_just_pressed: HashSet::new(),
            virtual_pressed: HashSet::new(),
        }
    }

    /// Check if an action was just pressed this frame
    pub fn just_pressed(&self, action: GameAction) -> bool {
        self.just_pressed.contains(&action) || self.virtual_just_pressed.contains(&action)
    }

    /// Check if an action is currently pressed
    pub fn pressed(&self, action: GameAction) -> bool {
        self.pressed.contains(&action) || self.virtual_pressed.contains(&action)
    }

    /// Check if an action was just released this frame
    pub fn just_released(&self, action: GameAction) -> bool {
        self.just_released.contains(&action)
    }

    /// Inject a press event for testing (marks as both just_pressed and pressed)
    pub fn inject_press(&mut self, action: GameAction) {
        self.virtual_just_pressed.insert(action);
        self.virtual_pressed.insert(action);
    }

    /// Inject a release event for testing
    pub fn inject_release(&mut self, action: GameAction) {
        self.virtual_pressed.remove(&action);
    }

    /// Clear all virtual input state
    pub fn clear_virtual(&mut self) {
        self.virtual_just_pressed.clear();
    }

    /// Get the bindings for an action
    pub fn get_bindings(&self, action: GameAction) -> Option<&Vec<InputBinding>> {
        self.bindings.get(&action)
    }

    /// Update internal state based on physical input
    pub(crate) fn update(
        &mut self,
        key_input: &ButtonInput<KeyCode>,
        mouse_input: &ButtonInput<MouseButton>,
    ) {
        self.just_pressed.clear();
        self.just_released.clear();

        let mut new_pressed = HashSet::new();

        for (action, bindings) in &self.bindings {
            let was_pressed = self.pressed.contains(action);
            let mut is_pressed = false;

            for binding in bindings {
                match binding {
                    InputBinding::Key(key) => {
                        if key_input.pressed(*key) {
                            is_pressed = true;
                            break;
                        }
                    }
                    InputBinding::Mouse(button) => {
                        if mouse_input.pressed(*button) {
                            is_pressed = true;
                            break;
                        }
                    }
                }
            }

            if is_pressed {
                new_pressed.insert(*action);
                if !was_pressed {
                    self.just_pressed.insert(*action);
                }
            } else if was_pressed {
                self.just_released.insert(*action);
            }
        }

        self.pressed = new_pressed;
    }
}

impl Default for InputManager {
    fn default() -> Self {
        let mut bindings: HashMap<GameAction, Vec<InputBinding>> = HashMap::new();

        // Movement
        bindings.insert(
            GameAction::MoveForward,
            vec![InputBinding::Key(KeyCode::KeyW)],
        );
        bindings.insert(
            GameAction::MoveBackward,
            vec![InputBinding::Key(KeyCode::KeyS)],
        );
        bindings.insert(GameAction::MoveLeft, vec![InputBinding::Key(KeyCode::KeyA)]);
        bindings.insert(
            GameAction::MoveRight,
            vec![InputBinding::Key(KeyCode::KeyD)],
        );
        bindings.insert(GameAction::Jump, vec![InputBinding::Key(KeyCode::Space)]);
        bindings.insert(
            GameAction::Descend,
            vec![InputBinding::Key(KeyCode::ShiftLeft)],
        );

        // Camera
        bindings.insert(
            GameAction::LookUp,
            vec![InputBinding::Key(KeyCode::ArrowUp)],
        );
        bindings.insert(
            GameAction::LookDown,
            vec![InputBinding::Key(KeyCode::ArrowDown)],
        );
        bindings.insert(
            GameAction::LookLeft,
            vec![InputBinding::Key(KeyCode::ArrowLeft)],
        );
        bindings.insert(
            GameAction::LookRight,
            vec![InputBinding::Key(KeyCode::ArrowRight)],
        );

        // UI
        bindings.insert(
            GameAction::ToggleInventory,
            vec![InputBinding::Key(KeyCode::KeyE)],
        );
        bindings.insert(
            GameAction::TogglePause,
            vec![InputBinding::Key(KeyCode::Escape)],
        );
        bindings.insert(
            GameAction::ToggleGlobalInventory,
            vec![InputBinding::Key(KeyCode::Tab)],
        );
        bindings.insert(
            GameAction::ToggleQuest,
            vec![InputBinding::Key(KeyCode::KeyQ)],
        );
        bindings.insert(
            GameAction::OpenCommand,
            vec![
                InputBinding::Key(KeyCode::KeyT),
                InputBinding::Key(KeyCode::Slash),
            ],
        );
        bindings.insert(
            GameAction::CloseUI,
            vec![InputBinding::Key(KeyCode::Escape)],
        );
        bindings.insert(GameAction::Confirm, vec![InputBinding::Key(KeyCode::Enter)]);
        bindings.insert(GameAction::Cancel, vec![InputBinding::Key(KeyCode::Escape)]);

        // Hotbar
        bindings.insert(
            GameAction::Hotbar1,
            vec![InputBinding::Key(KeyCode::Digit1)],
        );
        bindings.insert(
            GameAction::Hotbar2,
            vec![InputBinding::Key(KeyCode::Digit2)],
        );
        bindings.insert(
            GameAction::Hotbar3,
            vec![InputBinding::Key(KeyCode::Digit3)],
        );
        bindings.insert(
            GameAction::Hotbar4,
            vec![InputBinding::Key(KeyCode::Digit4)],
        );
        bindings.insert(
            GameAction::Hotbar5,
            vec![InputBinding::Key(KeyCode::Digit5)],
        );
        bindings.insert(
            GameAction::Hotbar6,
            vec![InputBinding::Key(KeyCode::Digit6)],
        );
        bindings.insert(
            GameAction::Hotbar7,
            vec![InputBinding::Key(KeyCode::Digit7)],
        );
        bindings.insert(
            GameAction::Hotbar8,
            vec![InputBinding::Key(KeyCode::Digit8)],
        );
        bindings.insert(
            GameAction::Hotbar9,
            vec![InputBinding::Key(KeyCode::Digit9)],
        );

        // Block operations
        bindings.insert(
            GameAction::PrimaryAction,
            vec![InputBinding::Mouse(MouseButton::Left)],
        );
        bindings.insert(
            GameAction::SecondaryAction,
            vec![InputBinding::Mouse(MouseButton::Right)],
        );
        bindings.insert(
            GameAction::RotateBlock,
            vec![InputBinding::Key(KeyCode::KeyR)],
        );

        // Modifier keys (both shift keys)
        bindings.insert(
            GameAction::ModifierShift,
            vec![
                InputBinding::Key(KeyCode::ShiftLeft),
                InputBinding::Key(KeyCode::ShiftRight),
            ],
        );

        // Debug
        bindings.insert(
            GameAction::ToggleDebug,
            vec![InputBinding::Key(KeyCode::F3)],
        );

        // Command input
        bindings.insert(
            GameAction::DeleteChar,
            vec![InputBinding::Key(KeyCode::Backspace)],
        );

        Self::new(bindings)
    }
}

/// System to update InputManager from physical input (runs in PreUpdate)
pub fn update_input_manager(
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut input_manager: ResMut<InputManager>,
) {
    input_manager.update(&key_input, &mouse_input);
}

/// System to clear virtual input after processing (runs in PostUpdate)
pub fn clear_virtual_input(mut input_manager: ResMut<InputManager>) {
    input_manager.clear_virtual();
}

/// テスト用入力イベント（WebSocket APIから送信）
#[derive(Event)]
pub struct TestInputEvent {
    pub action: GameAction,
}

/// TestInputEvent を InputManager に反映するシステム
pub fn process_test_input(
    mut events: EventReader<TestInputEvent>,
    mut input_manager: ResMut<InputManager>,
) {
    for event in events.read() {
        input_manager.inject_press(event.action);
    }
}

/// Plugin to add InputManager functionality
pub struct InputManagerPlugin;

impl Plugin for InputManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputManager>()
            .add_event::<TestInputEvent>()
            .add_systems(PreUpdate, process_test_input.before(update_input_manager))
            .add_systems(PreUpdate, update_input_manager)
            .add_systems(PostUpdate, clear_virtual_input);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bindings() {
        let manager = InputManager::default();

        // Check movement bindings
        assert!(manager.get_bindings(GameAction::MoveForward).is_some());
        let forward_bindings = manager.get_bindings(GameAction::MoveForward).unwrap();
        assert_eq!(forward_bindings.len(), 1);
        assert_eq!(forward_bindings[0], InputBinding::Key(KeyCode::KeyW));

        // Check multi-binding action (ModifierShift has both shifts)
        let shift_bindings = manager.get_bindings(GameAction::ModifierShift).unwrap();
        assert_eq!(shift_bindings.len(), 2);
        assert!(shift_bindings.contains(&InputBinding::Key(KeyCode::ShiftLeft)));
        assert!(shift_bindings.contains(&InputBinding::Key(KeyCode::ShiftRight)));

        // Check mouse bindings
        let primary_bindings = manager.get_bindings(GameAction::PrimaryAction).unwrap();
        assert_eq!(primary_bindings[0], InputBinding::Mouse(MouseButton::Left));
    }

    #[test]
    fn test_virtual_input() {
        let mut manager = InputManager::default();

        // Initially nothing pressed
        assert!(!manager.just_pressed(GameAction::Jump));
        assert!(!manager.pressed(GameAction::Jump));

        // Inject press
        manager.inject_press(GameAction::Jump);
        assert!(manager.just_pressed(GameAction::Jump));
        assert!(manager.pressed(GameAction::Jump));

        // Clear virtual just_pressed
        manager.clear_virtual();
        assert!(!manager.just_pressed(GameAction::Jump));
        assert!(manager.pressed(GameAction::Jump)); // Still pressed

        // Inject release
        manager.inject_release(GameAction::Jump);
        assert!(!manager.pressed(GameAction::Jump));
    }

    #[test]
    fn test_action_states() {
        let mut manager = InputManager::default();

        // Test just_released
        assert!(!manager.just_released(GameAction::MoveForward));

        // Simulate press and release using ButtonInput
        let mut key_input = ButtonInput::<KeyCode>::default();
        let mouse_input = ButtonInput::<MouseButton>::default();

        // First frame: press W
        key_input.press(KeyCode::KeyW);
        manager.update(&key_input, &mouse_input);
        assert!(manager.just_pressed(GameAction::MoveForward));
        assert!(manager.pressed(GameAction::MoveForward));
        assert!(!manager.just_released(GameAction::MoveForward));

        // Second frame: still holding W
        manager.update(&key_input, &mouse_input);
        assert!(!manager.just_pressed(GameAction::MoveForward));
        assert!(manager.pressed(GameAction::MoveForward));
        assert!(!manager.just_released(GameAction::MoveForward));

        // Third frame: release W
        key_input.release(KeyCode::KeyW);
        manager.update(&key_input, &mouse_input);
        assert!(!manager.just_pressed(GameAction::MoveForward));
        assert!(!manager.pressed(GameAction::MoveForward));
        assert!(manager.just_released(GameAction::MoveForward));
    }

    #[test]
    fn test_multi_binding() {
        let mut manager = InputManager::default();
        let mut key_input = ButtonInput::<KeyCode>::default();
        let mouse_input = ButtonInput::<MouseButton>::default();

        // OpenCommand is bound to T and Slash
        // Press T
        key_input.press(KeyCode::KeyT);
        manager.update(&key_input, &mouse_input);
        assert!(manager.just_pressed(GameAction::OpenCommand));
        assert!(manager.pressed(GameAction::OpenCommand));

        // Release T, press Slash
        key_input.release(KeyCode::KeyT);
        key_input.press(KeyCode::Slash);
        manager.update(&key_input, &mouse_input);
        // Should still be pressed (Slash is also bound)
        assert!(manager.pressed(GameAction::OpenCommand));
    }

    #[test]
    fn test_mouse_bindings() {
        let mut manager = InputManager::default();
        let key_input = ButtonInput::<KeyCode>::default();
        let mut mouse_input = ButtonInput::<MouseButton>::default();

        // Press left mouse button
        mouse_input.press(MouseButton::Left);
        manager.update(&key_input, &mouse_input);
        assert!(manager.just_pressed(GameAction::PrimaryAction));
        assert!(manager.pressed(GameAction::PrimaryAction));

        // Release left, press right
        mouse_input.release(MouseButton::Left);
        mouse_input.press(MouseButton::Right);
        manager.update(&key_input, &mouse_input);
        assert!(manager.just_released(GameAction::PrimaryAction));
        assert!(manager.just_pressed(GameAction::SecondaryAction));
    }

    #[test]
    fn test_all_hotbar_bindings() {
        let manager = InputManager::default();

        let hotbar_actions = [
            (GameAction::Hotbar1, KeyCode::Digit1),
            (GameAction::Hotbar2, KeyCode::Digit2),
            (GameAction::Hotbar3, KeyCode::Digit3),
            (GameAction::Hotbar4, KeyCode::Digit4),
            (GameAction::Hotbar5, KeyCode::Digit5),
            (GameAction::Hotbar6, KeyCode::Digit6),
            (GameAction::Hotbar7, KeyCode::Digit7),
            (GameAction::Hotbar8, KeyCode::Digit8),
            (GameAction::Hotbar9, KeyCode::Digit9),
        ];

        for (action, expected_key) in hotbar_actions {
            let bindings = manager.get_bindings(action).unwrap();
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0], InputBinding::Key(expected_key));
        }
    }
}
