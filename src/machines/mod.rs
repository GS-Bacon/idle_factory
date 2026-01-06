//! Machine systems: miner, furnace, crusher
//!
//! This module contains all machine-related systems.
//! Machines are production/processing units that transform items.
//!
//! Note: Conveyors are in the `logistics` module because they are
//! infrastructure (continuous item movement) rather than machines
//! (discrete item processing).

pub mod crusher;
pub mod furnace;
pub mod generic;
pub mod interaction;
pub mod miner;
pub mod output;

pub use crusher::*;
pub use furnace::*;
pub use generic::*;
pub use interaction::*;
pub use miner::*;
pub use output::*;
