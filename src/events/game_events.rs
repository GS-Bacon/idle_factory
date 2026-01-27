//! Game event definitions

use crate::core::ItemId;
use bevy::prelude::*;

/// イベント発生源
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventSource {
    Player(Entity),
    Machine(Entity),
    System,
}

// ========== ブロック系 ==========

/// ブロック配置完了イベント
#[derive(Message, Debug)]
pub struct BlockPlaced {
    pub pos: IVec3,
    pub block: ItemId,
    pub source: EventSource,
}

/// ブロック破壊完了イベント
#[derive(Message, Debug)]
pub struct BlockBroken {
    pub pos: IVec3,
    pub block: ItemId,
    pub source: EventSource,
}

// ========== 機械系 ==========

/// 機械生成イベント
#[derive(Message, Debug)]
pub struct MachineSpawned {
    pub entity: Entity,
    pub machine_type: ItemId,
    pub pos: IVec3,
}

/// 機械加工開始イベント
#[derive(Message, Debug)]
pub struct MachineStarted {
    pub entity: Entity,
    pub inputs: Vec<(ItemId, u32)>,
}

/// 機械加工完了イベント
#[derive(Message, Debug)]
pub struct MachineCompleted {
    pub entity: Entity,
    pub outputs: Vec<(ItemId, u32)>,
}

// ========== インベントリ系 ==========

/// インベントリ変更イベント
#[derive(Message, Debug, Clone)]
pub struct InventoryChanged {
    pub entity: Entity,
    pub item_id: ItemId,
    pub delta: i32, // 正=追加、負=消費
}

// ========== 物流系 ==========

/// コンベア転送イベント
#[derive(Message, Debug)]
pub struct ConveyorTransfer {
    pub from_pos: IVec3,
    pub to_pos: IVec3,
    pub item: ItemId,
}

/// アイテム納品イベント
#[derive(Message, Debug)]
pub struct ItemDelivered {
    pub item: ItemId,
    pub count: u32,
}

/// イベント登録プラグイン
pub struct GameEventsExtPlugin;

impl Plugin for GameEventsExtPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BlockPlaced>()
            .add_message::<BlockBroken>()
            .add_message::<MachineSpawned>()
            .add_message::<MachineStarted>()
            .add_message::<MachineCompleted>()
            .add_message::<InventoryChanged>()
            .add_message::<ConveyorTransfer>()
            .add_message::<ItemDelivered>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::items;

    #[test]
    fn test_event_source() {
        let source = EventSource::System;
        assert_eq!(source, EventSource::System);
    }

    #[test]
    fn test_block_placed_event() {
        let event = BlockPlaced {
            pos: IVec3::new(1, 2, 3),
            block: items::stone(),
            source: EventSource::System,
        };
        assert_eq!(event.pos, IVec3::new(1, 2, 3));
    }
}
