//! Save/Load system for game data persistence

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::block_type::BlockType;

/// Save data version for compatibility checking
/// - 0.1.0: Initial format (enum-based BlockTypeSave)
/// - 0.2.0: String ID format preparation (dual support)
pub const SAVE_VERSION: &str = "0.1.0";

/// New save version with string ID format
pub const SAVE_VERSION_V2: &str = "0.2.0";

/// Auto-save interval in seconds
pub const AUTO_SAVE_INTERVAL: f32 = 60.0;

/// Save directory name
pub const SAVE_DIR: &str = "saves";

/// Main save data structure containing all game state
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveData {
    /// Save format version
    pub version: String,
    /// Timestamp when saved (Unix milliseconds)
    pub timestamp: u64,
    /// Player state
    pub player: PlayerSaveData,
    /// Inventory state
    pub inventory: InventorySaveData,
    /// Global inventory (machines and items) - v0.2 feature
    #[serde(default)]
    pub global_inventory: GlobalInventorySaveData,
    /// World modifications
    pub world: WorldSaveData,
    /// All machines in the world
    pub machines: Vec<MachineSaveData>,
    /// Quest progress
    pub quests: QuestSaveData,
    /// Game mode
    pub mode: GameModeSaveData,
}

/// Player position and rotation
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerSaveData {
    pub position: Vec3Save,
    pub rotation: CameraRotation,
}

/// Vec3 wrapper for serialization
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Vec3Save {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for Vec3Save {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<Vec3Save> for Vec3 {
    fn from(v: Vec3Save) -> Self {
        Vec3::new(v.x, v.y, v.z)
    }
}

/// IVec3 wrapper for serialization
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IVec3Save {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl From<IVec3> for IVec3Save {
    fn from(v: IVec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<IVec3Save> for IVec3 {
    fn from(v: IVec3Save) -> Self {
        IVec3::new(v.x, v.y, v.z)
    }
}

/// Camera rotation (pitch/yaw)
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct CameraRotation {
    pub pitch: f32,
    pub yaw: f32,
}

/// Inventory save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InventorySaveData {
    pub selected_slot: usize,
    pub slots: Vec<Option<ItemStack>>,
}

/// Global inventory save data (v0.2)
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GlobalInventorySaveData {
    /// Items stored in global inventory: BlockType -> count
    pub items: HashMap<BlockTypeSave, u32>,
}

/// Single item stack
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemStack {
    pub item_type: BlockTypeSave,
    pub count: u32,
}

/// BlockType wrapper for serialization (string-based for forward compatibility)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum BlockTypeSave {
    Stone,
    Grass,
    IronOre,
    Coal,
    IronIngot,
    MinerBlock,
    ConveyorBlock,
    CopperOre,
    CopperIngot,
    CrusherBlock,
    FurnaceBlock,
    StonePickaxe,
    AssemblerBlock,
    IronDust,
    CopperDust,
    PlatformBlock,
}

impl From<BlockType> for BlockTypeSave {
    fn from(bt: BlockType) -> Self {
        match bt {
            BlockType::Stone => BlockTypeSave::Stone,
            BlockType::Grass => BlockTypeSave::Grass,
            BlockType::IronOre => BlockTypeSave::IronOre,
            BlockType::Coal => BlockTypeSave::Coal,
            BlockType::IronIngot => BlockTypeSave::IronIngot,
            BlockType::MinerBlock => BlockTypeSave::MinerBlock,
            BlockType::ConveyorBlock => BlockTypeSave::ConveyorBlock,
            BlockType::CopperOre => BlockTypeSave::CopperOre,
            BlockType::CopperIngot => BlockTypeSave::CopperIngot,
            BlockType::CrusherBlock => BlockTypeSave::CrusherBlock,
            BlockType::FurnaceBlock => BlockTypeSave::FurnaceBlock,
            BlockType::StonePickaxe => BlockTypeSave::StonePickaxe,
            BlockType::AssemblerBlock => BlockTypeSave::AssemblerBlock,
            BlockType::IronDust => BlockTypeSave::IronDust,
            BlockType::CopperDust => BlockTypeSave::CopperDust,
            BlockType::PlatformBlock => BlockTypeSave::PlatformBlock,
        }
    }
}

impl From<BlockTypeSave> for BlockType {
    fn from(bt: BlockTypeSave) -> Self {
        match bt {
            BlockTypeSave::Stone => BlockType::Stone,
            BlockTypeSave::Grass => BlockType::Grass,
            BlockTypeSave::IronOre => BlockType::IronOre,
            BlockTypeSave::Coal => BlockType::Coal,
            BlockTypeSave::IronIngot => BlockType::IronIngot,
            BlockTypeSave::MinerBlock => BlockType::MinerBlock,
            BlockTypeSave::ConveyorBlock => BlockType::ConveyorBlock,
            BlockTypeSave::CopperOre => BlockType::CopperOre,
            BlockTypeSave::CopperIngot => BlockType::CopperIngot,
            BlockTypeSave::CrusherBlock => BlockType::CrusherBlock,
            BlockTypeSave::FurnaceBlock => BlockType::FurnaceBlock,
            BlockTypeSave::StonePickaxe => BlockType::StonePickaxe,
            BlockTypeSave::AssemblerBlock => BlockType::AssemblerBlock,
            BlockTypeSave::IronDust => BlockType::IronDust,
            BlockTypeSave::CopperDust => BlockType::CopperDust,
            BlockTypeSave::PlatformBlock => BlockType::PlatformBlock,
        }
    }
}

// =============================================================================
// String ID Format (V2) - Preparation for future migration
// =============================================================================

/// Default namespace for base game items
pub const DEFAULT_NAMESPACE: &str = "base";

impl BlockTypeSave {
    /// Convert to string ID format ("namespace:id")
    /// Uses "base" namespace for all vanilla items
    pub fn to_string_id(&self) -> String {
        let id = match self {
            BlockTypeSave::Stone => "stone",
            BlockTypeSave::Grass => "grass",
            BlockTypeSave::IronOre => "iron_ore",
            BlockTypeSave::Coal => "coal",
            BlockTypeSave::IronIngot => "iron_ingot",
            BlockTypeSave::MinerBlock => "miner_block",
            BlockTypeSave::ConveyorBlock => "conveyor_block",
            BlockTypeSave::CopperOre => "copper_ore",
            BlockTypeSave::CopperIngot => "copper_ingot",
            BlockTypeSave::CrusherBlock => "crusher_block",
            BlockTypeSave::FurnaceBlock => "furnace_block",
            BlockTypeSave::StonePickaxe => "stone_pickaxe",
            BlockTypeSave::AssemblerBlock => "assembler_block",
            BlockTypeSave::IronDust => "iron_dust",
            BlockTypeSave::CopperDust => "copper_dust",
            BlockTypeSave::PlatformBlock => "platform_block",
        };
        format!("{}:{}", DEFAULT_NAMESPACE, id)
    }

    /// Parse from string ID format ("namespace:id")
    /// Returns None if the format is invalid or unknown
    pub fn from_string_id(s: &str) -> Option<Self> {
        // Parse "namespace:id" format
        let (namespace, id) = if let Some(colon_pos) = s.find(':') {
            (&s[..colon_pos], &s[colon_pos + 1..])
        } else {
            // Fallback: treat as just ID with default namespace
            (DEFAULT_NAMESPACE, s)
        };

        // Only support base namespace for now
        if namespace != DEFAULT_NAMESPACE {
            return None;
        }

        match id {
            "stone" => Some(BlockTypeSave::Stone),
            "grass" => Some(BlockTypeSave::Grass),
            "iron_ore" => Some(BlockTypeSave::IronOre),
            "coal" => Some(BlockTypeSave::Coal),
            "iron_ingot" => Some(BlockTypeSave::IronIngot),
            "miner_block" | "miner" => Some(BlockTypeSave::MinerBlock),
            "conveyor_block" | "conveyor" => Some(BlockTypeSave::ConveyorBlock),
            "copper_ore" => Some(BlockTypeSave::CopperOre),
            "copper_ingot" => Some(BlockTypeSave::CopperIngot),
            "crusher_block" | "crusher" => Some(BlockTypeSave::CrusherBlock),
            "furnace_block" | "furnace" => Some(BlockTypeSave::FurnaceBlock),
            "stone_pickaxe" | "pickaxe" => Some(BlockTypeSave::StonePickaxe),
            "assembler_block" | "assembler" => Some(BlockTypeSave::AssemblerBlock),
            "iron_dust" => Some(BlockTypeSave::IronDust),
            "copper_dust" => Some(BlockTypeSave::CopperDust),
            "platform_block" | "platform" => Some(BlockTypeSave::PlatformBlock),
            _ => None,
        }
    }
}

/// New format item stack using string IDs (V2)
/// This allows for mod items and future extensibility
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ItemStackV2 {
    /// Item ID in "namespace:id" format (e.g., "base:iron_ore", "mymod:copper_plate")
    pub item_id: String,
    /// Number of items in this stack
    pub count: u32,
}

impl ItemStackV2 {
    /// Create a new item stack with the given ID and count
    pub fn new(item_id: impl Into<String>, count: u32) -> Self {
        Self {
            item_id: item_id.into(),
            count,
        }
    }

    /// Try to convert to the old enum-based format
    /// Returns None if the item ID is not a known base game item
    pub fn to_legacy(&self) -> Option<ItemStack> {
        BlockTypeSave::from_string_id(&self.item_id).map(|item_type| ItemStack {
            item_type,
            count: self.count,
        })
    }
}

impl From<ItemStack> for ItemStackV2 {
    fn from(old: ItemStack) -> Self {
        ItemStackV2 {
            item_id: old.item_type.to_string_id(),
            count: old.count,
        }
    }
}

impl From<&ItemStack> for ItemStackV2 {
    fn from(old: &ItemStack) -> Self {
        ItemStackV2 {
            item_id: old.item_type.to_string_id(),
            count: old.count,
        }
    }
}

/// Convert V2 format back to legacy (if possible)
impl TryFrom<ItemStackV2> for ItemStack {
    type Error = String;

    fn try_from(v2: ItemStackV2) -> Result<Self, Self::Error> {
        BlockTypeSave::from_string_id(&v2.item_id)
            .map(|item_type| ItemStack {
                item_type,
                count: v2.count,
            })
            .ok_or_else(|| format!("Unknown item ID: {}", v2.item_id))
    }
}

/// Helper to convert BlockType directly to string ID
impl BlockType {
    /// Convert to save string ID format ("base:stone", "base:iron_ore", etc.)
    pub fn to_save_string_id(&self) -> String {
        let save: BlockTypeSave = (*self).into();
        save.to_string_id()
    }

    /// Parse from save string ID format
    pub fn from_save_string_id(s: &str) -> Option<Self> {
        BlockTypeSave::from_string_id(s).map(|bt| bt.into())
    }
}

// =============================================================================
// V2 Save Data Structures (String ID based)
// =============================================================================

/// V2 Inventory save data using string IDs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InventorySaveDataV2 {
    pub selected_slot: usize,
    pub slots: Vec<Option<ItemStackV2>>,
}

impl From<InventorySaveData> for InventorySaveDataV2 {
    fn from(v1: InventorySaveData) -> Self {
        Self {
            selected_slot: v1.selected_slot,
            slots: v1
                .slots
                .into_iter()
                .map(|opt| opt.map(Into::into))
                .collect(),
        }
    }
}

/// V2 Global inventory save data using string IDs
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GlobalInventorySaveDataV2 {
    /// Items stored: "namespace:id" -> count
    pub items: HashMap<String, u32>,
}

impl From<GlobalInventorySaveData> for GlobalInventorySaveDataV2 {
    fn from(v1: GlobalInventorySaveData) -> Self {
        Self {
            items: v1
                .items
                .into_iter()
                .map(|(bt, count)| (bt.to_string_id(), count))
                .collect(),
        }
    }
}

/// V2 World save data using string IDs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldSaveDataV2 {
    /// Modified blocks: "x,y,z" -> Some("namespace:id") for placed, None for removed
    pub modified_blocks: HashMap<String, Option<String>>,
}

impl WorldSaveDataV2 {
    /// Convert IVec3 to string key for JSON serialization
    pub fn pos_to_key(pos: IVec3) -> String {
        format!("{},{},{}", pos.x, pos.y, pos.z)
    }

    /// Parse string key back to IVec3
    pub fn key_to_pos(key: &str) -> Option<IVec3> {
        let parts: Vec<&str> = key.split(',').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(IVec3::new(
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ))
    }
}

impl From<WorldSaveData> for WorldSaveDataV2 {
    fn from(v1: WorldSaveData) -> Self {
        Self {
            modified_blocks: v1
                .modified_blocks
                .into_iter()
                .map(|(pos, opt_bt)| (pos, opt_bt.map(|bt| bt.to_string_id())))
                .collect(),
        }
    }
}

/// V2 Single item on conveyor using string ID
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConveyorItemSaveV2 {
    pub item_id: String,
    pub progress: f32,
    pub lateral_offset: f32,
}

impl From<ConveyorItemSave> for ConveyorItemSaveV2 {
    fn from(v1: ConveyorItemSave) -> Self {
        Self {
            item_id: v1.item_type.to_string_id(),
            progress: v1.progress,
            lateral_offset: v1.lateral_offset,
        }
    }
}

/// V2 Miner save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinerSaveDataV2 {
    pub position: IVec3Save,
    pub progress: f32,
    pub buffer: Option<ItemStackV2>,
}

impl From<MinerSaveData> for MinerSaveDataV2 {
    fn from(v1: MinerSaveData) -> Self {
        Self {
            position: v1.position,
            progress: v1.progress,
            buffer: v1.buffer.map(Into::into),
        }
    }
}

/// V2 Conveyor save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConveyorSaveDataV2 {
    pub position: IVec3Save,
    pub direction: DirectionSave,
    pub shape: ConveyorShapeSave,
    pub items: Vec<ConveyorItemSaveV2>,
    pub last_output_index: usize,
    pub last_input_source: usize,
}

impl From<ConveyorSaveData> for ConveyorSaveDataV2 {
    fn from(v1: ConveyorSaveData) -> Self {
        Self {
            position: v1.position,
            direction: v1.direction,
            shape: v1.shape,
            items: v1.items.into_iter().map(Into::into).collect(),
            last_output_index: v1.last_output_index,
            last_input_source: v1.last_input_source,
        }
    }
}

/// V2 Furnace save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FurnaceSaveDataV2 {
    pub position: IVec3Save,
    pub fuel: u32,
    pub input: Option<ItemStackV2>,
    pub output: Option<ItemStackV2>,
    pub progress: f32,
}

impl From<FurnaceSaveData> for FurnaceSaveDataV2 {
    fn from(v1: FurnaceSaveData) -> Self {
        Self {
            position: v1.position,
            fuel: v1.fuel,
            input: v1.input.map(Into::into),
            output: v1.output.map(Into::into),
            progress: v1.progress,
        }
    }
}

/// V2 Crusher save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CrusherSaveDataV2 {
    pub position: IVec3Save,
    pub input: Option<ItemStackV2>,
    pub output: Option<ItemStackV2>,
    pub progress: f32,
}

impl From<CrusherSaveData> for CrusherSaveDataV2 {
    fn from(v1: CrusherSaveData) -> Self {
        Self {
            position: v1.position,
            input: v1.input.map(Into::into),
            output: v1.output.map(Into::into),
            progress: v1.progress,
        }
    }
}

/// V2 Machine save data (all machine types)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum MachineSaveDataV2 {
    Miner(MinerSaveDataV2),
    Conveyor(ConveyorSaveDataV2),
    Furnace(FurnaceSaveDataV2),
    Crusher(CrusherSaveDataV2),
}

impl From<MachineSaveData> for MachineSaveDataV2 {
    fn from(v1: MachineSaveData) -> Self {
        match v1 {
            MachineSaveData::Miner(m) => MachineSaveDataV2::Miner(m.into()),
            MachineSaveData::Conveyor(c) => MachineSaveDataV2::Conveyor(c.into()),
            MachineSaveData::Furnace(f) => MachineSaveDataV2::Furnace(f.into()),
            MachineSaveData::Crusher(c) => MachineSaveDataV2::Crusher(c.into()),
        }
    }
}

/// V2 Quest save data using string IDs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuestSaveDataV2 {
    pub current_index: usize,
    pub completed: bool,
    pub rewards_claimed: bool,
    /// Items delivered: "namespace:id" -> count
    pub delivered: HashMap<String, u32>,
}

impl From<QuestSaveData> for QuestSaveDataV2 {
    fn from(v1: QuestSaveData) -> Self {
        Self {
            current_index: v1.current_index,
            completed: v1.completed,
            rewards_claimed: v1.rewards_claimed,
            delivered: v1
                .delivered
                .into_iter()
                .map(|(bt, count)| (bt.to_string_id(), count))
                .collect(),
        }
    }
}

/// V2 Main save data structure using string IDs throughout
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveDataV2 {
    /// Save format version (should be "0.2.0" or later)
    pub version: String,
    /// Timestamp when saved (Unix milliseconds)
    pub timestamp: u64,
    /// Player state
    pub player: PlayerSaveData,
    /// Inventory state
    pub inventory: InventorySaveDataV2,
    /// Global inventory
    #[serde(default)]
    pub global_inventory: GlobalInventorySaveDataV2,
    /// World modifications
    pub world: WorldSaveDataV2,
    /// All machines in the world
    pub machines: Vec<MachineSaveDataV2>,
    /// Quest progress
    pub quests: QuestSaveDataV2,
    /// Game mode
    pub mode: GameModeSaveData,
}

impl From<SaveData> for SaveDataV2 {
    fn from(v1: SaveData) -> Self {
        Self {
            version: SAVE_VERSION_V2.to_string(),
            timestamp: v1.timestamp,
            player: v1.player,
            inventory: v1.inventory.into(),
            global_inventory: v1.global_inventory.into(),
            world: v1.world.into(),
            machines: v1.machines.into_iter().map(Into::into).collect(),
            quests: v1.quests.into(),
            mode: v1.mode,
        }
    }
}

// =============================================================================
// V2 -> V1 Conversion (for backward compatibility during transition)
// =============================================================================

impl TryFrom<InventorySaveDataV2> for InventorySaveData {
    type Error = String;

    fn try_from(v2: InventorySaveDataV2) -> Result<Self, Self::Error> {
        let mut slots = Vec::new();
        for opt in v2.slots {
            match opt {
                Some(stack) => {
                    let legacy: ItemStack = stack.try_into()?;
                    slots.push(Some(legacy));
                }
                None => slots.push(None),
            }
        }
        Ok(Self {
            selected_slot: v2.selected_slot,
            slots,
        })
    }
}

impl TryFrom<GlobalInventorySaveDataV2> for GlobalInventorySaveData {
    type Error = String;

    fn try_from(v2: GlobalInventorySaveDataV2) -> Result<Self, Self::Error> {
        let mut items = HashMap::new();
        for (id, count) in v2.items {
            let bt = BlockTypeSave::from_string_id(&id)
                .ok_or_else(|| format!("Unknown item ID in global inventory: {}", id))?;
            items.insert(bt, count);
        }
        Ok(Self { items })
    }
}

impl TryFrom<WorldSaveDataV2> for WorldSaveData {
    type Error = String;

    fn try_from(v2: WorldSaveDataV2) -> Result<Self, Self::Error> {
        let mut modified_blocks = HashMap::new();
        for (pos, opt_id) in v2.modified_blocks {
            let block = match opt_id {
                Some(id) => Some(
                    BlockTypeSave::from_string_id(&id)
                        .ok_or_else(|| format!("Unknown block ID: {}", id))?,
                ),
                None => None,
            };
            modified_blocks.insert(pos, block);
        }
        Ok(Self { modified_blocks })
    }
}

impl TryFrom<ConveyorItemSaveV2> for ConveyorItemSave {
    type Error = String;

    fn try_from(v2: ConveyorItemSaveV2) -> Result<Self, Self::Error> {
        Ok(Self {
            item_type: BlockTypeSave::from_string_id(&v2.item_id)
                .ok_or_else(|| format!("Unknown item ID on conveyor: {}", v2.item_id))?,
            progress: v2.progress,
            lateral_offset: v2.lateral_offset,
        })
    }
}

impl TryFrom<MinerSaveDataV2> for MinerSaveData {
    type Error = String;

    fn try_from(v2: MinerSaveDataV2) -> Result<Self, Self::Error> {
        Ok(Self {
            position: v2.position,
            progress: v2.progress,
            buffer: v2.buffer.map(|s| s.try_into()).transpose()?,
        })
    }
}

impl TryFrom<ConveyorSaveDataV2> for ConveyorSaveData {
    type Error = String;

    fn try_from(v2: ConveyorSaveDataV2) -> Result<Self, Self::Error> {
        let items: Result<Vec<_>, _> = v2.items.into_iter().map(TryInto::try_into).collect();
        Ok(Self {
            position: v2.position,
            direction: v2.direction,
            shape: v2.shape,
            items: items?,
            last_output_index: v2.last_output_index,
            last_input_source: v2.last_input_source,
        })
    }
}

impl TryFrom<FurnaceSaveDataV2> for FurnaceSaveData {
    type Error = String;

    fn try_from(v2: FurnaceSaveDataV2) -> Result<Self, Self::Error> {
        Ok(Self {
            position: v2.position,
            fuel: v2.fuel,
            input: v2.input.map(|s| s.try_into()).transpose()?,
            output: v2.output.map(|s| s.try_into()).transpose()?,
            progress: v2.progress,
        })
    }
}

impl TryFrom<CrusherSaveDataV2> for CrusherSaveData {
    type Error = String;

    fn try_from(v2: CrusherSaveDataV2) -> Result<Self, Self::Error> {
        Ok(Self {
            position: v2.position,
            input: v2.input.map(|s| s.try_into()).transpose()?,
            output: v2.output.map(|s| s.try_into()).transpose()?,
            progress: v2.progress,
        })
    }
}

impl TryFrom<MachineSaveDataV2> for MachineSaveData {
    type Error = String;

    fn try_from(v2: MachineSaveDataV2) -> Result<Self, Self::Error> {
        Ok(match v2 {
            MachineSaveDataV2::Miner(m) => MachineSaveData::Miner(m.try_into()?),
            MachineSaveDataV2::Conveyor(c) => MachineSaveData::Conveyor(c.try_into()?),
            MachineSaveDataV2::Furnace(f) => MachineSaveData::Furnace(f.try_into()?),
            MachineSaveDataV2::Crusher(c) => MachineSaveData::Crusher(c.try_into()?),
        })
    }
}

impl TryFrom<QuestSaveDataV2> for QuestSaveData {
    type Error = String;

    fn try_from(v2: QuestSaveDataV2) -> Result<Self, Self::Error> {
        let mut delivered = HashMap::new();
        for (id, count) in v2.delivered {
            let bt = BlockTypeSave::from_string_id(&id)
                .ok_or_else(|| format!("Unknown item ID in quest delivery: {}", id))?;
            delivered.insert(bt, count);
        }
        Ok(Self {
            current_index: v2.current_index,
            completed: v2.completed,
            rewards_claimed: v2.rewards_claimed,
            delivered,
        })
    }
}

impl TryFrom<SaveDataV2> for SaveData {
    type Error = String;

    fn try_from(v2: SaveDataV2) -> Result<Self, Self::Error> {
        let machines: Result<Vec<_>, _> = v2.machines.into_iter().map(TryInto::try_into).collect();
        Ok(Self {
            version: v2.version,
            timestamp: v2.timestamp,
            player: v2.player,
            inventory: v2.inventory.try_into()?,
            global_inventory: v2.global_inventory.try_into()?,
            world: v2.world.try_into()?,
            machines: machines?,
            quests: v2.quests.try_into()?,
            mode: v2.mode,
        })
    }
}

/// World save data (modified blocks only)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldSaveData {
    /// Modified blocks: position -> Some(block) for placed, None for removed
    pub modified_blocks: HashMap<String, Option<BlockTypeSave>>,
}

impl WorldSaveData {
    /// Convert IVec3 to string key for JSON serialization
    pub fn pos_to_key(pos: IVec3) -> String {
        format!("{},{},{}", pos.x, pos.y, pos.z)
    }

    /// Parse string key back to IVec3
    pub fn key_to_pos(key: &str) -> Option<IVec3> {
        let parts: Vec<&str> = key.split(',').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(IVec3::new(
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ))
    }
}

/// Machine save data (all machine types)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum MachineSaveData {
    Miner(MinerSaveData),
    Conveyor(ConveyorSaveData),
    Furnace(FurnaceSaveData),
    Crusher(CrusherSaveData),
}

/// Direction for conveyors
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionSave {
    North,
    South,
    East,
    West,
}

/// Conveyor shape
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConveyorShapeSave {
    Straight,
    CornerLeft,
    CornerRight,
    TJunction,
    Splitter,
}

/// Miner save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinerSaveData {
    pub position: IVec3Save,
    pub progress: f32,
    pub buffer: Option<ItemStack>,
}

/// Conveyor save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConveyorSaveData {
    pub position: IVec3Save,
    pub direction: DirectionSave,
    pub shape: ConveyorShapeSave,
    pub items: Vec<ConveyorItemSave>,
    pub last_output_index: usize,
    pub last_input_source: usize,
}

/// Single item on conveyor
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConveyorItemSave {
    pub item_type: BlockTypeSave,
    pub progress: f32,
    pub lateral_offset: f32,
}

/// Furnace save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FurnaceSaveData {
    pub position: IVec3Save,
    pub fuel: u32,
    pub input: Option<ItemStack>,
    pub output: Option<ItemStack>,
    pub progress: f32,
}

/// Crusher save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CrusherSaveData {
    pub position: IVec3Save,
    pub input: Option<ItemStack>,
    pub output: Option<ItemStack>,
    pub progress: f32,
}

/// Quest progress save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuestSaveData {
    pub current_index: usize,
    pub completed: bool,
    pub rewards_claimed: bool,
    /// Items delivered to delivery platform
    pub delivered: HashMap<BlockTypeSave, u32>,
}

/// Game mode save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameModeSaveData {
    pub creative: bool,
}

/// Auto-save timer resource
#[derive(Resource)]
pub struct AutoSaveTimer {
    pub timer: Timer,
}

impl Default for AutoSaveTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(AUTO_SAVE_INTERVAL, TimerMode::Repeating),
        }
    }
}

/// Save slot info for listing saves
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SaveSlotInfo {
    pub filename: String,
    pub timestamp: u64,
}

/// Helper struct to detect save version
#[derive(Deserialize)]
struct VersionDetect {
    version: String,
}

pub mod native {
    use super::*;
    use std::fs;

    /// Get the saves directory path
    pub fn get_save_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(SAVE_DIR)
    }

    /// Ensure save directory exists
    pub fn ensure_save_dir() -> std::io::Result<()> {
        let dir = get_save_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        Ok(())
    }

    /// Save game data to a file (always uses V2 format)
    pub fn save_game(data: &SaveData, filename: &str) -> Result<(), String> {
        ensure_save_dir().map_err(|e| format!("Failed to create save directory: {}", e))?;

        // Convert to V2 format for saving
        let v2_data: SaveDataV2 = data.clone().into();

        let path = get_save_dir().join(format!("{}.json", filename));
        let json = serde_json::to_string_pretty(&v2_data)
            .map_err(|e| format!("Failed to serialize save data: {}", e))?;

        fs::write(&path, json).map_err(|e| format!("Failed to write save file: {}", e))?;

        Ok(())
    }

    /// Save game data in V2 format directly
    pub fn save_game_v2(data: &SaveDataV2, filename: &str) -> Result<(), String> {
        ensure_save_dir().map_err(|e| format!("Failed to create save directory: {}", e))?;

        let path = get_save_dir().join(format!("{}.json", filename));
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| format!("Failed to serialize save data: {}", e))?;

        fs::write(&path, json).map_err(|e| format!("Failed to write save file: {}", e))?;

        Ok(())
    }

    /// Load game data from a file (supports both V1 and V2 formats)
    pub fn load_game(filename: &str) -> Result<SaveData, String> {
        let path = get_save_dir().join(format!("{}.json", filename));

        if !path.exists() {
            return Err(format!("Save file not found: {}", filename));
        }

        let json =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read save file: {}", e))?;

        // Detect version
        let version_info: VersionDetect = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse save version: {}", e))?;

        if version_info.version.starts_with("0.2") {
            // V2 format - parse and convert to V1 for internal use
            let v2_data: SaveDataV2 = serde_json::from_str(&json)
                .map_err(|e| format!("Failed to parse V2 save data: {}", e))?;
            v2_data.try_into()
        } else {
            // V1 format - parse directly
            let data: SaveData = serde_json::from_str(&json)
                .map_err(|e| format!("Failed to parse save data: {}", e))?;
            Ok(data)
        }
    }

    /// Load game data in V2 format (supports both V1 and V2 formats)
    pub fn load_game_v2(filename: &str) -> Result<SaveDataV2, String> {
        let path = get_save_dir().join(format!("{}.json", filename));

        if !path.exists() {
            return Err(format!("Save file not found: {}", filename));
        }

        let json =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read save file: {}", e))?;

        // Detect version
        let version_info: VersionDetect = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse save version: {}", e))?;

        if version_info.version.starts_with("0.2") {
            // V2 format - parse directly
            serde_json::from_str(&json).map_err(|e| format!("Failed to parse V2 save data: {}", e))
        } else {
            // V1 format - parse and convert to V2
            let v1_data: SaveData = serde_json::from_str(&json)
                .map_err(|e| format!("Failed to parse save data: {}", e))?;
            Ok(v1_data.into())
        }
    }

    /// List all save files
    #[allow(dead_code)]
    pub fn list_saves() -> Result<Vec<SaveSlotInfo>, String> {
        let dir = get_save_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut saves = Vec::new();
        let entries =
            fs::read_dir(&dir).map_err(|e| format!("Failed to read save directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // Try to read timestamp from file
                    if let Ok(json) = fs::read_to_string(&path) {
                        if let Ok(data) = serde_json::from_str::<SaveData>(&json) {
                            saves.push(SaveSlotInfo {
                                filename: stem.to_string(),
                                timestamp: data.timestamp,
                            });
                        }
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(saves)
    }

    /// Delete a save file
    #[allow(dead_code)]
    pub fn delete_save(filename: &str) -> Result<(), String> {
        let path = get_save_dir().join(format!("{}.json", filename));

        if !path.exists() {
            return Err(format!("Save file not found: {}", filename));
        }

        fs::remove_file(&path).map_err(|e| format!("Failed to delete save file: {}", e))?;

        Ok(())
    }
}

/// Save game data to a file
pub fn save_game(data: &SaveData, filename: &str) -> Result<(), String> {
    native::save_game(data, filename)
}

/// Load game data from a file
pub fn load_game(filename: &str) -> Result<SaveData, String> {
    native::load_game(filename)
}

/// List all save files
#[allow(dead_code)]
pub fn list_saves() -> Result<Vec<SaveSlotInfo>, String> {
    native::list_saves()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_type_serialization() {
        let bt = BlockTypeSave::IronOre;
        let json = serde_json::to_string(&bt).expect("serialization should succeed");
        assert!(json.contains("IronOre"));

        let parsed: BlockTypeSave =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(parsed, bt);
    }

    #[test]
    fn test_block_type_conversion() {
        for bt in [
            BlockType::Stone,
            BlockType::Grass,
            BlockType::IronOre,
            BlockType::Coal,
            BlockType::IronIngot,
            BlockType::MinerBlock,
            BlockType::ConveyorBlock,
            BlockType::CopperOre,
            BlockType::CopperIngot,
            BlockType::CrusherBlock,
            BlockType::FurnaceBlock,
        ] {
            let save: BlockTypeSave = bt.into();
            let restored: BlockType = save.into();
            assert_eq!(bt, restored);
        }
    }

    #[test]
    fn test_pos_key_conversion() {
        let pos = IVec3::new(10, -5, 20);
        let key = WorldSaveData::pos_to_key(pos);
        assert_eq!(key, "10,-5,20");

        let parsed = WorldSaveData::key_to_pos(&key).expect("key_to_pos should succeed");
        assert_eq!(parsed, pos);
    }

    #[test]
    fn test_save_data_serialization() {
        let data = SaveData {
            version: SAVE_VERSION.to_string(),
            timestamp: 1704067200000,
            player: PlayerSaveData {
                position: Vec3Save {
                    x: 8.0,
                    y: 12.0,
                    z: 20.0,
                },
                rotation: CameraRotation {
                    pitch: 0.0,
                    yaw: 0.0,
                },
            },
            inventory: InventorySaveData {
                selected_slot: 0,
                slots: vec![
                    Some(ItemStack {
                        item_type: BlockTypeSave::Stone,
                        count: 10,
                    }),
                    None,
                ],
            },
            global_inventory: GlobalInventorySaveData::default(),
            world: WorldSaveData {
                modified_blocks: HashMap::new(),
            },
            machines: vec![],
            quests: QuestSaveData {
                current_index: 0,
                completed: false,
                rewards_claimed: false,
                delivered: HashMap::new(),
            },
            mode: GameModeSaveData { creative: false },
        };

        let json = serde_json::to_string_pretty(&data).expect("serialization should succeed");
        let parsed: SaveData = serde_json::from_str(&json).expect("deserialization should succeed");

        assert_eq!(parsed.version, data.version);
        assert_eq!(parsed.inventory.selected_slot, 0);
        assert!(parsed.inventory.slots[0].is_some());
    }

    // === Save/Load Round-trip Tests ===

    #[test]
    fn test_save_data_round_trip_with_machines() {
        // Create save data with all machine types
        let mut modified_blocks = HashMap::new();
        modified_blocks.insert("5,10,5".to_string(), Some(BlockTypeSave::Stone));
        modified_blocks.insert("6,10,5".to_string(), None); // Removed block

        let mut delivered = HashMap::new();
        delivered.insert(BlockTypeSave::IronIngot, 5);

        let data = SaveData {
            version: SAVE_VERSION.to_string(),
            timestamp: 1704067200000,
            player: PlayerSaveData {
                position: Vec3Save {
                    x: 100.0,
                    y: 50.0,
                    z: -200.0,
                },
                rotation: CameraRotation {
                    pitch: -0.5,
                    yaw: 3.14,
                },
            },
            inventory: InventorySaveData {
                selected_slot: 3,
                slots: vec![
                    Some(ItemStack {
                        item_type: BlockTypeSave::IronOre,
                        count: 64,
                    }),
                    Some(ItemStack {
                        item_type: BlockTypeSave::Coal,
                        count: 32,
                    }),
                    None,
                    Some(ItemStack {
                        item_type: BlockTypeSave::MinerBlock,
                        count: 5,
                    }),
                ],
            },
            global_inventory: GlobalInventorySaveData {
                items: {
                    let mut m = HashMap::new();
                    m.insert(BlockTypeSave::IronIngot, 100);
                    m.insert(BlockTypeSave::CopperOre, 50);
                    m
                },
            },
            world: WorldSaveData { modified_blocks },
            machines: vec![
                MachineSaveData::Miner(MinerSaveData {
                    position: IVec3Save { x: 10, y: 5, z: 10 },
                    progress: 0.5,
                    buffer: Some(ItemStack {
                        item_type: BlockTypeSave::IronOre,
                        count: 1,
                    }),
                }),
                MachineSaveData::Conveyor(ConveyorSaveData {
                    position: IVec3Save { x: 11, y: 5, z: 10 },
                    direction: DirectionSave::East,
                    shape: ConveyorShapeSave::Straight,
                    items: vec![ConveyorItemSave {
                        item_type: BlockTypeSave::IronOre,
                        progress: 0.3,
                        lateral_offset: 0.0,
                    }],
                    last_output_index: 0,
                    last_input_source: 0,
                }),
                MachineSaveData::Furnace(FurnaceSaveData {
                    position: IVec3Save { x: 12, y: 5, z: 10 },
                    fuel: 10,
                    input: Some(ItemStack {
                        item_type: BlockTypeSave::IronOre,
                        count: 5,
                    }),
                    output: Some(ItemStack {
                        item_type: BlockTypeSave::IronIngot,
                        count: 3,
                    }),
                    progress: 0.75,
                }),
                MachineSaveData::Crusher(CrusherSaveData {
                    position: IVec3Save { x: 13, y: 5, z: 10 },
                    input: Some(ItemStack {
                        item_type: BlockTypeSave::CopperOre,
                        count: 10,
                    }),
                    output: Some(ItemStack {
                        item_type: BlockTypeSave::CopperOre,
                        count: 6,
                    }),
                    progress: 0.25,
                }),
            ],
            quests: QuestSaveData {
                current_index: 2,
                completed: false,
                rewards_claimed: false,
                delivered,
            },
            mode: GameModeSaveData { creative: true },
        };

        // Serialize and deserialize
        let json = serde_json::to_string(&data).expect("serialization should succeed");
        let restored: SaveData =
            serde_json::from_str(&json).expect("deserialization should succeed");

        // Verify all fields
        assert_eq!(restored.version, data.version);
        assert_eq!(restored.timestamp, data.timestamp);

        // Player
        assert!((restored.player.position.x - 100.0).abs() < 0.001);
        assert!((restored.player.rotation.yaw - 3.14).abs() < 0.001);

        // Inventory
        assert_eq!(restored.inventory.selected_slot, 3);
        assert_eq!(restored.inventory.slots.len(), 4);
        assert_eq!(
            restored.inventory.slots[0]
                .as_ref()
                .expect("slot 0 should exist")
                .count,
            64
        );

        // Global inventory
        assert_eq!(
            restored
                .global_inventory
                .items
                .get(&BlockTypeSave::IronIngot),
            Some(&100)
        );

        // World
        assert_eq!(restored.world.modified_blocks.len(), 2);

        // Machines
        assert_eq!(restored.machines.len(), 4);
        match &restored.machines[0] {
            MachineSaveData::Miner(m) => assert!((m.progress - 0.5).abs() < 0.001),
            _ => panic!("Expected Miner"),
        }
        match &restored.machines[1] {
            MachineSaveData::Conveyor(c) => {
                assert_eq!(c.direction, DirectionSave::East);
                assert_eq!(c.items.len(), 1);
            }
            _ => panic!("Expected Conveyor"),
        }
        match &restored.machines[2] {
            MachineSaveData::Furnace(f) => assert_eq!(f.fuel, 10),
            _ => panic!("Expected Furnace"),
        }
        match &restored.machines[3] {
            MachineSaveData::Crusher(c) => assert_eq!(
                c.input.as_ref().expect("crusher input should exist").count,
                10
            ),
            _ => panic!("Expected Crusher"),
        }

        // Quests
        assert_eq!(restored.quests.current_index, 2);
        assert_eq!(
            restored.quests.delivered.get(&BlockTypeSave::IronIngot),
            Some(&5)
        );

        // Mode
        assert!(restored.mode.creative);
    }

    // === Edge Case Tests ===

    #[test]
    fn test_key_to_pos_invalid_formats() {
        // Too few parts
        assert!(WorldSaveData::key_to_pos("10,20").is_none());

        // Too many parts
        assert!(WorldSaveData::key_to_pos("10,20,30,40").is_none());

        // Non-numeric
        assert!(WorldSaveData::key_to_pos("abc,20,30").is_none());
        assert!(WorldSaveData::key_to_pos("10,xyz,30").is_none());
        assert!(WorldSaveData::key_to_pos("10,20,!!!").is_none());

        // Empty
        assert!(WorldSaveData::key_to_pos("").is_none());

        // Partial empty
        assert!(WorldSaveData::key_to_pos(",20,30").is_none());
        assert!(WorldSaveData::key_to_pos("10,,30").is_none());
    }

    #[test]
    fn test_key_to_pos_boundary_values() {
        // Large positive values
        let big_pos = IVec3::new(i32::MAX, i32::MAX, i32::MAX);
        let key = WorldSaveData::pos_to_key(big_pos);
        let restored =
            WorldSaveData::key_to_pos(&key).expect("key_to_pos should succeed for big values");
        assert_eq!(restored, big_pos);

        // Large negative values
        let small_pos = IVec3::new(i32::MIN, i32::MIN, i32::MIN);
        let key = WorldSaveData::pos_to_key(small_pos);
        let restored =
            WorldSaveData::key_to_pos(&key).expect("key_to_pos should succeed for small values");
        assert_eq!(restored, small_pos);

        // Zero
        let zero_pos = IVec3::ZERO;
        let key = WorldSaveData::pos_to_key(zero_pos);
        let restored = WorldSaveData::key_to_pos(&key).expect("key_to_pos should succeed for zero");
        assert_eq!(restored, zero_pos);
    }

    #[test]
    fn test_empty_save_data() {
        let data = SaveData {
            version: SAVE_VERSION.to_string(),
            timestamp: 0,
            player: PlayerSaveData {
                position: Vec3Save {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                rotation: CameraRotation {
                    pitch: 0.0,
                    yaw: 0.0,
                },
            },
            inventory: InventorySaveData {
                selected_slot: 0,
                slots: vec![],
            },
            global_inventory: GlobalInventorySaveData::default(),
            world: WorldSaveData {
                modified_blocks: HashMap::new(),
            },
            machines: vec![],
            quests: QuestSaveData {
                current_index: 0,
                completed: false,
                rewards_claimed: false,
                delivered: HashMap::new(),
            },
            mode: GameModeSaveData { creative: false },
        };

        let json = serde_json::to_string(&data).expect("serialization should succeed");
        let restored: SaveData =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert!(restored.inventory.slots.is_empty());
        assert!(restored.machines.is_empty());
        assert!(restored.world.modified_blocks.is_empty());
    }

    #[test]
    fn test_max_stack_values() {
        let stack = ItemStack {
            item_type: BlockTypeSave::Stone,
            count: u32::MAX,
        };

        let json = serde_json::to_string(&stack).expect("serialization should succeed");
        let restored: ItemStack =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert_eq!(restored.count, u32::MAX);
    }

    #[test]
    fn test_conveyor_all_shapes() {
        for shape in [
            ConveyorShapeSave::Straight,
            ConveyorShapeSave::CornerLeft,
            ConveyorShapeSave::CornerRight,
            ConveyorShapeSave::TJunction,
            ConveyorShapeSave::Splitter,
        ] {
            let conveyor = ConveyorSaveData {
                position: IVec3Save { x: 0, y: 0, z: 0 },
                direction: DirectionSave::North,
                shape,
                items: vec![],
                last_output_index: 0,
                last_input_source: 0,
            };

            let json = serde_json::to_string(&conveyor).expect("serialization should succeed");
            let restored: ConveyorSaveData =
                serde_json::from_str(&json).expect("deserialization should succeed");

            assert_eq!(restored.shape, shape);
        }
    }

    #[test]
    fn test_direction_all_values() {
        for dir in [
            DirectionSave::North,
            DirectionSave::South,
            DirectionSave::East,
            DirectionSave::West,
        ] {
            let conveyor = ConveyorSaveData {
                position: IVec3Save { x: 0, y: 0, z: 0 },
                direction: dir,
                shape: ConveyorShapeSave::Straight,
                items: vec![],
                last_output_index: 0,
                last_input_source: 0,
            };

            let json = serde_json::to_string(&conveyor).expect("serialization should succeed");
            let restored: ConveyorSaveData =
                serde_json::from_str(&json).expect("deserialization should succeed");

            assert_eq!(restored.direction, dir);
        }
    }

    // === String ID Format (V2) Tests ===

    #[test]
    fn test_block_type_save_to_string_id() {
        // Test all BlockTypeSave variants
        assert_eq!(BlockTypeSave::Stone.to_string_id(), "base:stone");
        assert_eq!(BlockTypeSave::Grass.to_string_id(), "base:grass");
        assert_eq!(BlockTypeSave::IronOre.to_string_id(), "base:iron_ore");
        assert_eq!(BlockTypeSave::Coal.to_string_id(), "base:coal");
        assert_eq!(BlockTypeSave::IronIngot.to_string_id(), "base:iron_ingot");
        assert_eq!(BlockTypeSave::MinerBlock.to_string_id(), "base:miner_block");
        assert_eq!(
            BlockTypeSave::ConveyorBlock.to_string_id(),
            "base:conveyor_block"
        );
        assert_eq!(BlockTypeSave::CopperOre.to_string_id(), "base:copper_ore");
        assert_eq!(
            BlockTypeSave::CopperIngot.to_string_id(),
            "base:copper_ingot"
        );
        assert_eq!(
            BlockTypeSave::CrusherBlock.to_string_id(),
            "base:crusher_block"
        );
        assert_eq!(
            BlockTypeSave::FurnaceBlock.to_string_id(),
            "base:furnace_block"
        );
        assert_eq!(
            BlockTypeSave::StonePickaxe.to_string_id(),
            "base:stone_pickaxe"
        );
        assert_eq!(
            BlockTypeSave::AssemblerBlock.to_string_id(),
            "base:assembler_block"
        );
        assert_eq!(BlockTypeSave::IronDust.to_string_id(), "base:iron_dust");
        assert_eq!(BlockTypeSave::CopperDust.to_string_id(), "base:copper_dust");
        assert_eq!(
            BlockTypeSave::PlatformBlock.to_string_id(),
            "base:platform_block"
        );
    }

    #[test]
    fn test_block_type_save_from_string_id() {
        // Test basic parsing
        assert_eq!(
            BlockTypeSave::from_string_id("base:stone"),
            Some(BlockTypeSave::Stone)
        );
        assert_eq!(
            BlockTypeSave::from_string_id("base:iron_ore"),
            Some(BlockTypeSave::IronOre)
        );
        assert_eq!(
            BlockTypeSave::from_string_id("base:miner_block"),
            Some(BlockTypeSave::MinerBlock)
        );

        // Test aliases (short names)
        assert_eq!(
            BlockTypeSave::from_string_id("base:miner"),
            Some(BlockTypeSave::MinerBlock)
        );
        assert_eq!(
            BlockTypeSave::from_string_id("base:conveyor"),
            Some(BlockTypeSave::ConveyorBlock)
        );
        assert_eq!(
            BlockTypeSave::from_string_id("base:furnace"),
            Some(BlockTypeSave::FurnaceBlock)
        );
        assert_eq!(
            BlockTypeSave::from_string_id("base:crusher"),
            Some(BlockTypeSave::CrusherBlock)
        );
        assert_eq!(
            BlockTypeSave::from_string_id("base:pickaxe"),
            Some(BlockTypeSave::StonePickaxe)
        );
        assert_eq!(
            BlockTypeSave::from_string_id("base:platform"),
            Some(BlockTypeSave::PlatformBlock)
        );

        // Test fallback (no namespace)
        assert_eq!(
            BlockTypeSave::from_string_id("stone"),
            Some(BlockTypeSave::Stone)
        );
        assert_eq!(
            BlockTypeSave::from_string_id("iron_ore"),
            Some(BlockTypeSave::IronOre)
        );

        // Test invalid cases
        assert_eq!(BlockTypeSave::from_string_id("unknown:stone"), None);
        assert_eq!(BlockTypeSave::from_string_id("base:unknown_item"), None);
        assert_eq!(BlockTypeSave::from_string_id("mod:custom_item"), None);
        assert_eq!(BlockTypeSave::from_string_id(""), None);
    }

    #[test]
    fn test_block_type_save_string_id_roundtrip() {
        // Test that all BlockTypeSave variants can be converted to string and back
        let all_types = [
            BlockTypeSave::Stone,
            BlockTypeSave::Grass,
            BlockTypeSave::IronOre,
            BlockTypeSave::Coal,
            BlockTypeSave::IronIngot,
            BlockTypeSave::MinerBlock,
            BlockTypeSave::ConveyorBlock,
            BlockTypeSave::CopperOre,
            BlockTypeSave::CopperIngot,
            BlockTypeSave::CrusherBlock,
            BlockTypeSave::FurnaceBlock,
            BlockTypeSave::StonePickaxe,
            BlockTypeSave::AssemblerBlock,
            BlockTypeSave::IronDust,
            BlockTypeSave::CopperDust,
            BlockTypeSave::PlatformBlock,
        ];

        for bt in all_types {
            let string_id = bt.to_string_id();
            let restored = BlockTypeSave::from_string_id(&string_id)
                .unwrap_or_else(|| panic!("Failed to parse string ID: {}", string_id));
            assert_eq!(bt, restored, "Roundtrip failed for {:?}", bt);
        }
    }

    #[test]
    fn test_item_stack_v2_new() {
        let stack = ItemStackV2::new("base:iron_ore", 64);
        assert_eq!(stack.item_id, "base:iron_ore");
        assert_eq!(stack.count, 64);
    }

    #[test]
    fn test_item_stack_v2_serialization() {
        let stack = ItemStackV2::new("base:iron_ore", 64);

        let json = serde_json::to_string(&stack).expect("serialization should succeed");
        assert!(json.contains("base:iron_ore"));
        assert!(json.contains("64"));

        let restored: ItemStackV2 =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(restored, stack);
    }

    #[test]
    fn test_item_stack_v2_from_legacy() {
        let legacy = ItemStack {
            item_type: BlockTypeSave::IronOre,
            count: 32,
        };

        let v2: ItemStackV2 = legacy.clone().into();
        assert_eq!(v2.item_id, "base:iron_ore");
        assert_eq!(v2.count, 32);

        // Also test reference conversion
        let v2_ref: ItemStackV2 = (&legacy).into();
        assert_eq!(v2_ref.item_id, "base:iron_ore");
        assert_eq!(v2_ref.count, 32);
    }

    #[test]
    fn test_item_stack_v2_to_legacy() {
        // Test successful conversion
        let v2 = ItemStackV2::new("base:iron_ore", 64);
        let legacy = v2.to_legacy().expect("conversion should succeed");
        assert_eq!(legacy.item_type, BlockTypeSave::IronOre);
        assert_eq!(legacy.count, 64);

        // Test conversion of unknown item (mod item)
        let mod_item = ItemStackV2::new("mymod:copper_plate", 10);
        assert!(mod_item.to_legacy().is_none());
    }

    #[test]
    fn test_item_stack_v2_try_from() {
        // Test successful TryFrom
        let v2 = ItemStackV2::new("base:stone", 100);
        let legacy: ItemStack = v2.try_into().expect("TryFrom should succeed");
        assert_eq!(legacy.item_type, BlockTypeSave::Stone);
        assert_eq!(legacy.count, 100);

        // Test failed TryFrom
        let mod_item = ItemStackV2::new("unknown:item", 1);
        let result: Result<ItemStack, _> = mod_item.try_into();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown item ID"));
    }

    #[test]
    fn test_item_stack_v2_roundtrip() {
        // Legacy -> V2 -> Legacy roundtrip
        let original = ItemStack {
            item_type: BlockTypeSave::CopperIngot,
            count: 999,
        };

        let v2: ItemStackV2 = original.clone().into();
        let restored: ItemStack = v2.try_into().expect("roundtrip should succeed");

        assert_eq!(original.item_type, restored.item_type);
        assert_eq!(original.count, restored.count);
    }

    #[test]
    fn test_block_type_save_string_id_helpers() {
        // Test BlockType::to_save_string_id
        assert_eq!(BlockType::Stone.to_save_string_id(), "base:stone");
        assert_eq!(BlockType::IronOre.to_save_string_id(), "base:iron_ore");
        assert_eq!(
            BlockType::MinerBlock.to_save_string_id(),
            "base:miner_block"
        );

        // Test BlockType::from_save_string_id
        assert_eq!(
            BlockType::from_save_string_id("base:stone"),
            Some(BlockType::Stone)
        );
        assert_eq!(
            BlockType::from_save_string_id("base:iron_ore"),
            Some(BlockType::IronOre)
        );
        assert_eq!(
            BlockType::from_save_string_id("base:miner"),
            Some(BlockType::MinerBlock)
        );
        assert_eq!(BlockType::from_save_string_id("unknown:item"), None);
    }

    #[test]
    fn test_item_stack_v2_json_format() {
        // Verify the JSON format is what we expect
        let stack = ItemStackV2::new("base:iron_ore", 64);
        let json = serde_json::to_string_pretty(&stack).expect("serialization should succeed");

        // The JSON should be human-readable with string IDs
        assert!(json.contains(r#""item_id": "base:iron_ore""#));
        assert!(json.contains(r#""count": 64"#));
    }

    // === V2 Save Data Tests ===

    #[test]
    fn test_save_data_v2_conversion() {
        // Create V1 save data
        let v1 = SaveData {
            version: SAVE_VERSION.to_string(),
            timestamp: 1704067200000,
            player: PlayerSaveData {
                position: Vec3Save {
                    x: 10.0,
                    y: 20.0,
                    z: 30.0,
                },
                rotation: CameraRotation {
                    pitch: 0.5,
                    yaw: 1.0,
                },
            },
            inventory: InventorySaveData {
                selected_slot: 1,
                slots: vec![
                    Some(ItemStack {
                        item_type: BlockTypeSave::IronOre,
                        count: 64,
                    }),
                    None,
                ],
            },
            global_inventory: GlobalInventorySaveData {
                items: {
                    let mut m = HashMap::new();
                    m.insert(BlockTypeSave::Coal, 100);
                    m
                },
            },
            world: WorldSaveData {
                modified_blocks: {
                    let mut m = HashMap::new();
                    m.insert("5,10,5".to_string(), Some(BlockTypeSave::Stone));
                    m
                },
            },
            machines: vec![MachineSaveData::Miner(MinerSaveData {
                position: IVec3Save { x: 10, y: 5, z: 10 },
                progress: 0.5,
                buffer: Some(ItemStack {
                    item_type: BlockTypeSave::IronOre,
                    count: 1,
                }),
            })],
            quests: QuestSaveData {
                current_index: 1,
                completed: false,
                rewards_claimed: false,
                delivered: {
                    let mut m = HashMap::new();
                    m.insert(BlockTypeSave::IronIngot, 5);
                    m
                },
            },
            mode: GameModeSaveData { creative: false },
        };

        // Convert to V2
        let v2: SaveDataV2 = v1.clone().into();

        // Check version
        assert_eq!(v2.version, SAVE_VERSION_V2);

        // Check inventory uses string IDs
        assert_eq!(
            v2.inventory.slots[0].as_ref().unwrap().item_id,
            "base:iron_ore"
        );

        // Check global inventory uses string IDs
        assert!(v2.global_inventory.items.contains_key("base:coal"));

        // Check world uses string IDs
        assert_eq!(
            v2.world.modified_blocks.get("5,10,5"),
            Some(&Some("base:stone".to_string()))
        );

        // Check quests use string IDs
        assert!(v2.quests.delivered.contains_key("base:iron_ingot"));
    }

    #[test]
    fn test_save_data_v2_roundtrip() {
        // Create V1 data
        let mut delivered = HashMap::new();
        delivered.insert(BlockTypeSave::CopperOre, 10);

        let original = SaveData {
            version: SAVE_VERSION.to_string(),
            timestamp: 1234567890,
            player: PlayerSaveData {
                position: Vec3Save {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                rotation: CameraRotation {
                    pitch: 0.0,
                    yaw: 0.0,
                },
            },
            inventory: InventorySaveData {
                selected_slot: 0,
                slots: vec![Some(ItemStack {
                    item_type: BlockTypeSave::Coal,
                    count: 32,
                })],
            },
            global_inventory: GlobalInventorySaveData::default(),
            world: WorldSaveData {
                modified_blocks: HashMap::new(),
            },
            machines: vec![],
            quests: QuestSaveData {
                current_index: 0,
                completed: false,
                rewards_claimed: false,
                delivered,
            },
            mode: GameModeSaveData { creative: true },
        };

        // V1 -> V2 -> V1 roundtrip
        let v2: SaveDataV2 = original.clone().into();
        let restored: SaveData = v2.try_into().expect("roundtrip should succeed");

        // Check key fields
        assert_eq!(restored.timestamp, original.timestamp);
        assert_eq!(
            restored.inventory.slots[0].as_ref().unwrap().item_type,
            BlockTypeSave::Coal
        );
        assert_eq!(
            restored.quests.delivered.get(&BlockTypeSave::CopperOre),
            Some(&10)
        );
        assert!(restored.mode.creative);
    }

    #[test]
    fn test_save_data_v2_serialization() {
        let v2 = SaveDataV2 {
            version: SAVE_VERSION_V2.to_string(),
            timestamp: 1704067200000,
            player: PlayerSaveData {
                position: Vec3Save {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                rotation: CameraRotation {
                    pitch: 0.0,
                    yaw: 0.0,
                },
            },
            inventory: InventorySaveDataV2 {
                selected_slot: 0,
                slots: vec![Some(ItemStackV2::new("base:iron_ore", 64))],
            },
            global_inventory: GlobalInventorySaveDataV2::default(),
            world: WorldSaveDataV2 {
                modified_blocks: HashMap::new(),
            },
            machines: vec![],
            quests: QuestSaveDataV2 {
                current_index: 0,
                completed: false,
                rewards_claimed: false,
                delivered: HashMap::new(),
            },
            mode: GameModeSaveData { creative: false },
        };

        // Serialize and deserialize
        let json = serde_json::to_string_pretty(&v2).expect("serialization should succeed");

        // JSON should contain string IDs
        assert!(json.contains("base:iron_ore"));
        assert!(json.contains("0.2.0"));

        // Deserialize back
        let restored: SaveDataV2 =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(restored.version, SAVE_VERSION_V2);
        assert_eq!(
            restored.inventory.slots[0].as_ref().unwrap().item_id,
            "base:iron_ore"
        );
    }

    #[test]
    fn test_machine_save_data_v2_all_types() {
        // Test all machine types convert correctly
        let machines = vec![
            MachineSaveData::Miner(MinerSaveData {
                position: IVec3Save { x: 0, y: 0, z: 0 },
                progress: 0.5,
                buffer: Some(ItemStack {
                    item_type: BlockTypeSave::IronOre,
                    count: 1,
                }),
            }),
            MachineSaveData::Conveyor(ConveyorSaveData {
                position: IVec3Save { x: 1, y: 0, z: 0 },
                direction: DirectionSave::East,
                shape: ConveyorShapeSave::Straight,
                items: vec![ConveyorItemSave {
                    item_type: BlockTypeSave::Coal,
                    progress: 0.3,
                    lateral_offset: 0.0,
                }],
                last_output_index: 0,
                last_input_source: 0,
            }),
            MachineSaveData::Furnace(FurnaceSaveData {
                position: IVec3Save { x: 2, y: 0, z: 0 },
                fuel: 10,
                input: Some(ItemStack {
                    item_type: BlockTypeSave::IronOre,
                    count: 5,
                }),
                output: Some(ItemStack {
                    item_type: BlockTypeSave::IronIngot,
                    count: 3,
                }),
                progress: 0.75,
            }),
            MachineSaveData::Crusher(CrusherSaveData {
                position: IVec3Save { x: 3, y: 0, z: 0 },
                input: Some(ItemStack {
                    item_type: BlockTypeSave::CopperOre,
                    count: 10,
                }),
                output: None,
                progress: 0.25,
            }),
        ];

        for machine in machines {
            let v2: MachineSaveDataV2 = machine.clone().into();
            let restored: MachineSaveData = v2.try_into().expect("roundtrip should succeed");

            // Verify the type matches
            match (&machine, &restored) {
                (MachineSaveData::Miner(_), MachineSaveData::Miner(_)) => {}
                (MachineSaveData::Conveyor(_), MachineSaveData::Conveyor(_)) => {}
                (MachineSaveData::Furnace(_), MachineSaveData::Furnace(_)) => {}
                (MachineSaveData::Crusher(_), MachineSaveData::Crusher(_)) => {}
                _ => panic!("Machine type mismatch after roundtrip"),
            }
        }
    }
}
