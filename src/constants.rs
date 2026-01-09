//! Game constants

/// Chunk dimensions
pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_HEIGHT: i32 = 64; // 4 sections (expandable to 128 = 8 sections)
pub const SECTION_HEIGHT: i32 = 16; // Height of each section within a chunk
pub const GROUND_LEVEL: i32 = 32; // Y coordinate of ground surface (Minecraft-style)

/// Number of sections per chunk (CHUNK_HEIGHT / SECTION_HEIGHT)
pub const SECTIONS_PER_CHUNK: usize = (CHUNK_HEIGHT / SECTION_HEIGHT) as usize;

/// Block size in world units
pub const BLOCK_SIZE: f32 = 1.0;

/// Player movement speed
pub const PLAYER_SPEED: f32 = 5.0;

/// Survival mode physics constants
pub const GRAVITY: f32 = 20.0; // Gravity acceleration (blocks/sec^2)
pub const JUMP_VELOCITY: f32 = 8.0; // Initial jump velocity
pub const PLAYER_HEIGHT: f32 = 2.0; // Player collision height (2 blocks tall)
pub const PLAYER_WIDTH: f32 = 0.6; // Player collision width
pub const TERMINAL_VELOCITY: f32 = 50.0; // Max fall speed

/// Maximum distance for block interaction
pub const REACH_DISTANCE: f32 = 5.0;

/// View distance in chunks (7x7 = 49 chunks)
pub const VIEW_DISTANCE: i32 = 3;

/// Camera settings
pub const MOUSE_SENSITIVITY: f32 = 0.002;
pub const KEY_ROTATION_SPEED: f32 = 2.0;

/// Conveyor speed (synced with game_spec/machines.rs CONVEYOR.process_time)
pub const CONVEYOR_SPEED: f32 = 2.0; // Conveyor blocks/second

/// Conveyor settings
pub const CONVEYOR_MAX_ITEMS: usize = 3; // Maximum items per conveyor
pub const CONVEYOR_ITEM_SPACING: f32 = 0.4; // Minimum spacing between items (0.0-1.0)
pub const CONVEYOR_ITEM_SIZE: f32 = 0.25; // Item visual size (fraction of BLOCK_SIZE)
pub const CONVEYOR_BELT_WIDTH: f32 = 0.8; // Belt width (fraction of BLOCK_SIZE, 8/10)
pub const CONVEYOR_BELT_HEIGHT: f32 = 0.2; // Belt height (fraction of BLOCK_SIZE)

/// Delivery platform
pub const PLATFORM_SIZE: i32 = 8;

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
