//! Player-related modules

pub mod global_inventory;
pub mod inventory;

pub use global_inventory::GlobalInventory;
pub use inventory::sync_inventory_system;
pub use inventory::Inventory;
pub use inventory::LocalPlayer;
pub use inventory::PlayerInventory;
