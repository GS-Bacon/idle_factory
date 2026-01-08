//! Dynamic ID system with type safety via Phantom Type
//!
//! # Migration Guide: BlockType → ItemId
//!
//! This module provides the foundation for migrating from static `BlockType` enum
//! to dynamic `ItemId`. During the migration period, both systems coexist.
//!
//! ## Key Concepts
//!
//! - `Id<Category>`: Type-safe dynamic ID (e.g., `ItemId`, `MachineId`)
//! - `StringInterner`: Maps string IDs ("base:stone") to u32 for fast comparison
//! - `GameRegistry`: Bevy Resource that manages all descriptors
//!
//! ## Migration Pattern
//!
//! ```rust,ignore
//! use idle_factory::core::{ItemId, StringInterner};
//! use idle_factory::BlockType;
//!
//! // 1. Create or get interner (usually from GameRegistry)
//! let mut interner = StringInterner::new();
//!
//! // 2. Convert BlockType to ItemId
//! let block_type = BlockType::IronOre;
//! let item_id = ItemId::from_block_type(block_type, &mut interner);
//!
//! // 3. Convert back (for compatibility with existing code)
//! let recovered: Option<BlockType> = item_id.to_block_type(&interner);
//! assert_eq!(recovered, Some(BlockType::IronOre));
//!
//! // 4. Use string format for Mods
//! let mod_item = ItemId::from_string("mymod:super_ingot", &mut interner);
//! let name = mod_item.to_string_id(&interner);
//! assert_eq!(name, Some("mymod:super_ingot"));
//! ```
//!
//! ## Namespace Convention
//!
//! - Base game items: `"base:{snake_case_name}"` (e.g., "base:iron_ore")
//! - Mod items: `"{mod_id}:{item_name}"` (e.g., "mymod:super_ingot")
//!
//! ## When to Use Each
//!
//! | Use Case | Type | Example |
//! |----------|------|---------|
//! | New code | `ItemId` | `fn process(item: ItemId)` |
//! | Legacy compat | Conversion | `item_id.to_block_type()` |
//! | Mod support | `ItemId` + string | `"mymod:custom_ore"` |
//! | Save data | String ID | `"base:iron_ore"` (never save raw u32) |

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

/// 型安全な動的ID
///
/// - カテゴリ混同をコンパイル時に防止（Phantom Type）
/// - 存在保証はRegistry経由でのみ生成することで担保
/// - 高速比較（内部u32）
/// - 可読性（Interned String で "namespace:id" 形式対応）
#[derive(Copy, Clone)]
pub struct Id<Category> {
    raw: u32,
    _marker: PhantomData<Category>,
}

// Manual impls to avoid requiring bounds on Category
impl<C> PartialEq for Id<C> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<C> Eq for Id<C> {}

impl<C> Hash for Id<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<C> Id<C> {
    /// Registry経由でのみ生成可能（pub(crate)）
    #[allow(dead_code)] // Used by Registry (to be implemented)
    pub(crate) fn new(raw: u32) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    pub fn raw(&self) -> u32 {
        self.raw
    }

    /// Create an ID from a string (interning it if necessary)
    ///
    /// # Example
    /// ```rust,ignore
    /// let item_id = ItemId::from_string("base:iron_ore", &mut interner);
    /// ```
    pub fn from_string(s: &str, interner: &mut StringInterner) -> Self {
        let raw = interner.get_or_intern(s);
        Self::new(raw)
    }

    /// Get the string representation of this ID
    ///
    /// Returns None if the ID was not properly interned (should not happen in normal usage).
    pub fn to_string_id<'a>(&self, interner: &'a StringInterner) -> Option<&'a str> {
        interner.resolve(self.raw)
    }
}

impl<C> std::fmt::Debug for Id<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id({})", self.raw)
    }
}

impl<C> Default for Id<C> {
    fn default() -> Self {
        Self::new(0)
    }
}

// Serialize as raw u32 (for internal use)
// For save files, use to_string_id() instead
impl<C> Serialize for Id<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.raw.serialize(serializer)
    }
}

// Deserialize from raw u32 (for internal use)
// For save files, use from_string() instead
impl<'de, C> Deserialize<'de> for Id<C> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = u32::deserialize(deserializer)?;
        Ok(Self::new(raw))
    }
}

// カテゴリマーカー（ゼロサイズ型）
#[derive(Copy, Clone)]
pub struct ItemCategory;
#[derive(Copy, Clone)]
pub struct MachineCategory;
#[derive(Copy, Clone)]
pub struct RecipeCategory;
#[derive(Copy, Clone)]
pub struct FluidCategory;

// 型エイリアス
pub type ItemId = Id<ItemCategory>;
pub type MachineId = Id<MachineCategory>;
pub type RecipeId = Id<RecipeCategory>;
pub type FluidId = Id<FluidCategory>;

// =============================================================================
// ValidItemId - Type-safe validated ItemId
// =============================================================================

/// Type-safe ItemId that is guaranteed to exist in the registry.
///
/// This type can only be constructed via `GameRegistry::validate()`,
/// ensuring the ItemId is valid at compile-time.
///
/// # Example
/// ```rust,ignore
/// use idle_factory::core::{ItemId, ValidItemId};
/// use idle_factory::game_spec::GameRegistry;
///
/// let registry = GameRegistry::new();
/// let item_id = items::iron_ore();
///
/// // Validate returns None for unknown items
/// let valid: Option<ValidItemId> = registry.validate(item_id);
///
/// // Use the validated ID safely
/// if let Some(valid_id) = valid {
///     let descriptor = registry.item_by_valid_id(valid_id);
///     // descriptor is guaranteed to exist
/// }
/// ```
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ValidItemId(ItemId);

impl ValidItemId {
    /// Create a ValidItemId (internal use only - use GameRegistry::validate())
    ///
    /// # Safety
    /// Caller must ensure the ItemId exists in the registry.
    pub(crate) fn new_unchecked(id: ItemId) -> Self {
        Self(id)
    }

    /// Get the underlying ItemId
    #[inline]
    pub fn get(&self) -> ItemId {
        self.0
    }

    /// Get the underlying ItemId (alias for get())
    #[inline]
    pub fn item_id(&self) -> ItemId {
        self.0
    }

    /// Get the raw u32 value
    #[inline]
    pub fn raw(&self) -> u32 {
        self.0.raw()
    }

    /// Get the string name
    pub fn name(&self) -> Option<&'static str> {
        self.0.name()
    }
}

impl From<ValidItemId> for ItemId {
    fn from(valid: ValidItemId) -> Self {
        valid.0
    }
}

impl AsRef<ItemId> for ValidItemId {
    fn as_ref(&self) -> &ItemId {
        &self.0
    }
}

// =============================================================================
// BlockType <-> ItemId Conversion (Migration Helpers)
// =============================================================================

/// Base namespace for built-in items
pub const BASE_NAMESPACE: &str = "base";

impl ItemId {
    /// Try to convert a BlockType to ItemId using the global interner
    ///
    /// Returns None if the BlockType is not registered in the items module.
    /// This is the safe version that never panics.
    pub fn try_from_block_type_static(block_type: crate::block_type::BlockType) -> Option<Self> {
        let name = format!("{}", block_type); // snake_case from strum
        items::by_name(&name)
    }

    /// Convert a BlockType to ItemId using the global interner
    ///
    /// Uses the format "base:{snake_case_name}" for base game items.
    /// Falls back to stone() if the BlockType is not registered (with warning).
    ///
    /// # Example
    /// ```rust,ignore
    /// use idle_factory::{BlockType, ItemId};
    ///
    /// let item_id = ItemId::from(BlockType::IronOre);
    /// assert_eq!(item_id.name(), Some("base:iron_ore"));
    /// ```
    pub fn from_block_type_static(block_type: crate::block_type::BlockType) -> Self {
        Self::try_from_block_type_static(block_type).unwrap_or_else(|| {
            tracing::warn!(
                "BlockType::{:?} not found in items module, using stone fallback",
                block_type
            );
            items::stone()
        })
    }

    /// Get the string ID using the global interner
    pub fn name(&self) -> Option<&'static str> {
        self.to_string_id(items::interner())
    }

    /// Get the color for this item (convenience method for UI)
    ///
    /// Returns the BlockType color if this is a base game item,
    /// otherwise returns a default gray color for mod items.
    pub fn color(&self) -> bevy::prelude::Color {
        if let Ok(block_type) = crate::block_type::BlockType::try_from(*self) {
            block_type.color()
        } else {
            // Default color for mod items
            bevy::prelude::Color::srgb(0.5, 0.5, 0.5)
        }
    }

    /// Get a short display name for UI (e.g., "Fe" for iron_ore)
    pub fn short_name(&self) -> &'static str {
        if let Ok(block_type) = crate::block_type::BlockType::try_from(*self) {
            block_type.short_name()
        } else {
            // Return first 3 chars of name for mod items
            self.name().unwrap_or("???")
        }
    }

    /// Convert a BlockType to ItemId (legacy version with explicit interner)
    ///
    /// Uses the format "base:{snake_case_name}" for base game items.
    ///
    /// # Example
    /// ```rust,ignore
    /// use idle_factory::{BlockType, ItemId, StringInterner};
    ///
    /// let mut interner = StringInterner::new();
    /// let item_id = ItemId::from_block_type(BlockType::IronOre, &mut interner);
    /// assert_eq!(item_id.to_string_id(&interner), Some("base:iron_ore"));
    /// ```
    pub fn from_block_type(
        block_type: crate::block_type::BlockType,
        interner: &mut StringInterner,
    ) -> Self {
        // BlockType's Display impl (from strum) gives snake_case
        let string_id = format!("{}:{}", BASE_NAMESPACE, block_type);
        Self::from_string(&string_id, interner)
    }

    /// Try to convert this ItemId back to a BlockType
    ///
    /// Returns None if:
    /// - The ID is not in the "base:" namespace
    /// - The name doesn't match any BlockType variant
    ///
    /// # Example
    /// ```rust,ignore
    /// use idle_factory::{BlockType, ItemId, StringInterner};
    ///
    /// let mut interner = StringInterner::new();
    /// let item_id = ItemId::from_block_type(BlockType::Stone, &mut interner);
    /// assert_eq!(item_id.to_block_type(&interner), Some(BlockType::Stone));
    ///
    /// // Mod items return None
    /// let mod_item = ItemId::from_string("mymod:custom", &mut interner);
    /// assert_eq!(mod_item.to_block_type(&interner), None);
    /// ```
    pub fn to_block_type(&self, interner: &StringInterner) -> Option<crate::block_type::BlockType> {
        use std::str::FromStr;

        let string_id = interner.resolve(self.raw)?;

        // Check for "base:" prefix
        let name = string_id.strip_prefix(&format!("{}:", BASE_NAMESPACE))?;

        // Use strum's FromStr to parse the snake_case name
        crate::block_type::BlockType::from_str(name).ok()
    }

    /// Check if this ItemId represents a base game item (not a mod item)
    pub fn is_base_item(&self, interner: &StringInterner) -> bool {
        interner
            .resolve(self.raw)
            .map(|s| s.starts_with(&format!("{}:", BASE_NAMESPACE)))
            .unwrap_or(false)
    }

    /// Get the namespace of this ItemId (e.g., "base", "mymod")
    pub fn namespace<'a>(&self, interner: &'a StringInterner) -> Option<&'a str> {
        let string_id = interner.resolve(self.raw)?;
        string_id.split(':').next()
    }

    /// Get the local name of this ItemId (without namespace)
    pub fn local_name<'a>(&self, interner: &'a StringInterner) -> Option<&'a str> {
        let string_id = interner.resolve(self.raw)?;
        string_id.split(':').nth(1)
    }
}

/// String Interner for dynamic string -> ID mapping
#[derive(Default)]
pub struct StringInterner {
    to_id: HashMap<String, u32>,
    to_str: Vec<String>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create an ID for the given string
    pub fn get_or_intern(&mut self, s: &str) -> u32 {
        if let Some(&id) = self.to_id.get(s) {
            return id;
        }
        let id = self.to_str.len() as u32;
        self.to_str.push(s.to_string());
        self.to_id.insert(s.to_string(), id);
        id
    }

    /// Get ID for string (if exists)
    pub fn get(&self, s: &str) -> Option<u32> {
        self.to_id.get(s).copied()
    }

    /// Resolve ID to string
    pub fn resolve(&self, id: u32) -> Option<&str> {
        self.to_str.get(id as usize).map(|s| s.as_str())
    }

    /// Number of interned strings
    pub fn len(&self) -> usize {
        self.to_str.len()
    }

    pub fn is_empty(&self) -> bool {
        self.to_str.is_empty()
    }
}

/// スレッドセーフ版（マルチプレイ用）
pub type SharedInterner = Arc<RwLock<StringInterner>>;

// =============================================================================
// BlockType <-> ItemId Conversion Traits
// =============================================================================

impl From<crate::block_type::BlockType> for ItemId {
    fn from(block_type: crate::block_type::BlockType) -> Self {
        ItemId::from_block_type_static(block_type)
    }
}

impl TryFrom<ItemId> for crate::block_type::BlockType {
    type Error = ();

    fn try_from(item_id: ItemId) -> Result<Self, Self::Error> {
        item_id.to_block_type(items::interner()).ok_or(())
    }
}

// =============================================================================
// Base Game ItemId Constants
// =============================================================================

/// Pre-defined ItemId values for base game items.
/// These are lazily initialized on first access.
pub mod items {
    use super::*;
    use std::sync::OnceLock;

    // Global interner for static item IDs
    static INTERNER: OnceLock<StringInterner> = OnceLock::new();

    fn get_interner() -> &'static StringInterner {
        INTERNER.get_or_init(|| {
            let mut interner = StringInterner::new();
            // Pre-intern all base items
            for name in BASE_ITEM_NAMES {
                interner.get_or_intern(&format!("{}:{}", BASE_NAMESPACE, name));
            }
            interner
        })
    }

    /// All base item names (snake_case)
    const BASE_ITEM_NAMES: &[&str] = &[
        "stone",
        "grass",
        "iron_ore",
        "copper_ore",
        "coal",
        "iron_ingot",
        "copper_ingot",
        "iron_dust",
        "copper_dust",
        "miner_block",
        "conveyor_block",
        "furnace_block",
        "crusher_block",
        "assembler_block",
        "platform_block",
        "stone_pickaxe",
    ];

    /// Get an ItemId by its base name (e.g., "stone", "iron_ore")
    pub fn by_name(name: &str) -> Option<ItemId> {
        let interner = get_interner();
        let full_id = format!("{}:{}", BASE_NAMESPACE, name);
        interner.get(&full_id).map(Id::new)
    }

    /// Get the global interner (read-only)
    pub fn interner() -> &'static StringInterner {
        get_interner()
    }

    // Terrain - stone is the default fallback, must exist
    pub fn stone() -> ItemId {
        by_name("stone").expect("stone must be registered (default fallback)")
    }
    pub fn grass() -> ItemId {
        by_name("grass").unwrap_or_else(stone)
    }

    // Ores
    pub fn iron_ore() -> ItemId {
        by_name("iron_ore").unwrap_or_else(stone)
    }
    pub fn copper_ore() -> ItemId {
        by_name("copper_ore").unwrap_or_else(stone)
    }
    pub fn coal() -> ItemId {
        by_name("coal").unwrap_or_else(stone)
    }

    // Processed
    pub fn iron_ingot() -> ItemId {
        by_name("iron_ingot").unwrap_or_else(stone)
    }
    pub fn copper_ingot() -> ItemId {
        by_name("copper_ingot").unwrap_or_else(stone)
    }
    pub fn iron_dust() -> ItemId {
        by_name("iron_dust").unwrap_or_else(stone)
    }
    pub fn copper_dust() -> ItemId {
        by_name("copper_dust").unwrap_or_else(stone)
    }

    // Machines
    pub fn miner_block() -> ItemId {
        by_name("miner_block").unwrap_or_else(stone)
    }
    pub fn conveyor_block() -> ItemId {
        by_name("conveyor_block").unwrap_or_else(stone)
    }
    pub fn furnace_block() -> ItemId {
        by_name("furnace_block").unwrap_or_else(stone)
    }
    pub fn crusher_block() -> ItemId {
        by_name("crusher_block").unwrap_or_else(stone)
    }
    pub fn assembler_block() -> ItemId {
        by_name("assembler_block").unwrap_or_else(stone)
    }
    pub fn platform_block() -> ItemId {
        by_name("platform_block").unwrap_or_else(stone)
    }

    // Tools
    pub fn stone_pickaxe() -> ItemId {
        by_name("stone_pickaxe").unwrap_or_else(stone)
    }

    /// Get all base item IDs
    pub fn all() -> Vec<ItemId> {
        BASE_ITEM_NAMES
            .iter()
            .filter_map(|name| by_name(name))
            .collect()
    }

    // === Category Checks ===

    /// Check if an item is fuel (currently only coal)
    pub fn is_fuel(item_id: ItemId) -> bool {
        item_id == coal()
    }

    /// Check if an item can be smelted in a furnace
    /// (ores and dusts that produce ingots)
    pub fn is_smeltable(item_id: ItemId) -> bool {
        item_id == iron_ore()
            || item_id == copper_ore()
            || item_id == iron_dust()
            || item_id == copper_dust()
    }

    /// Check if an item can be crushed
    /// (ores that produce dust)
    pub fn is_crushable(item_id: ItemId) -> bool {
        item_id == iron_ore() || item_id == copper_ore()
    }

    /// Check if an item is a raw ore
    pub fn is_ore(item_id: ItemId) -> bool {
        item_id == iron_ore() || item_id == copper_ore() || item_id == coal()
    }

    /// Check if an item is a processed material (ingot or dust)
    pub fn is_processed(item_id: ItemId) -> bool {
        item_id == iron_ingot()
            || item_id == copper_ingot()
            || item_id == iron_dust()
            || item_id == copper_dust()
    }

    /// Check if an item is a machine block
    pub fn is_machine(item_id: ItemId) -> bool {
        item_id == miner_block()
            || item_id == conveyor_block()
            || item_id == furnace_block()
            || item_id == crusher_block()
            || item_id == assembler_block()
            || item_id == platform_block()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_items_module() {
        // All items should be accessible
        let stone = items::stone();
        let iron_ore = items::iron_ore();
        let miner = items::miner_block();

        // They should have different IDs
        assert_ne!(stone, iron_ore);
        assert_ne!(iron_ore, miner);

        // Should resolve to correct names
        let interner = items::interner();
        assert_eq!(stone.to_string_id(interner), Some("base:stone"));
        assert_eq!(iron_ore.to_string_id(interner), Some("base:iron_ore"));
        assert_eq!(miner.to_string_id(interner), Some("base:miner_block"));
    }

    #[test]
    fn test_base_items_all() {
        let all = items::all();
        assert_eq!(all.len(), 16); // All 16 base items
    }

    #[test]
    fn test_base_items_by_name() {
        assert!(items::by_name("stone").is_some());
        assert!(items::by_name("iron_ore").is_some());
        assert!(items::by_name("nonexistent").is_none());
    }

    #[test]
    fn test_id_equality() {
        let id1: ItemId = Id::new(42);
        let id2: ItemId = Id::new(42);
        let id3: ItemId = Id::new(43);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_id_different_categories() {
        let item: ItemId = Id::new(1);
        let machine: MachineId = Id::new(1);
        // These are different types - compile error if compared
        assert_eq!(item.raw(), machine.raw()); // But raw values can be compared
    }

    #[test]
    fn test_string_interner_basic() {
        let mut interner = StringInterner::new();
        let id1 = interner.get_or_intern("base:stone");
        let id2 = interner.get_or_intern("base:iron_ore");
        let id3 = interner.get_or_intern("base:stone");

        assert_eq!(id1, id3); // Same string = same ID
        assert_ne!(id1, id2);
        assert_eq!(interner.len(), 2);
    }

    #[test]
    fn test_string_interner_resolve() {
        let mut interner = StringInterner::new();
        let id = interner.get_or_intern("mymod:super_ingot");

        assert_eq!(interner.resolve(id), Some("mymod:super_ingot"));
        assert_eq!(interner.resolve(999), None);
    }

    #[test]
    fn test_string_interner_get() {
        let mut interner = StringInterner::new();
        interner.get_or_intern("test");

        assert!(interner.get("test").is_some());
        assert!(interner.get("nonexistent").is_none());
    }

    // =========================================================================
    // BlockType <-> ItemId Conversion Tests
    // =========================================================================

    #[test]
    fn test_item_id_from_block_type() {
        use crate::block_type::BlockType;

        let mut interner = StringInterner::new();

        // Test various block types
        let stone_id = ItemId::from_block_type(BlockType::Stone, &mut interner);
        let iron_ore_id = ItemId::from_block_type(BlockType::IronOre, &mut interner);
        let miner_id = ItemId::from_block_type(BlockType::MinerBlock, &mut interner);

        // Check string format
        assert_eq!(stone_id.to_string_id(&interner), Some("base:stone"));
        assert_eq!(iron_ore_id.to_string_id(&interner), Some("base:iron_ore"));
        assert_eq!(miner_id.to_string_id(&interner), Some("base:miner_block"));
    }

    #[test]
    fn test_item_id_to_block_type() {
        use crate::block_type::BlockType;

        let mut interner = StringInterner::new();

        // Convert BlockType -> ItemId -> BlockType
        let original = BlockType::IronOre;
        let item_id = ItemId::from_block_type(original, &mut interner);
        let recovered = item_id.to_block_type(&interner);

        assert_eq!(recovered, Some(BlockType::IronOre));
    }

    #[test]
    fn test_item_id_roundtrip_all_block_types() {
        use crate::block_type::BlockType;
        use strum::IntoEnumIterator;

        let mut interner = StringInterner::new();

        // All BlockType variants should roundtrip correctly
        for block_type in BlockType::iter() {
            let item_id = ItemId::from_block_type(block_type, &mut interner);
            let recovered = item_id.to_block_type(&interner);
            assert_eq!(
                recovered,
                Some(block_type),
                "Roundtrip failed for {:?}",
                block_type
            );
        }
    }

    #[test]
    fn test_item_id_mod_item_not_block_type() {
        let mut interner = StringInterner::new();

        // Mod items should return None when converting to BlockType
        let mod_item = ItemId::from_string("mymod:super_ingot", &mut interner);
        assert_eq!(mod_item.to_block_type(&interner), None);
    }

    #[test]
    fn test_item_id_is_base_item() {
        use crate::block_type::BlockType;

        let mut interner = StringInterner::new();

        let base_item = ItemId::from_block_type(BlockType::Stone, &mut interner);
        let mod_item = ItemId::from_string("mymod:custom", &mut interner);

        assert!(base_item.is_base_item(&interner));
        assert!(!mod_item.is_base_item(&interner));
    }

    #[test]
    fn test_item_id_namespace_and_local_name() {
        use crate::block_type::BlockType;

        let mut interner = StringInterner::new();

        let base_item = ItemId::from_block_type(BlockType::IronOre, &mut interner);
        assert_eq!(base_item.namespace(&interner), Some("base"));
        assert_eq!(base_item.local_name(&interner), Some("iron_ore"));

        let mod_item = ItemId::from_string("mymod:super_ingot", &mut interner);
        assert_eq!(mod_item.namespace(&interner), Some("mymod"));
        assert_eq!(mod_item.local_name(&interner), Some("super_ingot"));
    }

    #[test]
    fn test_item_id_same_block_type_same_id() {
        use crate::block_type::BlockType;

        let mut interner = StringInterner::new();

        // Same BlockType should produce same ItemId
        let id1 = ItemId::from_block_type(BlockType::Stone, &mut interner);
        let id2 = ItemId::from_block_type(BlockType::Stone, &mut interner);

        assert_eq!(id1, id2);
        assert_eq!(id1.raw(), id2.raw());
    }

    // =========================================================================
    // ValidItemId Tests
    // =========================================================================

    #[test]
    fn test_valid_item_id_methods() {
        // ValidItemId basic methods
        let stone = items::stone();
        let valid = ValidItemId::new_unchecked(stone);

        assert_eq!(valid.get(), stone);
        assert_eq!(valid.item_id(), stone);
        assert_eq!(valid.raw(), stone.raw());
        assert!(valid.name().is_some());
    }

    #[test]
    fn test_valid_item_id_equality() {
        let stone = items::stone();
        let valid1 = ValidItemId::new_unchecked(stone);
        let valid2 = ValidItemId::new_unchecked(stone);
        let iron = items::iron_ore();
        let valid3 = ValidItemId::new_unchecked(iron);

        assert_eq!(valid1, valid2);
        assert_ne!(valid1, valid3);
    }

    #[test]
    fn test_valid_item_id_into_item_id() {
        let stone = items::stone();
        let valid = ValidItemId::new_unchecked(stone);

        // Test From trait
        let converted: ItemId = valid.into();
        assert_eq!(converted, stone);

        // Test AsRef trait
        assert_eq!(*valid.as_ref(), stone);
    }

    #[test]
    fn test_valid_item_id_hash() {
        use std::collections::HashSet;

        let stone = ValidItemId::new_unchecked(items::stone());
        let iron = ValidItemId::new_unchecked(items::iron_ore());

        let mut set = HashSet::new();
        set.insert(stone);
        set.insert(iron);

        assert_eq!(set.len(), 2);
        assert!(set.contains(&stone));
        assert!(set.contains(&iron));
    }
}
