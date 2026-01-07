//! Storage system for warehouses and item management

use crate::block_type::BlockType;
use crate::components::MachineSlot;
use bevy::prelude::*;

/// ストレージブロックコンポーネント
#[derive(Component, Debug, Clone)]
pub struct StorageBlock {
    /// 最大容量
    pub capacity: u32,
    /// スロット一覧
    pub slots: Vec<MachineSlot>,
    /// フィルター（許可アイテムリスト、None = 全て許可）
    pub filter: Option<Vec<BlockType>>,
    /// 入出力優先度（高いほど優先）
    pub priority: i32,
}

impl StorageBlock {
    pub fn new(capacity: u32, slot_count: usize) -> Self {
        Self {
            capacity,
            slots: vec![MachineSlot::empty(); slot_count],
            filter: None,
            priority: 0,
        }
    }

    /// フィルターを設定
    pub fn with_filter(mut self, items: Vec<BlockType>) -> Self {
        self.filter = Some(items);
        self
    }

    /// 優先度を設定
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// アイテムが受け入れ可能か確認
    pub fn accepts(&self, item: BlockType) -> bool {
        match &self.filter {
            None => true,
            Some(allowed) => allowed.contains(&item),
        }
    }

    /// 現在の使用量を計算
    pub fn used_capacity(&self) -> u32 {
        self.slots.iter().map(|s| s.count).sum()
    }

    /// 空き容量を計算
    pub fn free_capacity(&self) -> u32 {
        self.capacity.saturating_sub(self.used_capacity())
    }

    /// 空きスロット数
    pub fn empty_slots(&self) -> usize {
        self.slots
            .iter()
            .filter(|s: &&MachineSlot| s.is_empty())
            .count()
    }
}

impl Default for StorageBlock {
    fn default() -> Self {
        Self::new(100, 10)
    }
}

/// ストレージネットワーク（接続されたストレージの集合）
#[derive(Resource, Debug, Default)]
pub struct StorageNetwork {
    /// ネットワーク内のストレージEntity
    pub storages: Vec<Entity>,
    /// 自動整理が有効か
    pub auto_sort: bool,
}

impl StorageNetwork {
    pub fn new() -> Self {
        Self::default()
    }

    /// ストレージを追加
    pub fn add_storage(&mut self, entity: Entity) {
        if !self.storages.contains(&entity) {
            self.storages.push(entity);
        }
    }

    /// ストレージを削除
    pub fn remove_storage(&mut self, entity: Entity) {
        self.storages.retain(|&e| e != entity);
    }

    /// ストレージ数
    pub fn count(&self) -> usize {
        self.storages.len()
    }
}

/// ストレージサイズプリセット
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageSize {
    Small,  // 50容量, 5スロット
    Medium, // 200容量, 20スロット
    Large,  // 1000容量, 50スロット
}

impl StorageSize {
    pub fn capacity(&self) -> u32 {
        match self {
            StorageSize::Small => 50,
            StorageSize::Medium => 200,
            StorageSize::Large => 1000,
        }
    }

    pub fn slots(&self) -> usize {
        match self {
            StorageSize::Small => 5,
            StorageSize::Medium => 20,
            StorageSize::Large => 50,
        }
    }
}

pub struct StoragePlugin;

impl Plugin for StoragePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StorageNetwork>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_block_new() {
        let storage = StorageBlock::new(100, 10);
        assert_eq!(storage.capacity, 100);
        assert_eq!(storage.slots.len(), 10);
        assert_eq!(storage.free_capacity(), 100);
    }

    #[test]
    fn test_storage_filter() {
        let storage =
            StorageBlock::new(100, 10).with_filter(vec![BlockType::IronOre, BlockType::Coal]);

        assert!(storage.accepts(BlockType::IronOre));
        assert!(!storage.accepts(BlockType::Stone));
    }

    #[test]
    fn test_storage_network() {
        let mut network = StorageNetwork::new();
        let entity = Entity::from_raw(1);

        network.add_storage(entity);
        assert_eq!(network.count(), 1);

        network.add_storage(entity); // Duplicate
        assert_eq!(network.count(), 1);

        network.remove_storage(entity);
        assert_eq!(network.count(), 0);
    }

    #[test]
    fn test_storage_size() {
        assert_eq!(StorageSize::Small.capacity(), 50);
        assert_eq!(StorageSize::Medium.slots(), 20);
        assert_eq!(StorageSize::Large.capacity(), 1000);
    }

    #[test]
    fn test_storage_capacity_tracking() {
        let mut storage = StorageBlock::new(100, 10);

        // Add some items to slots
        storage.slots[0].add(BlockType::IronOre, 10);
        storage.slots[1].add(BlockType::Coal, 20);

        assert_eq!(storage.used_capacity(), 30);
        assert_eq!(storage.free_capacity(), 70);
        assert_eq!(storage.empty_slots(), 8);
    }

    #[test]
    fn test_storage_priority() {
        let storage = StorageBlock::new(100, 10).with_priority(5);

        assert_eq!(storage.priority, 5);
    }
}
