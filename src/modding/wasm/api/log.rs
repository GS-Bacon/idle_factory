//! ログ関連ホスト関数

use super::super::{ModState, WasmError};
use wasmtime::{Caller, Linker};

/// ログ関連ホスト関数を登録
pub fn register(linker: &mut Linker<ModState>) -> Result<(), WasmError> {
    linker
        .func_wrap("env", "host_log_info", host_log_info)
        .map_err(|e| WasmError::LinkError(e.to_string()))?;

    linker
        .func_wrap("env", "host_log_error", host_log_error)
        .map_err(|e| WasmError::LinkError(e.to_string()))?;

    Ok(())
}

fn host_log_info(mut caller: Caller<'_, ModState>, ptr: u32, len: u32) {
    if let Some(msg) = read_string(&mut caller, ptr, len) {
        let mod_id = &caller.data().mod_id;
        tracing::info!("[Mod:{}] {}", mod_id, msg);
    }
}

fn host_log_error(mut caller: Caller<'_, ModState>, ptr: u32, len: u32) {
    if let Some(msg) = read_string(&mut caller, ptr, len) {
        let mod_id = &caller.data().mod_id;
        tracing::error!("[Mod:{}] {}", mod_id, msg);
    }
}

/// WASMメモリから文字列を読み取る
fn read_string(caller: &mut Caller<'_, ModState>, ptr: u32, len: u32) -> Option<String> {
    let memory = caller.get_export("memory")?.into_memory()?;
    let data = memory.data(&caller);
    let slice = data.get(ptr as usize..(ptr + len) as usize)?;
    String::from_utf8(slice.to_vec()).ok()
}
