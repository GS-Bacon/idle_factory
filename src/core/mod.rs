//! Core game logic (Bevy-independent)
//!
//! This module contains pure game logic that doesn't depend on Bevy.
//! It can be easily unit tested without spinning up a full ECS world.

pub mod inventory;
pub mod network;

pub use inventory::*;
pub use network::*;
