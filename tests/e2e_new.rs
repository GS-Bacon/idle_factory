//! E2E test modules
//!
//! Tests are organized by domain:
//! - common: Shared test helpers and types
//! - world: Chunk and world generation tests
//! - inventory: Inventory and slot tests
//! - machines: Machine component tests
//! - automation: Conveyor and automation tests
//! - quest: Quest and delivery tests
//! - ui: UI state tests
//! - events: Event system tests
//! - save: Save/load tests
//! - bugs: Bug regression tests

#![allow(dead_code)]

#[path = "e2e/common.rs"]
pub mod common;

#[path = "e2e/automation.rs"]
pub mod automation;
#[path = "e2e/bugs.rs"]
pub mod bugs;
#[path = "e2e/events.rs"]
pub mod events;
#[path = "e2e/inventory.rs"]
pub mod inventory;
#[path = "e2e/machines.rs"]
pub mod machines;
#[path = "e2e/quest.rs"]
pub mod quest;
#[path = "e2e/save.rs"]
pub mod save;
#[path = "e2e/ui.rs"]
pub mod ui;
#[path = "e2e/world.rs"]
pub mod world;
