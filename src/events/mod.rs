//! Game events for state changes
//! These events will be used for multiplayer synchronization in the future

#![allow(dead_code)] // These events are prepared for future multiplayer support

use bevy::prelude::*;
use crate::block_type::BlockType;

/// Event for block placement
#[derive(Event, Clone, Debug)]
pub struct BlockPlaceEvent {
    pub position: IVec3,
    pub block_type: BlockType,
    pub player_id: u64,
}

/// Event for block destruction
#[derive(Event, Clone, Debug)]
pub struct BlockBreakEvent {
    pub position: IVec3,
    pub player_id: u64,
}

/// Event for machine interaction
#[derive(Event, Clone, Debug)]
pub struct MachineInteractEvent {
    pub position: IVec3,
    pub action: MachineAction,
    pub player_id: u64,
}

/// Types of machine interactions
#[derive(Clone, Debug)]
pub enum MachineAction {
    Open,
    Close,
    AddItem(BlockType),
    TakeItem(BlockType),
}

/// Event for item transfer between machines/conveyors
#[derive(Event, Clone, Debug)]
pub struct ItemTransferEvent {
    pub from_pos: IVec3,
    pub to_pos: IVec3,
    pub item: BlockType,
    pub count: u32,
}

/// Event for quest progress
#[derive(Event, Clone, Debug)]
pub struct QuestProgressEvent {
    pub item_type: BlockType,
    pub amount: u32,
}

/// Plugin for game events
pub struct GameEventsPlugin;

impl Plugin for GameEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BlockPlaceEvent>()
            .add_event::<BlockBreakEvent>()
            .add_event::<MachineInteractEvent>()
            .add_event::<ItemTransferEvent>()
            .add_event::<QuestProgressEvent>();
    }
}
