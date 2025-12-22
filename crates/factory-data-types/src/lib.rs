//! Factory Data Types - Shared types for Editor and Game
//!
//! This crate provides common data structures used by both:
//! - Factory Data Architect (Editor)
//! - Idle Factory (Game)
//!
//! ## TypeScript Type Generation
//!
//! To generate TypeScript types, run:
//! ```bash
//! cargo test --features typescript -p factory-data-types
//! ```
//! This will generate `.ts` files in the `bindings/` directory.

mod item;
mod recipe;
mod quest;
mod export_ts;

pub use item::*;
pub use recipe::*;
pub use quest::*;

/// Re-export serde for convenience
pub use serde;
pub use serde_json;
pub use serde_yaml;
