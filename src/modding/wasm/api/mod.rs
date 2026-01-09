//! WASM Mod ホスト関数API
//!
//! Core Modから呼び出せるホスト関数を定義

pub mod event;
pub mod inventory;
pub mod log;
pub mod machine;

use super::{ModState, WasmError};
use wasmtime::Linker;

/// 全ホスト関数をLinkerに登録
pub fn register_all(linker: &mut Linker<ModState>) -> Result<(), WasmError> {
    log::register(linker)?;
    machine::register(linker)?;
    inventory::register(linker)?;
    event::register(linker)?;
    Ok(())
}
