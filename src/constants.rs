//! Game constants

/// Chunk dimensions
pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_HEIGHT: i32 = 32;
pub const GROUND_LEVEL: i32 = 7; // Y coordinate of ground surface

/// Block size in world units
pub const BLOCK_SIZE: f32 = 1.0;

/// Player movement speed
pub const PLAYER_SPEED: f32 = 5.0;

/// Maximum distance for block interaction
pub const REACH_DISTANCE: f32 = 5.0;

/// View distance in chunks
#[cfg(target_arch = "wasm32")]
pub const VIEW_DISTANCE: i32 = 2; // 5x5 chunks for WASM

#[cfg(not(target_arch = "wasm32"))]
pub const VIEW_DISTANCE: i32 = 3; // 7x7 chunks for native (49 chunks)

/// Camera settings
pub const MOUSE_SENSITIVITY: f32 = 0.002;
pub const KEY_ROTATION_SPEED: f32 = 2.0;

/// Machine timings
pub const SMELT_TIME: f32 = 3.0;
pub const CRUSH_TIME: f32 = 4.0;
pub const MINE_TIME: f32 = 5.0;
pub const CONVEYOR_SPEED: f32 = 1.0;

/// Conveyor settings
pub const CONVEYOR_MAX_ITEMS: usize = 3; // Maximum items per conveyor
pub const CONVEYOR_ITEM_SPACING: f32 = 0.4; // Minimum spacing between items (0.0-1.0)
pub const CONVEYOR_ITEM_SIZE: f32 = 0.25; // Item visual size (fraction of BLOCK_SIZE)
pub const CONVEYOR_BELT_WIDTH: f32 = 0.8; // Belt width (fraction of BLOCK_SIZE, 8/10)
pub const CONVEYOR_BELT_HEIGHT: f32 = 0.2; // Belt height (fraction of BLOCK_SIZE)

/// Delivery platform
pub const PLATFORM_SIZE: i32 = 12;

/// Inventory
pub const HOTBAR_SLOTS: usize = 9;
pub const MAIN_INVENTORY_ROWS: usize = 3;
pub const MAIN_INVENTORY_COLS: usize = 9;
pub const MAIN_INVENTORY_SLOTS: usize = MAIN_INVENTORY_ROWS * MAIN_INVENTORY_COLS; // 27
pub const NUM_SLOTS: usize = HOTBAR_SLOTS + MAIN_INVENTORY_SLOTS; // 36 total
pub const MAX_STACK_SIZE: u32 = 999;

// ============================================================================
// UI Color Constants (Hybrid Dark Theme)
// ============================================================================

/// Panel background colors
pub mod ui_colors {
    use bevy::prelude::Color;

    /// Dark panel background with slight transparency
    pub const PANEL_BG: Color = Color::srgba(0.12, 0.12, 0.14, 0.97);
    /// Empty slot background
    pub const SLOT_EMPTY: Color = Color::srgb(0.18, 0.18, 0.20);
    /// Filled slot background
    pub const SLOT_FILLED: Color = Color::srgb(0.22, 0.22, 0.25);

    /// Border colors for depth effect
    pub const BORDER_HIGHLIGHT: Color = Color::srgb(0.4, 0.4, 0.45);
    pub const BORDER_SHADOW: Color = Color::srgb(0.08, 0.08, 0.10);
    pub const BORDER_ACTIVE: Color = Color::srgb(0.8, 0.8, 0.8);
    pub const BORDER_HOVER: Color = Color::srgb(0.7, 0.7, 0.7);

    /// Tab/button states
    pub const TAB_SELECTED: Color = Color::srgb(0.35, 0.38, 0.50);
    pub const TAB_UNSELECTED: Color = Color::srgb(0.20, 0.20, 0.25);
    pub const TAB_HOVER: Color = Color::srgb(0.28, 0.30, 0.38);

    /// Text colors
    pub const TEXT_PRIMARY: Color = Color::srgb(0.9, 0.9, 0.95);
    pub const TEXT_SECONDARY: Color = Color::srgb(0.85, 0.85, 0.9);

    /// Search/input background
    pub const INPUT_BG: Color = Color::srgb(0.15, 0.15, 0.18);

    /// Button colors
    pub const BTN_BG: Color = Color::srgb(0.25, 0.25, 0.30);
    pub const BTN_HOVER: Color = Color::srgb(0.35, 0.35, 0.40);

    /// Danger/warning colors
    pub const DANGER_BG: Color = Color::srgb(0.4, 0.1, 0.1);
    pub const DANGER_HOVER: Color = Color::srgb(0.6, 0.1, 0.1);
}
