use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use crate::gameplay::grid::{Direction, MachineInstance};
use std::collections::HashMap;

// A placeholder for all player inputs in a single tick
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerInput {
    // TODO: Define actual inputs, e.g.,
    // pub keys_pressed: Vec<KeyCode>,
    // pub mouse_delta: (f32, f32),
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClientMessage {
    JoinRequest { player_name: String },
    PlayerInput { tick: u32, inputs: PlayerInput },
    PlaceBlock { pos: IVec3, block_id: String, orientation: Direction },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServerMessage {
    Welcome {
        player_id: u64,
        game_state: InitialGameState,
    },
    GameStateUpdate {
        tick: u32,
        delta: GameStateDelta,
    },
    PlayerConnected {
        player_id: u64,
        player_name: String,
    },
    PlayerDisconnected {
        player_id: u64,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitialGameState {
    pub grid: HashMap<IVec3, MachineInstance>,
    // pub players: HashMap<u64, PlayerState>, // etc.
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameStateDelta {
    pub updated_machines: Vec<(IVec3, MachineInstance)>,
    pub removed_machines: Vec<IVec3>,
    // etc. for items, players...
}
