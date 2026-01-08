//! Player-related modules

pub mod global_inventory;
pub mod inventory;

// Platform inventory exports
pub use global_inventory::LocalPlatform;
pub use global_inventory::LocalPlatformInventory;
pub use global_inventory::PlatformInventory;

pub use inventory::LocalPlayer;
pub use inventory::PlayerInventory;
