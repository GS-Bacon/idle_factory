//! UI-related components and resources

use crate::BlockType;
use bevy::prelude::*;

// === Inventory UI ===

/// Inventory open state (toggle with E key)
#[derive(Resource, Default)]
pub struct InventoryOpen(pub bool);

/// Marker for inventory UI panel (full inventory overlay)
#[derive(Component)]
pub struct InventoryUI;

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
pub struct HeldItem(pub Option<(BlockType, u32)>);

/// Marker for held item cursor display
#[derive(Component)]
pub struct HeldItemDisplay;

/// Marker for held item count text
#[derive(Component)]
pub struct HeldItemText;

/// Marker for held item sprite image
#[derive(Component)]
pub struct HeldItemImage;

/// Creative inventory item button - stores the BlockType it represents
#[derive(Component)]
pub struct CreativeItemButton(pub BlockType);

/// Marker for creative catalog item sprite image
#[derive(Component)]
pub struct CreativeItemImage(pub BlockType);

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

// === Machine UI ===

/// Currently interacting furnace entity
#[derive(Resource, Default)]
pub struct InteractingFurnace(pub Option<Entity>);

/// Marker for furnace UI panel
#[derive(Component)]
pub struct FurnaceUI;

/// Marker for furnace UI text
#[derive(Component)]
pub struct FurnaceUIText;

/// Machine slot type for unified slot handling
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MachineSlotType {
    Fuel,
    Input,
    Output,
}

/// Machine slot button component
#[derive(Component)]
pub struct MachineSlotButton(pub MachineSlotType);

/// Currently interacting crusher entity
#[derive(Resource, Default)]
pub struct InteractingCrusher(pub Option<Entity>);

/// Marker for crusher UI panel
#[derive(Component)]
pub struct CrusherUI;

/// Crusher UI progress bar fill
#[derive(Component)]
pub struct CrusherProgressBar;

/// Crusher UI slot button
#[derive(Component)]
pub struct CrusherSlotButton(pub MachineSlotType);

/// Currently interacting miner entity
#[derive(Resource, Default)]
pub struct InteractingMiner(pub Option<Entity>);

/// Marker for miner UI panel
#[derive(Component)]
pub struct MinerUI;

/// Miner UI buffer slot button (take buffer contents)
#[derive(Component)]
pub struct MinerBufferButton;

/// Miner UI clear button (discard buffer)
#[derive(Component)]
pub struct MinerClearButton;

/// Miner UI buffer count text
#[derive(Component)]
pub struct MinerBufferCountText;

/// Machine UI progress bar fill
#[derive(Component)]
pub struct MachineProgressBar;

/// Machine UI slot count text
#[derive(Component)]
pub struct MachineSlotCount(pub MachineSlotType);

/// Crusher UI slot count text
#[derive(Component)]
pub struct CrusherSlotCount(pub MachineSlotType);

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
}

/// Marker for command input UI container
#[derive(Component)]
pub struct CommandInputUI;

/// Marker for command input text display
#[derive(Component)]
pub struct CommandInputText;

// === Global Inventory UI ===

/// Global inventory open state (toggle with Tab key)
#[derive(Resource, Default)]
pub struct GlobalInventoryOpen(pub bool);

/// Marker for global inventory UI panel
#[derive(Component)]
pub struct GlobalInventoryUI;

/// Marker for global inventory slot
#[derive(Component)]
pub struct GlobalInventorySlot(pub usize);

/// Marker for global inventory slot image
#[derive(Component)]
pub struct GlobalInventorySlotImage(pub usize);

/// Marker for global inventory slot count text
#[derive(Component)]
pub struct GlobalInventorySlotCount(pub usize);

/// Global inventory page state
#[derive(Resource, Default)]
pub struct GlobalInventoryPage(pub usize);

/// Marker for page navigation buttons
#[derive(Component)]
pub struct GlobalInventoryPageButton {
    pub next: bool, // true = next, false = prev
}

/// Marker for page indicator text
#[derive(Component)]
pub struct GlobalInventoryPageText;

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

    pub fn matches(&self, block_type: crate::BlockType) -> bool {
        use crate::BlockType;
        match self {
            ItemCategory::All => true,
            ItemCategory::Ores => matches!(
                block_type,
                BlockType::IronOre | BlockType::CopperOre | BlockType::Coal | BlockType::Stone
            ),
            ItemCategory::Ingots => matches!(
                block_type,
                BlockType::IronIngot | BlockType::CopperIngot
            ),
            ItemCategory::Machines => matches!(
                block_type,
                BlockType::MinerBlock
                    | BlockType::ConveyorBlock
                    | BlockType::FurnaceBlock
                    | BlockType::CrusherBlock
            ),
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

