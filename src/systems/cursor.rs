//! Centralized cursor management utilities
//!
//! This module provides unified functions for cursor state management.
//!
//! ## CAD-style Controls (New)
//!
//! In CAD-style mode, the cursor is always visible. Camera rotation is controlled
//! by middle-mouse drag or Alt+left-drag.
//!
//! ## Usage Patterns (Bevy 0.18+)
//!
//! ### Opening UI (unlock cursor)
//! ```ignore
//! cursor::unlock_cursor(&mut cursor_options);
//! ```
//!
//! ### Closing UI
//! ```ignore
//! cursor::release_cursor(&mut cursor_options);
//! ```

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use tracing::debug;

use crate::systems::inventory_ui::set_ui_open_state;

/// Lock cursor for camera rotation (middle-drag mode)
///
/// In CAD-style controls, cursor remains visible but confined to window
/// during camera rotation operations.
#[inline]
pub fn lock_cursor(cursor_options: &mut CursorOptions) {
    debug!("[Cursor] lock_cursor called (CAD mode - confined, visible)");
    // Confine cursor to window during rotation, but keep it visible
    cursor_options.grab_mode = CursorGrabMode::Confined;
    // CAD-style: cursor always visible
    cursor_options.visible = true;
    set_ui_open_state(false);
}

/// Unlock cursor and show it (enter UI mode)
///
/// Use when:
/// - Opening any UI (inventory, machine, storage, command)
/// - Pausing the game with ESC
#[inline]
pub fn unlock_cursor(cursor_options: &mut CursorOptions) {
    debug!("[Cursor] unlock_cursor called");
    cursor_options.grab_mode = CursorGrabMode::None;
    cursor_options.visible = true;
    set_ui_open_state(true);
}

/// Release cursor (return to normal gameplay)
///
/// In CAD-style controls, cursor is always visible and not grabbed.
#[inline]
pub fn release_cursor(cursor_options: &mut CursorOptions) {
    debug!("[Cursor] release_cursor called");
    cursor_options.grab_mode = CursorGrabMode::None;
    cursor_options.visible = true;
    // Note: does NOT call set_ui_open_state
}

/// Check if cursor is currently grabbed (during camera rotation)
#[inline]
pub fn is_locked(cursor_options: &CursorOptions) -> bool {
    cursor_options.grab_mode != CursorGrabMode::None
}

/// Check if cursor is currently free (normal state)
#[inline]
pub fn is_unlocked(cursor_options: &CursorOptions) -> bool {
    cursor_options.grab_mode == CursorGrabMode::None
}

// =============================================================================
// Cursor Sync System (CAD-style: cursor always visible)
// =============================================================================

use crate::components::UIState;

/// Synchronize cursor state based on UIState
///
/// In CAD-style controls, the cursor is always visible. This system only
/// manages UI-related state changes (not cursor visibility).
///
/// CAD-style behavior:
/// - Cursor always visible
/// - Camera controlled by middle-drag or Alt+left-drag
pub fn sync_cursor_to_ui_state(
    ui_state: Res<UIState>,
    mut cursor_query: Query<&mut CursorOptions, With<bevy::window::PrimaryWindow>>,
) {
    let Ok(mut cursor_options) = cursor_query.single_mut() else {
        return;
    };

    // CAD-style: ensure cursor is always visible
    if !cursor_options.visible {
        debug!("[Cursor] sync: ensuring cursor visible (CAD mode)");
        cursor_options.visible = true;
    }

    // Release any grab when UI is open
    if !ui_state.is_gameplay() && is_locked(&cursor_options) {
        debug!("[Cursor] sync: releasing grab ({:?})", ui_state.current());
        release_cursor(&mut cursor_options);
    }
}

#[cfg(test)]
mod tests {
    // Note: Window cannot be easily unit tested without Bevy app context
    // These tests would require integration testing
}
