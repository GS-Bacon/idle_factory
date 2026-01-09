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

/// インベントリスロットを取得
/// 戻り値: 上位32bit=item_id, 下位32bit=count
fn host_get_inventory_slot(_caller: Caller<'_, ModState>, entity_id: u64, slot: u32) -> u64 {
    // TODO: 実際のインベントリからの取得
    tracing::debug!(
        "host_get_inventory_slot called for entity {} slot {}",
        entity_id,
        slot
    );
    0 // 空スロット
}

/// アイテムを転送
/// 戻り値: 0=成功, -1=転送元エラー, -2=転送先エラー, -3=アイテム不足
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
