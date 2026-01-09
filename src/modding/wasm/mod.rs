//! WASM Mod ランタイム
//!
//! Core Mod (WASM) の読み込みと実行を管理する

#[cfg(not(target_arch = "wasm32"))]
pub mod api;
#[cfg(not(target_arch = "wasm32"))]
pub mod loader;
#[cfg(not(target_arch = "wasm32"))]
pub mod runtime;

#[cfg(not(target_arch = "wasm32"))]
pub use loader::WasmModLoader;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::{ModState, WasmError, WasmRuntime};
