//! Common machine output logic
//!
//! This module provides unified functions for machine-to-conveyor and
//! machine-to-machine item transfer.

use crate::{BlockType, Conveyor, Crusher, Direction, Furnace};
use bevy::prelude::*;

/// Result of a transfer attempt
pub struct TransferResult {
    pub transferred: bool,
}

/// Try to transfer an item to a conveyor at the output position
///
/// Returns true if the item was successfully transferred.
pub fn try_transfer_to_conveyor(
    source_pos: IVec3,
    output_pos: IVec3,
    block_type: BlockType,
    conveyor_query: &mut Query<&mut Conveyor>,
) -> bool {
    for mut conveyor in conveyor_query.iter_mut() {
        if conveyor.position == output_pos {
            if let Some(progress) = conveyor.get_join_progress(source_pos) {
                if conveyor.can_accept_item(progress) {
                    conveyor.add_item(block_type, progress);
                    return true;
                }
            }
        }
    }
    false
}

/// Try to transfer an item to a furnace at the output position
///
/// The furnace must be facing such that its back is at source_pos.
/// Returns true if the item was successfully transferred.
pub fn try_transfer_to_furnace(
    source_pos: IVec3,
    output_pos: IVec3,
    block_type: BlockType,
    furnace_query: &mut Query<&mut Furnace>,
) -> bool {
    for mut furnace in furnace_query.iter_mut() {
        let furnace_back = furnace.position - furnace.facing.to_ivec3();
        if furnace.position == output_pos
            && furnace_back == source_pos
            && furnace.can_add_input(block_type)
        {
            furnace.input_type = Some(block_type);
            furnace.input_count += 1;
            return true;
        }
    }
    false
}

/// Try to transfer an item to a crusher at the output position
///
/// The crusher must be facing such that its back is at source_pos.
/// Returns true if the item was successfully transferred.
pub fn try_transfer_to_crusher(
    source_pos: IVec3,
    output_pos: IVec3,
    block_type: BlockType,
    crusher_query: &mut Query<&mut Crusher>,
) -> bool {
    for mut crusher in crusher_query.iter_mut() {
        let crusher_back = crusher.position - crusher.facing.to_ivec3();
        if crusher.position == output_pos
            && crusher_back == source_pos
            && Crusher::can_crush(block_type)
            && (crusher.input_type.is_none() || crusher.input_type == Some(block_type))
            && crusher.input_count < 64
        {
            crusher.input_type = Some(block_type);
            crusher.input_count += 1;
            return true;
        }
    }
    false
}

/// Universal output transfer function
///
/// Tries to transfer an item from a machine to:
/// 1. Conveyor at output position (highest priority)
/// 2. Furnace at output position (if furnace accepts from this direction)
/// 3. Crusher at output position (if crusher accepts this item type)
///
/// Returns true if the item was successfully transferred to any target.
pub fn transfer_output(
    source_pos: IVec3,
    source_facing: Direction,
    block_type: BlockType,
    conveyor_query: &mut Query<&mut Conveyor>,
    furnace_query: &mut Query<&mut Furnace>,
    crusher_query: &mut Query<&mut Crusher>,
) -> bool {
    let output_pos = source_pos + source_facing.to_ivec3();

    // Priority 1: Conveyor
    if try_transfer_to_conveyor(source_pos, output_pos, block_type, conveyor_query) {
        return true;
    }

    // Priority 2: Furnace
    if try_transfer_to_furnace(source_pos, output_pos, block_type, furnace_query) {
        return true;
    }

    // Priority 3: Crusher
    if try_transfer_to_crusher(source_pos, output_pos, block_type, crusher_query) {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_result_default() {
        let result = TransferResult { transferred: false };
        assert!(!result.transferred);
    }
}
