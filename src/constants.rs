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
pub const CONVEYOR_ITEM_SPACING: f32 = 0.33; // Minimum spacing between items (0.0-1.0)

/// Delivery platform
pub const PLATFORM_SIZE: i32 = 12;
