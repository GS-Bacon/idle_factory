//! All game components, resources, and enums
//!
//! This module defines all ECS components and resources used in the game.
//! It is the lowest layer and has no dependencies on other game modules.

mod input;
mod machines;
mod network;
mod player;
mod ui;
mod ui_state;

pub use input::*;
pub use machines::*;
pub use network::*;
pub use player::*;
pub use ui::*;
pub use ui_state::*;

use crate::core::ItemId;
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

/// Marker for biome HUD text (always visible, shows current biome)
#[derive(Component)]
pub struct BiomeHudText;

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
    /// Last selected item
    pub last_selected: Option<ItemId>,
}

impl GuideMarkers {
    /// Get last selected as ItemId
    pub fn last_selected_id(&self) -> Option<ItemId> {
        self.last_selected
    }

    /// Set last selected from ItemId
    pub fn set_last_selected_id(&mut self, item: Option<ItemId>) {
        self.last_selected = item;
    }
}

/// Conveyor rotation offset (R key cycles through 0-3)
#[derive(Resource, Default)]
pub struct ConveyorRotationOffset {
    /// Number of 90-degree clockwise rotations (0-3)
    pub offset: u8,
}

/// Creative mode resource (CAD-style: always enabled for fly mode)
#[derive(Resource)]
pub struct CreativeMode {
    pub enabled: bool,
}

impl Default for CreativeMode {
    fn default() -> Self {
        // CAD-style: always fly mode
        Self { enabled: true }
    }
}

/// Tutorial shown state (prevents showing again)
#[derive(Resource)]
pub struct TutorialShown(pub bool);

impl Default for TutorialShown {
    fn default() -> Self {
        // Start with tutorial already "shown" (no welcome popup)
        Self(true)
    }
}

// =============================================================================
// Tutorial Quest System
// =============================================================================

/// Tutorial actions that can trigger quest completion
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TutorialAction {
    /// Move a certain distance
    Move { distance: u32 },
    /// Break any block
    BreakBlock,
    /// Open inventory
    OpenInventory,
    /// Place a specific machine type
    PlaceMachine(ItemId),
    /// Place consecutive conveyors
    PlaceConveyors { count: u32 },
    /// Create a valid machine connection line
    CreateConnection,
    /// Produce a specific item
    ProduceItem(ItemId),
}

impl TutorialAction {
    /// Get the machine type as ItemId if this is a PlaceMachine action
    pub fn place_machine_id(&self) -> Option<ItemId> {
        match self {
            TutorialAction::PlaceMachine(id) => Some(*id),
            _ => None,
        }
    }

    /// Get the item type as ItemId if this is a ProduceItem action
    pub fn produce_item_id(&self) -> Option<ItemId> {
        match self {
            TutorialAction::ProduceItem(id) => Some(*id),
            _ => None,
        }
    }

    /// Check if this action matches a placed machine (by ItemId)
    pub fn matches_place_machine_id(&self, item: ItemId) -> bool {
        match self {
            TutorialAction::PlaceMachine(id) => *id == item,
            _ => false,
        }
    }

    /// Check if this action matches a produced item (by ItemId)
    pub fn matches_produce_item_id(&self, item: ItemId) -> bool {
        match self {
            TutorialAction::ProduceItem(id) => *id == item,
            _ => false,
        }
    }
}

/// Tutorial step definition
pub struct TutorialStep {
    pub id: &'static str,
    pub description: &'static str,
    pub hint: &'static str,
    pub action: TutorialAction,
}

/// All tutorial steps (lazily initialized to use ItemId)
static TUTORIAL_STEPS_VEC: std::sync::LazyLock<Vec<TutorialStep>> =
    std::sync::LazyLock::new(|| {
        use crate::core::items;
        vec![
            TutorialStep {
                id: "tut_move",
                description: "WASDで移動しよう",
                hint: "WASDキーで移動、マウスで視点操作",
                action: TutorialAction::Move { distance: 20 },
            },
            TutorialStep {
                id: "tut_break",
                description: "ブロックを掘ろう",
                hint: "左クリックで採掘",
                action: TutorialAction::BreakBlock,
            },
            TutorialStep {
                id: "tut_inventory",
                description: "Eでインベントリを開こう",
                hint: "Eキーでインベントリを開閉",
                action: TutorialAction::OpenInventory,
            },
            TutorialStep {
                id: "tut_place_miner",
                description: "採掘機を設置しよう",
                hint: "ホットバーから採掘機を選択して右クリック",
                action: TutorialAction::PlaceMachine(items::miner_block()),
            },
            TutorialStep {
                id: "tut_place_conveyor",
                description: "コンベアを3個繋げよう",
                hint: "コンベアを選択して連続設置",
                action: TutorialAction::PlaceConveyors { count: 3 },
            },
            TutorialStep {
                id: "tut_place_furnace",
                description: "精錬炉を設置しよう",
                hint: "コンベアの先に精錬炉を設置",
                action: TutorialAction::PlaceMachine(items::furnace_block()),
            },
            TutorialStep {
                id: "tut_first_ingot",
                description: "インゴットを作ろう",
                hint: "採掘機→コンベア→精錬炉の接続を待つ",
                action: TutorialAction::ProduceItem(items::iron_ingot()),
            },
        ]
    });

/// Get all tutorial steps
pub fn tutorial_steps() -> &'static [TutorialStep] {
    &TUTORIAL_STEPS_VEC
}

/// Tutorial progress tracking
#[derive(Resource, Default)]
pub struct TutorialProgress {
    /// Current tutorial step index (0-based)
    pub current_step: usize,
    /// Whether all tutorials are completed
    pub completed: bool,
    /// Accumulated move distance for move tutorial
    pub move_distance: f32,
    /// Consecutive conveyor placement count
    pub conveyor_count: u32,
    /// Last conveyor position (for consecutive check)
    pub last_conveyor_pos: Option<IVec3>,
}

impl TutorialProgress {
    /// Get current tutorial step, if any
    pub fn current(&self) -> Option<&'static TutorialStep> {
        if self.completed {
            None
        } else {
            tutorial_steps().get(self.current_step)
        }
    }

    /// Advance to next step
    pub fn advance(&mut self) {
        self.current_step += 1;
        // Reset tracking values
        self.move_distance = 0.0;
        self.conveyor_count = 0;
        self.last_conveyor_pos = None;

        if self.current_step >= tutorial_steps().len() {
            self.completed = true;
        }
    }
}

/// Marker for tutorial UI panel
#[derive(Component)]
pub struct TutorialPanel;

/// Marker for tutorial step text
#[derive(Component)]
pub struct TutorialStepText;

/// Marker for tutorial progress text (e.g., "3/5")
#[derive(Component)]
pub struct TutorialProgressText;

/// Marker for tutorial progress bar background
#[derive(Component)]
pub struct TutorialProgressBarBg;

/// Marker for tutorial progress bar fill
#[derive(Component)]
pub struct TutorialProgressBarFill;

/// Quest definition
/// Note: systems/quest.rs has its own QuestDef struct
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct QuestDef {
    /// Quest description
    pub description: &'static str,
    /// Required item type
    pub required_item: ItemId,
    /// Required amount
    pub required_amount: u32,
    /// Rewards: (ItemId, amount)
    pub rewards: Vec<(ItemId, u32)>,
}

#[allow(dead_code)]
impl QuestDef {
    /// Create a new quest definition using ItemIds
    pub fn new(
        description: &'static str,
        required_item: ItemId,
        required_amount: u32,
        rewards: Vec<(ItemId, u32)>,
    ) -> Self {
        Self {
            description,
            required_item,
            required_amount,
            rewards,
        }
    }

    /// Get required item as ItemId
    pub fn required_item_id(&self) -> ItemId {
        self.required_item
    }

    /// Get rewards as ItemIds
    pub fn rewards_ids(&self) -> &[(ItemId, u32)] {
        &self.rewards
    }
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

// NOTE: SubQuestState and ActiveSubQuests removed (dead code)
// Sub-quest system defined in game_spec/mod.rs but not yet implemented
// Reimplement when sub-quest UI and logic are added

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

/// Delivery platform - accepts items from conveyors and stores in GlobalInventory
#[derive(Component)]
pub struct DeliveryPlatform {
    /// Position of the platform origin (bottom-left corner)
    pub position: IVec3,
}

impl DeliveryPlatform {
    pub fn new(position: IVec3) -> Self {
        Self { position }
    }
}

/// Marker for delivery platform UI
#[allow(dead_code)]
#[derive(Component)]
pub struct DeliveryUI;

/// Marker for delivery UI text
#[derive(Component)]
pub struct DeliveryUIText;

/// All available items for creative mode, organized by category
static CREATIVE_ITEMS_VEC: std::sync::LazyLock<Vec<(ItemId, &'static str)>> =
    std::sync::LazyLock::new(|| {
        use crate::core::items;
        vec![
            // Blocks
            (items::stone(), "Blocks"),
            (items::grass(), "Blocks"),
            // Ores
            (items::iron_ore(), "Ores"),
            (items::copper_ore(), "Ores"),
            (items::coal(), "Ores"),
            // Ingots
            (items::iron_ingot(), "Ingots"),
            (items::copper_ingot(), "Ingots"),
            // Machines
            (items::miner_block(), "Machines"),
            (items::conveyor_block(), "Machines"),
            (items::crusher_block(), "Machines"),
            (items::furnace_block(), "Machines"),
        ]
    });

/// Get creative items as ItemIds with categories
pub fn creative_items() -> &'static [(ItemId, &'static str)] {
    &CREATIVE_ITEMS_VEC
}

/// State for save/load operations
#[derive(Resource, Default)]
pub struct SaveLoadState {
    /// Pending load data (applied on next frame to avoid borrow conflicts)
    #[allow(dead_code)]
    pub pending_load: Option<crate::save::SaveDataV2>,
    /// Last save/load message for display
    #[allow(dead_code)]
    pub last_message: Option<String>,
}

/// Resource to hold item sprite textures for UI
#[derive(Resource, Default)]
pub struct ItemSprites {
    /// Textures indexed by ItemId
    pub textures: HashMap<ItemId, Handle<Image>>,
}

impl ItemSprites {
    /// Get sprite handle for an ItemId
    pub fn get_id(&self, item_id: ItemId) -> Option<Handle<Image>> {
        self.textures.get(&item_id).cloned()
    }

    /// Insert a sprite for an ItemId
    pub fn insert_id(&mut self, item_id: ItemId, handle: Handle<Image>) {
        self.textures.insert(item_id, handle);
    }

    /// Check if a sprite exists for an ItemId
    pub fn contains_id(&self, item_id: ItemId) -> bool {
        self.textures.contains_key(&item_id)
    }

    /// Check if any sprites are still loading
    pub fn any_loading(&self, assets: &AssetServer) -> bool {
        self.textures
            .values()
            .any(|h| matches!(assets.load_state(h.id()), bevy::asset::LoadState::Loading))
    }

    /// Get iterator over all textures
    pub fn iter(&self) -> impl Iterator<Item = (&ItemId, &Handle<Image>)> {
        self.textures.iter()
    }
}
