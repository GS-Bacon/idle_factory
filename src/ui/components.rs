//! UI component definitions

use crate::block_type::BlockType;
use bevy::prelude::*;

/// Marker for inventory UI container
#[derive(Component)]
pub struct InventoryUI;

/// Marker for an inventory slot button (index 0-35)
#[derive(Component)]
pub struct InventorySlotUI(pub usize);

/// Marker for trash slot
#[derive(Component)]
pub struct TrashSlot;

/// Currently held item for drag and drop
#[derive(Resource, Default)]
pub struct HeldItem(pub Option<(BlockType, u32)>);

/// Marker for held item cursor display
#[derive(Component)]
pub struct HeldItemDisplay;

/// Marker for held item count text
#[derive(Component)]
pub struct HeldItemText;

/// Creative inventory item button - stores the BlockType it represents
#[derive(Component)]
pub struct CreativeItemButton(pub BlockType);

/// Marker for the creative catalog panel (right side of inventory UI)
#[derive(Component)]
pub struct CreativePanel;

/// Whether inventory is currently open
#[derive(Resource, Default)]
pub struct InventoryOpen(pub bool);

/// Command input state - tracks whether command input is open and the current text
#[derive(Resource, Default)]
pub struct CommandInputState {
    /// Whether command input is open
    pub open: bool,
    /// Current text in the command input
    pub text: String,
}

/// Marker for command input UI container
#[derive(Component)]
pub struct CommandInputUI;

/// Marker for command input text display
#[derive(Component)]
pub struct CommandInputText;

/// Marker for hotbar UI container
#[derive(Component)]
pub struct HotbarUI;

/// Marker for hotbar slot (0-8)
#[derive(Component)]
pub struct HotbarSlot(pub usize);

/// Marker for hotbar slot count text
#[derive(Component)]
pub struct HotbarSlotCount(pub usize);

/// Marker for hotbar item name text
#[derive(Component)]
pub struct HotbarItemNameText;

/// Marker for inventory tooltip
#[derive(Component)]
pub struct InventoryTooltip;

/// Tutorial shown state
#[derive(Resource, Default)]
pub struct TutorialShown(pub bool);

/// Marker for tutorial popup
#[derive(Component)]
pub struct TutorialPopup;

/// Debug HUD state
#[derive(Resource, Default)]
pub struct DebugHudState {
    pub visible: bool,
}

/// Marker for debug HUD text
#[derive(Component)]
pub struct DebugHudText;

/// Creative mode state
#[derive(Resource, Default)]
pub struct CreativeMode {
    pub enabled: bool,
}

/// Timer for continuous block break/place operations
#[derive(Resource)]
pub struct ContinuousActionTimer {
    /// Timer for block breaking
    pub break_timer: Timer,
    /// Timer for block placing
    pub place_timer: Timer,
    /// Timer for inventory shift-click
    pub inventory_timer: Timer,
}

impl Default for ContinuousActionTimer {
    fn default() -> Self {
        Self {
            break_timer: Timer::from_seconds(0.15, TimerMode::Once),
            place_timer: Timer::from_seconds(0.15, TimerMode::Once),
            inventory_timer: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }
}
