//! All game components, resources, and enums
//!
//! This module defines all ECS components and resources used in the game.
//! It is the lowest layer and has no dependencies on other game modules.

mod input;
mod machines;
mod player;
mod ui;

pub use input::*;
pub use machines::*;
pub use player::*;
pub use ui::*;

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

/// Breaking progress state (for time-based block breaking)
#[derive(Resource, Default)]
pub struct BreakingProgress {
    /// Position of block being broken (world block or machine position)
    pub target_pos: Option<IVec3>,
    /// Entity being broken (for machines)
    pub target_entity: Option<Entity>,
    /// Current progress (0.0 to 1.0)
    pub progress: f32,
    /// Total time required to break (seconds)
    pub total_time: f32,
    /// Whether breaking a machine (true) or world block (false)
    pub is_machine: bool,
}

impl BreakingProgress {
    /// Reset breaking progress
    pub fn reset(&mut self) {
        self.target_pos = None;
        self.target_entity = None;
        self.progress = 0.0;
        self.total_time = 0.0;
        self.is_machine = false;
    }

    /// Check if currently breaking something
    pub fn is_breaking(&self) -> bool {
        self.target_pos.is_some() || self.target_entity.is_some()
    }

    /// Check if breaking is complete
    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }
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

/// Active sub-quest state
#[derive(Clone, Default)]
pub struct SubQuestState {
    /// Index of sub-quest in SUB_QUESTS array
    pub quest_index: usize,
    /// Whether the sub-quest is completed
    pub completed: bool,
    /// Whether rewards were claimed
    pub rewards_claimed: bool,
}

/// Active sub-quests resource (up to MAX_ACTIVE_SUB_QUESTS)
#[derive(Resource, Default)]
pub struct ActiveSubQuests {
    /// Currently active sub-quests
    pub quests: Vec<SubQuestState>,
}

impl ActiveSubQuests {
    /// Check if a sub-quest is already active
    pub fn is_active(&self, quest_index: usize) -> bool {
        self.quests.iter().any(|q| q.quest_index == quest_index)
    }

    /// Add a sub-quest if not already active and under limit
    pub fn add_quest(&mut self, quest_index: usize, max_active: usize) -> bool {
        if self.quests.len() >= max_active || self.is_active(quest_index) {
            return false;
        }
        self.quests.push(SubQuestState {
            quest_index,
            completed: false,
            rewards_claimed: false,
        });
        true
    }

    /// Remove a completed and claimed sub-quest
    pub fn remove_claimed(&mut self) {
        self.quests.retain(|q| !q.rewards_claimed);
    }
}

/// Marker for quest UI
#[derive(Component)]
pub struct QuestUI;

/// Marker for quest UI text (title/description)
#[derive(Component)]
pub struct QuestUIText;

/// Marker for quest progress container (holds progress bars)
#[derive(Component)]
pub struct QuestProgressContainer;

/// Individual progress item row (item icon + progress bar + text)
#[derive(Component)]
pub struct QuestProgressItem(pub usize);

/// Progress bar background
#[derive(Component)]
pub struct QuestProgressBarBg(pub usize);

/// Progress bar fill
#[derive(Component)]
pub struct QuestProgressBarFill(pub usize);

/// Progress bar text (delivered/required)
#[derive(Component)]
pub struct QuestProgressText(pub usize);

/// Marker for quest deliver button
#[derive(Component)]
pub struct QuestDeliverButton;

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
