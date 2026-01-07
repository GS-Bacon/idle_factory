//! Machine systems (Phase C: Data-Driven)
//!
//! This module contains generic machine systems using the Machine component.
//! All machines (Miner, Furnace, Crusher) are now handled by generic_machine_tick.
//!
//! Note: Conveyors are in the `logistics` module because they are
//! infrastructure (continuous item movement) rather than machines
//! (discrete item processing).

pub mod generic;

pub use generic::*;
