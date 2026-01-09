//! インベントリ関連ホスト関数

use super::super::{ModState, WasmError};
use wasmtime::{Caller, Linker};

/// インベントリ関連ホスト関数を登録
pub fn register(linker: &mut Linker<ModState>) -> Result<(), WasmError> {
    linker
        .func_wrap("env", "host_get_inventory_slot", host_get_inventory_slot)
        .map_err(|e| WasmError::LinkError(e.to_string()))?;

    linker
        .func_wrap("env", "host_transfer_item", host_transfer_item)
        .map_err(|e| WasmError::LinkError(e.to_string()))?;

    Ok(())
}

/// Get inventory slot contents
///
/// # ja
/// インベントリスロットの内容を取得
///
/// # Returns
/// Upper 32 bits: item_id, Lower 32 bits: count (0 = empty slot)
fn host_get_inventory_slot(_caller: Caller<'_, ModState>, entity_id: u64, slot: u32) -> u64 {
    // TODO: 実際のインベントリからの取得
    tracing::debug!(
        "host_get_inventory_slot called for entity {} slot {}",
        entity_id,
        slot
    );
    0 // 空スロット
}

/// Transfer items between entities
///
/// # ja
/// エンティティ間でアイテムを転送
///
/// # Returns
/// - 0: Success
/// - -1: Source entity error
/// - -2: Destination entity error
/// - -3: Insufficient items
fn host_transfer_item(
    _caller: Caller<'_, ModState>,
    from_entity: u64,
    to_entity: u64,
    item_id: u32,
    count: u32,
) -> i32 {
    // TODO: 実際のアイテム転送
    tracing::debug!(
        "host_transfer_item called: {} -> {} (item={}, count={})",
        from_entity,
        to_entity,
        item_id,
        count
    );
    0 // 成功
}
