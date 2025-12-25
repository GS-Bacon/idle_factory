//! ネットワークメッセージ定義（将来実装予定）
//!
//! マルチプレイヤー通信用のメッセージ型を定義します。
//! 現在はスタブで、実際の通信には使用されていません。

use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use crate::gameplay::grid::{Direction, MachineInstance};
use std::collections::HashMap;

/// プレイヤー入力（1ティック分）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerInput {
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
