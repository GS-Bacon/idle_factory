//! UI-related components and resources

use crate::core::ItemId;
use bevy::prelude::*;

// === Inventory UI ===

/// Inventory open state (toggle with E key)
#[derive(Resource, Default)]
pub struct InventoryOpen(pub bool);

/// Marker for inventory UI panel (full inventory overlay)
#[derive(Component)]
pub struct InventoryUI;

/// Marker for inventory background overlay (darkens screen)
#[derive(Component)]
pub struct InventoryBackgroundOverlay;

/// Marker for an inventory slot button (index 0-35)
#[derive(Component)]
pub struct InventorySlotUI(pub usize);

/// Marker for inventory slot sprite image
#[derive(Component)]
pub struct InventorySlotImage(pub usize);

/// Marker for trash slot
#[derive(Component)]
pub struct TrashSlot;

/// Currently held item for drag and drop
#[derive(Resource, Default)]
pub struct HeldItem(pub Option<(ItemId, u32)>);

/// Marker for held item cursor display
#[derive(Component)]
pub struct HeldItemDisplay;

/// Marker for held item count text
#[derive(Component)]
pub struct HeldItemText;

/// Marker for held item sprite image
#[derive(Component)]
pub struct HeldItemImage;

/// Creative inventory item button - stores the ItemId it represents
#[derive(Component)]
pub struct CreativeItemButton(pub ItemId);

/// Marker for creative catalog item sprite image
#[derive(Component)]
pub struct CreativeItemImage(pub ItemId);

/// Marker for the creative catalog panel (right side of inventory UI)
#[derive(Component)]
pub struct CreativePanel;

/// Marker for inventory tooltip (shown when hovering over slots)
#[derive(Component)]
pub struct InventoryTooltip;

// === Hotbar UI ===

/// Marker for hotbar UI container
#[derive(Component)]
pub struct HotbarUI;

/// Marker for hotbar slot (index 0-8)
#[derive(Component)]
pub struct HotbarSlot(pub usize);

/// Marker for hotbar slot count display
#[derive(Component)]
pub struct HotbarSlotCount(pub usize);

/// Marker for the hotbar item name display (shown above hotbar)
#[derive(Component)]
pub struct HotbarItemNameText;

/// Marker for hotbar slot sprite image
#[derive(Component)]
pub struct HotbarSlotImage(pub usize);

// === Generic Machine UI ===

/// Currently interacting machine entity (generic)
#[derive(Resource, Default)]
pub struct InteractingMachine(pub Option<Entity>);

/// Generic machine UI panel marker
/// Holds the machine spec id for identification
#[derive(Component)]
pub struct GenericMachineUI {
    pub machine_id: &'static str,
}

/// Generic machine UI slot button
/// slot_id corresponds to UiSlotDef.slot_id from MachineSpec
#[derive(Component)]
pub struct GenericMachineSlotButton {
    pub slot_id: u8,
    pub is_input: bool,
    pub is_fuel: bool,
}

/// Generic machine UI slot count text
#[derive(Component)]
pub struct GenericMachineSlotCount {
    pub slot_id: u8,
    pub is_input: bool,
    pub is_fuel: bool,
}

/// Generic machine UI progress bar
#[derive(Component)]
pub struct GenericMachineProgressBar;

/// Generic machine UI header text
#[derive(Component)]
pub struct GenericMachineHeaderText;

// === Command UI ===

/// Command input UI state
#[derive(Resource, Default)]
pub struct CommandInputState {
    /// Whether command input is open
    pub open: bool,
    /// Current text in the command input
    pub text: String,
    /// Skip input this frame (to avoid T/slash being added when opening)
    pub skip_input_frame: bool,
    /// Currently selected suggestion index
    pub suggestion_index: usize,
}

/// Available command suggestions
pub const COMMAND_SUGGESTIONS: &[&str] = &[
    "/creative",
    "/survival",
    "/give",
    "/clear",
    "/save",
    "/load",
    "/tp",
    "/look",
    "/setblock",
    "/spawn_machine",
    "/screenshot",
];

/// Marker for command suggestions UI
#[derive(Component)]
pub struct CommandSuggestionsUI;

/// Marker for command suggestion text
#[derive(Component)]
pub struct CommandSuggestionText(pub usize);

/// Marker for command input UI container
#[derive(Component)]
pub struct CommandInputUI;

/// Marker for command input text display
#[derive(Component)]
pub struct CommandInputText;

// === Upper Panel (Integrated in Inventory UI) ===

/// Global inventory page state (used for upper panel pagination)
#[derive(Resource, Default)]
pub struct GlobalInventoryPage(pub usize);

/// Marker for page navigation buttons
#[derive(Component)]
pub struct GlobalInventoryPageButton {
    pub next: bool, // true = next, false = prev
}

/// Item category for filtering
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ItemCategory {
    #[default]
    All,
    Ores,
    Ingots,
    Machines,
}

impl ItemCategory {
    pub fn label(&self) -> &'static str {
        match self {
            ItemCategory::All => "All",
            ItemCategory::Ores => "Ores",
            ItemCategory::Ingots => "Ingots",
            ItemCategory::Machines => "Machines",
        }
    }

    pub fn matches(&self, item_id: ItemId) -> bool {
        use crate::core::items;
        match self {
            ItemCategory::All => true,
            ItemCategory::Ores => {
                item_id == items::iron_ore()
                    || item_id == items::copper_ore()
                    || item_id == items::coal()
                    || item_id == items::stone()
            }
            ItemCategory::Ingots => {
                item_id == items::iron_ingot() || item_id == items::copper_ingot()
            }
            ItemCategory::Machines => {
                item_id == items::miner_block()
                    || item_id == items::conveyor_block()
                    || item_id == items::furnace_block()
                    || item_id == items::crusher_block()
            }
        }
    }
}

/// Current category filter for global inventory
#[derive(Resource, Default)]
pub struct GlobalInventoryCategory(pub ItemCategory);

/// Search text for global inventory
#[derive(Resource, Default)]
pub struct GlobalInventorySearch(pub String);

/// Marker for category tab button
#[derive(Component)]
pub struct GlobalInventoryCategoryTab(pub ItemCategory);

/// Marker for search input box
#[derive(Component)]
pub struct GlobalInventorySearchInput;

// === Breaking Progress UI ===

/// Marker for breaking progress bar container (centered on screen)
#[derive(Component)]
pub struct BreakingProgressUI;

/// Marker for breaking progress bar fill
#[derive(Component)]
pub struct BreakingProgressBarFill;

// === Pause UI ===

/// Marker for pause overlay UI
#[derive(Component)]
pub struct PauseUI;

// === 3D Held Item Display ===

/// Marker for 3D held item display (first-person view in bottom-right)
#[derive(Component)]
pub struct HeldItem3D;

/// Marker for 3D held item scene (for machine models using SceneRoot)
#[derive(Component)]
pub struct HeldItem3DScene;

/// Cached materials for held item 3D display
#[derive(Resource)]
pub struct HeldItem3DCache {
    pub cube_mesh: Handle<Mesh>,
    pub materials: std::collections::HashMap<ItemId, Handle<StandardMaterial>>,
}
