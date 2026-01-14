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

#[cfg(test)]
mod tests {
    // Note: Window cannot be easily unit tested without Bevy app context
    // These tests would require integration testing
}
