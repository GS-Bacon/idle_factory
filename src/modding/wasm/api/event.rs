//! イベント関連ホスト関数

use super::super::{ModState, WasmError};
use wasmtime::{Caller, Linker};

/// イベント関連ホスト関数を登録
pub fn register(linker: &mut Linker<ModState>) -> Result<(), WasmError> {
    linker
        .func_wrap("env", "host_subscribe_event", host_subscribe_event)
        .map_err(|e| WasmError::LinkError(e.to_string()))?;

    linker
        .func_wrap("env", "host_emit_event", host_emit_event)
        .map_err(|e| WasmError::LinkError(e.to_string()))?;

    Ok(())
}

/// Subscribe to game events
///
/// Event types: 0=BlockPlace, 1=BlockBreak, 2=ItemDeliver, 3=MachineComplete
///
/// # Returns
/// Subscription ID (>= 0 on success, negative on error)
fn host_subscribe_event(_caller: Caller<'_, ModState>, event_type: u32) -> i32 {
    // TODO: イベント購読の実装
    tracing::debug!("host_subscribe_event called for type {}", event_type);
    0 // 仮のsubscription_id
}

/// Emit a custom event
///
/// # Returns
/// Nothing (void function)
fn host_emit_event(
    mut caller: Caller<'_, ModState>,
    event_type: u32,
    data_ptr: u32,
    data_len: u32,
) {
    // データを読み取り
    if let Some(data) = read_bytes(&mut caller, data_ptr, data_len) {
        let mod_id = &caller.data().mod_id;
        tracing::debug!(
            "[Mod:{}] emit_event type={} data_len={}",
            mod_id,
            event_type,
            data.len()
        );
        // TODO: 実際のイベント発火
    }
}

/// WASMメモリからバイト列を読み取る
fn read_bytes(caller: &mut Caller<'_, ModState>, ptr: u32, len: u32) -> Option<Vec<u8>> {
    let memory = caller.get_export("memory")?.into_memory()?;
    let data = memory.data(&caller);
    let slice = data.get(ptr as usize..(ptr + len) as usize)?;
    Some(slice.to_vec())
}
