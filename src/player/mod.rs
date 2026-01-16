//! Player-related modules

pub mod inventory;
pub mod platform_inventory;

// Platform inventory exports
pub use platform_inventory::LocalPlatform;
pub use platform_inventory::LocalPlatformInventory;
pub use platform_inventory::PlatformInventory;

pub use inventory::LocalPlayer;
pub use inventory::PlayerInventory;
