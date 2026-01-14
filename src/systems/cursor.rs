//! Centralized cursor management utilities
//!
//! This module provides unified functions for cursor state management,
//! eliminating duplicated cursor handling code across the codebase.
//!
//! ## Usage Patterns
//!
//! ### Opening UI (unlock cursor)
//! ```ignore
//! cursor::unlock_cursor(&mut window);
//! ```
//!
//! ### Closing UI (lock cursor)
//! ```ignore
//! cursor::lock_cursor(&mut window);
//! ```
//!
//! ### Checking cursor state
//! ```ignore
//! if cursor::is_locked(&window) { ... }
//! ```

use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use tracing::debug;

use crate::systems::inventory_ui::set_ui_open_state;

/// Lock cursor and hide it (return to game mode)
///
/// Use when:
/// - Closing a UI with E key (return to game)
/// - Player clicks to resume game from pause
/// - Command execution complete
///
/// Note: Windows does not support `CursorGrabMode::Locked`, so we use `Confined` instead.
/// See: https://bevy-cheatbook.github.io/window/mouse-grab.html
#[inline]
pub fn lock_cursor(window: &mut Window) {
    debug!("[Cursor] lock_cursor called");
    // Windows doesn't support Locked mode, use Confined instead
    // Confined keeps cursor within window bounds, which works well for FPS-style games
    #[cfg(target_os = "windows")]
    {
        window.cursor_options.grab_mode = CursorGrabMode::Confined;
    }
    #[cfg(not(target_os = "windows"))]
    {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
    }
    window.cursor_options.visible = false;
    set_ui_open_state(false);
}

/// Unlock cursor and show it (enter UI mode)
///
/// Use when:
/// - Opening any UI (inventory, machine, storage, command)
/// - Pausing the game with ESC
#[inline]
pub fn unlock_cursor(window: &mut Window) {
    debug!("[Cursor] unlock_cursor called");
    window.cursor_options.grab_mode = CursorGrabMode::None;
    window.cursor_options.visible = true;
    set_ui_open_state(true);
}

/// Unlock cursor without setting UI open state
///
/// Use when:
/// - Pausing the game (ESC without UI)
/// - Releasing cursor but not opening a UI
#[inline]
pub fn release_cursor(window: &mut Window) {
    debug!("[Cursor] release_cursor called");
    window.cursor_options.grab_mode = CursorGrabMode::None;
    window.cursor_options.visible = true;
    // Note: does NOT call set_ui_open_state
}

/// Check if cursor is currently locked (game mode)
#[inline]
pub fn is_locked(window: &Window) -> bool {
    window.cursor_options.grab_mode != CursorGrabMode::None
}

/// Check if cursor is currently unlocked (UI/pause mode)
#[inline]
pub fn is_unlocked(window: &Window) -> bool {
    window.cursor_options.grab_mode == CursorGrabMode::None
}

// =============================================================================
// Cursor Sync System (Best Practice: Single Source of Truth)
// =============================================================================

use crate::components::UIState;

/// Synchronize cursor state based on UIState
///
/// This system runs in PostUpdate to ensure it executes AFTER all other systems
/// that might change cursor state. UIState is the single source of truth for
/// whether cursor should be locked or unlocked.
///
/// Best Practice (Bevy Cheatbook):
/// - Lock cursor during active gameplay
/// - Unlock cursor when UI is open or game is paused
pub fn sync_cursor_to_ui_state(ui_state: Res<UIState>, mut windows: Query<&mut Window>) {
    let Ok(mut window) = windows.get_single_mut() else {
        return;
    };

    let should_lock = ui_state.is_gameplay();
    let currently_locked = is_locked(&window);

    // Only change if state differs (avoid unnecessary changes)
    if should_lock && !currently_locked {
        debug!("[Cursor] sync: locking (Gameplay)");
        lock_cursor(&mut window);
    } else if !should_lock && currently_locked {
        debug!("[Cursor] sync: releasing ({:?})", ui_state.current());
        release_cursor(&mut window);
    }
}

#[cfg(test)]
mod tests {
    // Note: Window cannot be easily unit tested without Bevy app context
    // These tests would require integration testing
}
