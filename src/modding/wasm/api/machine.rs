//! 機械関連ホスト関数

use super::super::{ModState, WasmError};
use wasmtime::{Caller, Linker};

/// 機械関連ホスト関数を登録
pub fn register(linker: &mut Linker<ModState>) -> Result<(), WasmError> {
    linker
        .func_wrap("env", "host_get_machine_state", host_get_machine_state)
        .map_err(|e| WasmError::LinkError(e.to_string()))?;

    linker
        .func_wrap("env", "host_set_machine_enabled", host_set_machine_enabled)
        .map_err(|e| WasmError::LinkError(e.to_string()))?;

    Ok(())
}

/// 機械の状態を取得
/// 戻り値: 0=正常, 1=処理中, 2=待機中, -1=エラー
fn host_get_machine_state(_caller: Caller<'_, ModState>, entity_id: u64) -> i32 {
    // TODO: 実際のBevy Worldからの状態取得
    // 現在はスタブ実装
    tracing::debug!("host_get_machine_state called for entity {}", entity_id);
    0 // 正常
}

/// 機械の有効/無効を設定
/// 戻り値: 0=成功, -1=エンティティ不存在, -2=権限エラー
fn host_set_machine_enabled(_caller: Caller<'_, ModState>, entity_id: u64, enabled: i32) -> i32 {
    // TODO: 実際のBevy Worldへの状態設定
    tracing::debug!(
        "host_set_machine_enabled called for entity {} -> {}",
        entity_id,
        enabled != 0
    );
    0 // 成功
}
