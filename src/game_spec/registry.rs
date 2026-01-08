//! Game Registry - Single Source of Truth for all game data
//!
//! All descriptors (blocks, items, machines, recipes) are registered here
//! and can be accessed via O(1) lookup.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::core::items;
//!
//! // Item lookup by ItemId
//! registry.item(items::iron_ore())
//!
//! // Machine lookup by ItemId
//! registry.machine(items::furnace_block())
//! ```

use bevy::prelude::*;
use std::collections::HashMap;

use crate::block_type::{BlockCategory, BlockType};
use crate::core::{ItemId, ValidItemId};

use super::machines::MachineSpec;
use super::recipes::Recipe;

// =============================================================================
// Item Descriptor (unified block/item definition)
// =============================================================================

/// Unified descriptor for all blocks and items
#[derive(Debug, Clone)]
pub struct ItemDescriptor {
    /// Display name
    pub name: &'static str,
    /// Short name for UI (max 4 chars)
    pub short_name: &'static str,
    /// Display color
    pub color: Color,
    /// Category
    pub category: BlockCategory,
    /// Max stack size (1 for tools, 999 for materials)
    pub stack_size: u32,
    /// Can be placed in world
    pub is_placeable: bool,
    /// Base break time in seconds (before tool multiplier)
    pub hardness: f32,
    /// What this block drops when broken (None = drops itself)
    pub drops: Option<BlockType>,
}

impl ItemDescriptor {
    pub const fn new(
        name: &'static str,
        short_name: &'static str,
        color: (f32, f32, f32),
        category: BlockCategory,
        stack_size: u32,
        is_placeable: bool,
    ) -> Self {
        Self {
            name,
            short_name,
            color: Color::srgb(color.0, color.1, color.2),
            category,
            stack_size,
            is_placeable,
            hardness: 1.0, // Default hardness
            drops: None,   // Default: drops itself
        }
    }

    /// Create with custom hardness
    pub const fn with_hardness(mut self, hardness: f32) -> Self {
        self.hardness = hardness;
        self
    }

    /// Create with custom drops
    pub const fn with_drops(mut self, drops: BlockType) -> Self {
        self.drops = Some(drops);
        self
    }

    /// Get what this block drops (self if None)
    pub fn get_drops(&self, block_type: BlockType) -> BlockType {
        self.drops.unwrap_or(block_type)
    }
}

// =============================================================================
// Static Item Definitions
// =============================================================================

/// All item descriptors (indexed by BlockType)
/// Hardness values: 1.0 = terrain, 0.5 = machines, 0.0 = instant
pub const ITEM_DESCRIPTORS: &[(BlockType, ItemDescriptor)] = &[
    // Terrain (hardness 1.0)
    (
        BlockType::Stone,
        ItemDescriptor::new(
            "Stone",
            "Stn",
            (0.5, 0.5, 0.5),
            BlockCategory::Terrain,
            999,
            true,
        )
        .with_hardness(1.0),
    ),
    (
        BlockType::Grass,
        ItemDescriptor::new(
            "Grass",
            "Grs",
            (0.2, 0.8, 0.2),
            BlockCategory::Terrain,
            999,
            true,
        )
        .with_hardness(0.8),
    ),
    // Ores (hardness 1.2 - slightly harder than stone)
    (
        BlockType::IronOre,
        ItemDescriptor::new(
            "Iron Ore",
            "FeO",
            (0.6, 0.5, 0.4),
            BlockCategory::Ore,
            999,
            true,
        )
        .with_hardness(1.2),
    ),
    (
        BlockType::CopperOre,
        ItemDescriptor::new(
            "Copper Ore",
            "CuO",
            (0.7, 0.4, 0.3),
            BlockCategory::Ore,
            999,
            true,
        )
        .with_hardness(1.2),
    ),
    (
        BlockType::Coal,
        ItemDescriptor::new(
            "Coal",
            "C",
            (0.15, 0.15, 0.15),
            BlockCategory::Ore,
            999,
            true,
        )
        .with_hardness(1.0),
    ),
    // Processed (not placeable, no hardness needed)
    (
        BlockType::IronIngot,
        ItemDescriptor::new(
            "Iron Ingot",
            "Fe",
            (0.8, 0.8, 0.85),
            BlockCategory::Processed,
            999,
            false,
        ),
    ),
    (
        BlockType::CopperIngot,
        ItemDescriptor::new(
            "Copper Ingot",
            "Cu",
            (0.9, 0.5, 0.3),
            BlockCategory::Processed,
            999,
            false,
        ),
    ),
    (
        BlockType::IronDust,
        ItemDescriptor::new(
            "Iron Dust",
            "FeD",
            (0.7, 0.7, 0.75),
            BlockCategory::Processed,
            999,
            false,
        ),
    ),
    (
        BlockType::CopperDust,
        ItemDescriptor::new(
            "Copper Dust",
            "CuD",
            (0.85, 0.55, 0.4),
            BlockCategory::Processed,
            999,
            false,
        ),
    ),
    // Machines (hardness 0.5 - easier to break)
    (
        BlockType::MinerBlock,
        ItemDescriptor::new(
            "Miner",
            "Min",
            (0.8, 0.6, 0.2),
            BlockCategory::Machine,
            999,
            true,
        )
        .with_hardness(0.5),
    ),
    (
        BlockType::ConveyorBlock,
        ItemDescriptor::new(
            "Conveyor",
            "Conv",
            (0.3, 0.3, 0.35),
            BlockCategory::Machine,
            999,
            true,
        )
        .with_hardness(0.3),
    ),
    (
        BlockType::FurnaceBlock,
        ItemDescriptor::new(
            "Furnace",
            "Fur",
            (0.4, 0.3, 0.3),
            BlockCategory::Machine,
            999,
            true,
        )
        .with_hardness(0.5),
    ),
    (
        BlockType::CrusherBlock,
        ItemDescriptor::new(
            "Crusher",
            "Cru",
            (0.4, 0.3, 0.5),
            BlockCategory::Machine,
            999,
            true,
        )
        .with_hardness(0.5),
    ),
    (
        BlockType::AssemblerBlock,
        ItemDescriptor::new(
            "Assembler",
            "Asm",
            (0.3, 0.5, 0.4),
            BlockCategory::Machine,
            999,
            true,
        )
        .with_hardness(0.5),
    ),
    (
        BlockType::PlatformBlock,
        ItemDescriptor::new(
            "Platform",
            "Plt",
            (0.2, 0.5, 0.3),
            BlockCategory::Machine,
            999,
            true,
        )
        .with_hardness(0.5),
    ),
    // Tools (not placeable)
    (
        BlockType::StonePickaxe,
        ItemDescriptor::new(
            "Stone Pickaxe",
            "Pick",
            (0.6, 0.6, 0.6),
            BlockCategory::Tool,
            1,
            false,
        ),
    ),
];

// =============================================================================
// Game Registry (Bevy Resource)
// =============================================================================

/// Central registry for all game data
#[derive(Resource)]
pub struct GameRegistry {
    /// ItemId-indexed item descriptors (static, from ITEM_DESCRIPTORS)
    items: HashMap<ItemId, &'static ItemDescriptor>,
    /// ItemId-indexed item descriptors (dynamic, from Mods/TOML)
    mod_items: HashMap<ItemId, ItemDescriptor>,
    /// ItemId-indexed machine specs
    machines: HashMap<ItemId, &'static MachineSpec>,
    /// BlockType to ItemId mapping
    block_to_item: HashMap<BlockType, ItemId>,
    /// ItemId to BlockType mapping
    item_to_block: HashMap<ItemId, BlockType>,
    /// All recipes
    recipes: Vec<&'static Recipe>,
}

impl Default for GameRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl GameRegistry {
    /// Create a new registry with all static data
    pub fn new() -> Self {
        let mut items = HashMap::new();
        let mut block_to_item = HashMap::new();
        let mut item_to_block = HashMap::new();

        for (block_type, descriptor) in ITEM_DESCRIPTORS {
            let item_id = ItemId::from(*block_type);
            items.insert(item_id, descriptor);
            block_to_item.insert(*block_type, item_id);
            item_to_block.insert(item_id, *block_type);
        }

        let mut machines = HashMap::new();
        for spec in super::machines::ALL_MACHINES {
            let item_id = ItemId::from(spec.block_type);
            machines.insert(item_id, *spec);
        }

        let recipes = super::recipes::all_recipes().iter().collect();

        Self {
            items,
            mod_items: HashMap::new(),
            machines,
            block_to_item,
            item_to_block,
            recipes,
        }
    }

    // =========================================================================
    // Mod item registration (TOML/dynamic)
    // =========================================================================

    /// Register a mod item (from TOML/dynamic source)
    pub fn register_mod_item(&mut self, item_id: ItemId, descriptor: ItemDescriptor) {
        self.mod_items.insert(item_id, descriptor);
    }

    /// Get count of mod items
    pub fn mod_item_count(&self) -> usize {
        self.mod_items.len()
    }

    // =========================================================================
    /// Get item descriptor by ItemId (checks both static and mod items)
    pub fn item(&self, item_id: ItemId) -> Option<&ItemDescriptor> {
        // First check static items
        if let Some(desc) = self.items.get(&item_id).copied() {
            return Some(desc);
        }
        // Then check mod items
        self.mod_items.get(&item_id)
    }

    /// Get machine spec by ItemId
    pub fn machine(&self, item_id: ItemId) -> Option<&MachineSpec> {
        self.machines.get(&item_id).copied()
    }

    /// Check if an ItemId is registered (in static or mod items)
    pub fn is_registered(&self, item_id: ItemId) -> bool {
        self.items.contains_key(&item_id) || self.mod_items.contains_key(&item_id)
    }

    /// Get all item IDs (static + mod items)
    pub fn all_item_ids(&self) -> impl Iterator<Item = ItemId> + '_ {
        self.items
            .keys()
            .copied()
            .chain(self.mod_items.keys().copied())
    }

    /// Get all machine item IDs
    pub fn all_machine_ids(&self) -> impl Iterator<Item = ItemId> + '_ {
        self.machines.keys().copied()
    }

    // =========================================================================
    // Validated ItemId API (type-safe)
    // =========================================================================

    /// Validate an ItemId, returning a type-safe ValidItemId if it exists
    ///
    /// # Example
    /// ```rust,ignore
    /// let valid = registry.validate(items::iron_ore());
    /// assert!(valid.is_some());
    ///
    /// let unknown = ItemId::from_string("unknown:item", &mut interner);
    /// assert!(registry.validate(unknown).is_none());
    /// ```
    pub fn validate(&self, item_id: ItemId) -> Option<ValidItemId> {
        if self.is_registered(item_id) {
            Some(ValidItemId::new_unchecked(item_id))
        } else {
            None
        }
    }

    /// Validate an ItemId, returning the default (stone) if invalid
    pub fn validate_or_default(&self, item_id: ItemId) -> ValidItemId {
        self.validate(item_id)
            .unwrap_or_else(|| self.validate(crate::core::items::stone()).unwrap())
    }

    /// Get item descriptor by ValidItemId (guaranteed to succeed)
    ///
    /// This method never returns None because ValidItemId is guaranteed
    /// to exist in the registry.
    pub fn item_by_valid_id(&self, valid_id: ValidItemId) -> &ItemDescriptor {
        // SAFETY: ValidItemId can only be created via validate() which checks existence
        self.items
            .get(&valid_id.get())
            .expect("ValidItemId must exist in registry")
    }

    /// Get machine spec by ValidItemId (if it's a machine)
    pub fn machine_by_valid_id(&self, valid_id: ValidItemId) -> Option<&MachineSpec> {
        self.machines.get(&valid_id.get()).copied()
    }

    // =========================================================================
    // Conversion helpers
    // =========================================================================

    /// Convert BlockType to ItemId
    pub fn to_item_id(&self, block_type: BlockType) -> Option<ItemId> {
        self.block_to_item.get(&block_type).copied()
    }

    /// Convert ItemId to BlockType
    pub fn to_block_type(&self, item_id: ItemId) -> Option<BlockType> {
        self.item_to_block.get(&item_id).copied()
    }

    // =========================================================================
    // Common API
    // =========================================================================

    /// Get all recipes
    pub fn recipes(&self) -> &[&'static Recipe] {
        &self.recipes
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameRegistry>().add_systems(
            Startup,
            integrate_mod_items.after(crate::modding::load_base_mod),
        );
    }
}

/// Integrate items from LoadedModData into GameRegistry
///
/// Currently this only logs the loaded mod items. Full integration requires
/// mutable string interner support which will be added in a future update.
fn integrate_mod_items(mod_data: Res<crate::modding::LoadedModData>, registry: Res<GameRegistry>) {
    use bevy::log::info;

    let mod_item_count = mod_data.item_count();
    let static_item_count = registry.items.len();

    if mod_item_count > 0 {
        info!(
            "Loaded {} items from mods (static items: {})",
            mod_item_count, static_item_count
        );

        // Log which items from TOML match existing static items
        let mut matched = 0;
        for item_def in mod_data.all_items() {
            if let Some(item_id) = crate::core::items::by_name(&item_def.id) {
                if registry.items.contains_key(&item_id) {
                    matched += 1;
                }
            }
        }

        if matched > 0 {
            info!("{} mod items match existing static definitions", matched);
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::items;
    use strum::IntoEnumIterator;

    #[test]
    fn test_all_block_types_registered() {
        let registry = GameRegistry::new();
        for block_type in BlockType::iter() {
            let item_id: ItemId = block_type.into();
            assert!(
                registry.is_registered(item_id),
                "BlockType::{:?} is not registered in ITEM_DESCRIPTORS",
                block_type
            );
        }
    }

    #[test]
    fn test_item_lookup() {
        let registry = GameRegistry::new();

        let stone = registry.item(items::stone()).unwrap();
        assert_eq!(stone.name, "Stone");
        assert_eq!(stone.category, BlockCategory::Terrain);

        let iron_ingot = registry.item(items::iron_ingot()).unwrap();
        assert_eq!(iron_ingot.name, "Iron Ingot");
        assert!(!iron_ingot.is_placeable);
    }

    #[test]
    fn test_machine_lookup() {
        let registry = GameRegistry::new();

        let miner = registry.machine(items::miner_block());
        assert!(miner.is_some());
        assert_eq!(miner.unwrap().id, "miner");

        let furnace = registry.machine(items::furnace_block());
        assert!(furnace.is_some());
        assert!(furnace.unwrap().requires_fuel);

        // Non-machine should return None
        let stone = registry.machine(items::stone());
        assert!(stone.is_none());
    }

    #[test]
    fn test_stack_sizes() {
        let registry = GameRegistry::new();

        // Tools have stack size 1
        let pickaxe = registry.item(items::stone_pickaxe()).unwrap();
        assert_eq!(pickaxe.stack_size, 1);

        // Materials have stack size 999
        let iron = registry.item(items::iron_ingot()).unwrap();
        assert_eq!(iron.stack_size, 999);
    }

    #[test]
    fn test_recipes_loaded() {
        let registry = GameRegistry::new();
        assert!(!registry.recipes().is_empty());
    }

    #[test]
    fn test_hardness_values() {
        let registry = GameRegistry::new();

        // Machines have lower hardness (easier to break)
        let miner = registry.item(items::miner_block()).unwrap();
        assert_eq!(miner.hardness, 0.5);

        let conveyor = registry.item(items::conveyor_block()).unwrap();
        assert!(conveyor.hardness < 0.5); // Conveyor is even easier

        // Ores are harder than stone
        let iron_ore = registry.item(items::iron_ore()).unwrap();
        let stone = registry.item(items::stone()).unwrap();
        assert!(iron_ore.hardness > stone.hardness);
    }

    #[test]
    fn test_block_type_hardness_method() {
        // Test that BlockType.hardness() delegates to ItemDescriptor
        assert_eq!(BlockType::Stone.hardness(), 1.0);
        assert_eq!(BlockType::MinerBlock.hardness(), 0.5);
        assert!(BlockType::IronOre.hardness() > BlockType::Stone.hardness());
    }

    #[test]
    fn test_block_drops() {
        // Most blocks drop themselves
        assert_eq!(BlockType::Stone.drops(), BlockType::Stone);
        assert_eq!(BlockType::IronOre.drops(), BlockType::IronOre);
        assert_eq!(BlockType::MinerBlock.drops(), BlockType::MinerBlock);
    }

    #[test]
    fn test_all_item_ids() {
        let registry = GameRegistry::new();
        let all_ids: Vec<_> = registry.all_item_ids().collect();

        assert_eq!(all_ids.len(), 16); // All 16 base items
    }

    #[test]
    fn test_all_machine_ids() {
        let registry = GameRegistry::new();
        let machine_ids: Vec<_> = registry.all_machine_ids().collect();

        assert_eq!(machine_ids.len(), 4); // 4 machines: miner, furnace, crusher, assembler
    }

    #[test]
    fn test_block_type_item_id_conversion() {
        let registry = GameRegistry::new();

        // BlockType -> ItemId
        let stone_id = registry.to_item_id(BlockType::Stone).unwrap();
        assert_eq!(stone_id.name(), Some("base:stone"));

        // ItemId -> BlockType
        let stone_bt = registry.to_block_type(stone_id).unwrap();
        assert_eq!(stone_bt, BlockType::Stone);

        // Round-trip for all items
        for block_type in BlockType::iter() {
            let item_id = registry.to_item_id(block_type).unwrap();
            let recovered = registry.to_block_type(item_id).unwrap();
            assert_eq!(recovered, block_type);
        }
    }

    // =========================================================================
    // ValidItemId API tests (P.5)
    // =========================================================================

    #[test]
    fn test_validate_known_item() {
        let registry = GameRegistry::new();

        let valid = registry.validate(items::stone());
        assert!(valid.is_some());

        let valid_iron = registry.validate(items::iron_ore());
        assert!(valid_iron.is_some());
    }

    #[test]
    fn test_validate_unknown_item() {
        let registry = GameRegistry::new();

        // Create an unknown ItemId using a raw value that definitely doesn't exist
        // (raw value 9999 is way beyond the 16 base items)
        let unknown = crate::core::Id::new(9999);

        let valid = registry.validate(unknown);
        assert!(valid.is_none());
    }

    #[test]
    fn test_validate_or_default() {
        let registry = GameRegistry::new();

        // Known item returns itself
        let valid_iron = registry.validate_or_default(items::iron_ore());
        assert_eq!(valid_iron.item_id(), items::iron_ore());

        // Unknown item returns stone (default)
        // Use a raw value that doesn't exist in the registry
        let unknown = crate::core::Id::new(9999);
        let valid_default = registry.validate_or_default(unknown);
        assert_eq!(valid_default.item_id(), items::stone());
    }

    #[test]
    fn test_item_by_valid_id() {
        let registry = GameRegistry::new();

        let valid = registry.validate(items::iron_ore()).unwrap();
        let descriptor = registry.item_by_valid_id(valid);

        assert_eq!(descriptor.name, "Iron Ore");
        assert_eq!(descriptor.category, BlockCategory::Ore);
    }

    #[test]
    fn test_machine_by_valid_id() {
        let registry = GameRegistry::new();

        // Machine item
        let valid_miner = registry.validate(items::miner_block()).unwrap();
        let spec = registry.machine_by_valid_id(valid_miner);
        assert!(spec.is_some());
        assert_eq!(spec.unwrap().id, "miner");

        // Non-machine item
        let valid_stone = registry.validate(items::stone()).unwrap();
        let no_spec = registry.machine_by_valid_id(valid_stone);
        assert!(no_spec.is_none());
    }

    #[test]
    fn test_validate_all_base_items() {
        let registry = GameRegistry::new();

        // All base items should be validatable
        for item_id in items::all() {
            let valid = registry.validate(item_id);
            assert!(
                valid.is_some(),
                "Base item {:?} should be validatable",
                item_id.name()
            );
        }
    }
}
