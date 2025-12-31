//! All game components, resources, and enums
//!
//! This module defines all ECS components and resources used in the game.
//! It is the lowest layer and has no dependencies on other game modules.

mod player;
mod machines;
mod ui;
mod input;

pub use player::*;
pub use machines::*;
pub use ui::*;
pub use input::*;

use crate::BlockType;
use bevy::prelude::*;
use std::collections::HashMap;

// === Core Resources ===

/// Font resource for UI text
#[derive(Resource)]
pub struct GameFont(#[allow(dead_code)] pub Handle<Font>);

impl FromWorld for GameFont {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        GameFont(asset_server.load("fonts/NotoSansJP-Regular.ttf"))
    }
}

/// Debug HUD visibility state
#[derive(Resource, Default)]
pub struct DebugHudState {
    pub visible: bool,
}

/// Marker for debug HUD text
#[derive(Component)]
pub struct DebugHudText;

/// Target block for highlighting (what the player is looking at)
#[derive(Resource, Default)]
pub struct TargetBlock {
    /// Position of block that would be broken (left click)
    pub break_target: Option<IVec3>,
    /// Position where block would be placed (right click)
    pub place_target: Option<IVec3>,
    /// Entity for break highlight visualization
    pub break_highlight_entity: Option<Entity>,
    /// Entity for place highlight visualization
    pub place_highlight_entity: Option<Entity>,
}

/// Marker component for break target highlight (edges)
#[derive(Component)]
pub struct TargetHighlight;

/// Marker component for place target highlight (edges)
#[derive(Component)]
pub struct PlaceHighlight;

/// Marker component for guide markers (recommended placement positions)
#[derive(Component)]
pub struct GuideMarker;

/// Resource to track guide marker entities
#[derive(Resource, Default)]
pub struct GuideMarkers {
    pub entities: Vec<Entity>,
    pub last_selected: Option<BlockType>,
}

/// Conveyor rotation offset (R key cycles through 0-3)
#[derive(Resource, Default)]
pub struct ConveyorRotationOffset {
    /// Number of 90-degree clockwise rotations (0-3)
    pub offset: u8,
}

/// Creative mode resource for spawning items
#[derive(Resource, Default)]
pub struct CreativeMode {
    pub enabled: bool,
}

/// Tutorial shown state (prevents showing again)
#[derive(Resource, Default)]
pub struct TutorialShown(pub bool);

/// Marker for tutorial popup UI
#[derive(Component)]
pub struct TutorialPopup;

/// Quest definition
/// Note: systems/quest.rs has its own QuestDef struct
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct QuestDef {
    /// Quest description
    pub description: &'static str,
    /// Required item type
    pub required_item: BlockType,
    /// Required amount
    pub required_amount: u32,
    /// Rewards: (BlockType, amount)
    pub rewards: Vec<(BlockType, u32)>,
}

/// Current quest state
#[derive(Resource, Default)]
pub struct CurrentQuest {
    /// Index of current quest (0-based)
    pub index: usize,
    /// Whether the quest is completed
    pub completed: bool,
    /// Whether rewards were claimed
    pub rewards_claimed: bool,
}

/// Marker for quest UI
#[derive(Component)]
pub struct QuestUI;

/// Marker for quest UI text
#[derive(Component)]
pub struct QuestUIText;

/// Delivery platform - accepts items for delivery quests
#[derive(Component, Default)]
pub struct DeliveryPlatform {
    /// Total items delivered (by type)
    pub delivered: HashMap<BlockType, u32>,
}

/// Marker for delivery platform UI
#[allow(dead_code)]
#[derive(Component)]
pub struct DeliveryUI;

/// Marker for delivery UI text
#[derive(Component)]
pub struct DeliveryUIText;

/// All available items for creative mode, organized by category
pub const CREATIVE_ITEMS: &[(BlockType, &str)] = &[
    // Blocks
    (BlockType::Stone, "Blocks"),
    (BlockType::Grass, "Blocks"),
    // Ores
    (BlockType::IronOre, "Ores"),
    (BlockType::CopperOre, "Ores"),
    (BlockType::Coal, "Ores"),
    // Ingots
    (BlockType::IronIngot, "Ingots"),
    (BlockType::CopperIngot, "Ingots"),
    // Machines
    (BlockType::MinerBlock, "Machines"),
    (BlockType::ConveyorBlock, "Machines"),
    (BlockType::CrusherBlock, "Machines"),
    (BlockType::FurnaceBlock, "Machines"),
];

/// State for save/load operations
#[derive(Resource, Default)]
pub struct SaveLoadState {
    /// Pending load data (applied on next frame to avoid borrow conflicts)
    #[allow(dead_code)]
    pub pending_load: Option<crate::save::SaveData>,
    /// Last save/load message for display
    #[allow(dead_code)]
    pub last_message: Option<String>,
}

/// Resource to hold item sprite textures for UI
#[derive(Resource, Default)]
pub struct ItemSprites {
    pub textures: HashMap<BlockType, Handle<Image>>,
}

impl ItemSprites {
    /// Get sprite handle for a block type, returns None if not loaded
    pub fn get(&self, block_type: BlockType) -> Option<Handle<Image>> {
        self.textures.get(&block_type).cloned()
    }
}
