//! Dynamic ID system with type safety via Phantom Type

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
}

impl<C> std::fmt::Debug for Id<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id({})", self.raw)
    }
}

// カテゴリマーカー（ゼロサイズ型）
pub struct ItemCategory;
pub struct MachineCategory;
pub struct RecipeCategory;
pub struct FluidCategory;

// 型エイリアス
pub type ItemId = Id<ItemCategory>;
pub type MachineId = Id<MachineCategory>;
pub type RecipeId = Id<RecipeCategory>;
pub type FluidId = Id<FluidCategory>;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
