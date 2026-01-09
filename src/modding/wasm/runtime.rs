//! WASMランタイム実装

use super::api;
use std::collections::HashMap;
use wasmtime::*;

/// WASMランタイムエラー
#[derive(Debug)]
pub enum WasmError {
    IoError(std::io::Error),
    CompileError(String),
    LinkError(String),
    RuntimeError(String),
    ModNotFound(String),
}

impl std::fmt::Display for WasmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmError::IoError(e) => write!(f, "IO error: {}", e),
            WasmError::CompileError(e) => write!(f, "Compile error: {}", e),
            WasmError::LinkError(e) => write!(f, "Link error: {}", e),
            WasmError::RuntimeError(e) => write!(f, "Runtime error: {}", e),
            WasmError::ModNotFound(id) => write!(f, "Mod not found: {}", id),
        }
    }
}

impl std::error::Error for WasmError {}

impl From<std::io::Error> for WasmError {
    fn from(e: std::io::Error) -> Self {
        WasmError::IoError(e)
    }
}

/// Modの実行コンテキスト
pub struct ModState {
    pub mod_id: String,
}

/// ロード済みModインスタンス
struct LoadedMod {
    instance: Instance,
    store: Store<ModState>,
}

/// WASMランタイム
pub struct WasmRuntime {
    engine: Engine,
    modules: HashMap<String, Module>,
    instances: HashMap<String, LoadedMod>,
}

impl WasmRuntime {
    /// 新しいランタイムを作成
    pub fn new() -> Result<Self, WasmError> {
        let engine = Engine::default();
        Ok(Self {
            engine,
            modules: HashMap::new(),
            instances: HashMap::new(),
        })
    }

    /// WASMモジュールをロード（コンパイルのみ）
    pub fn load_module(&mut self, mod_id: &str, wasm_bytes: &[u8]) -> Result<(), WasmError> {
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| WasmError::CompileError(e.to_string()))?;
        self.modules.insert(mod_id.to_string(), module);
        Ok(())
    }

    /// Modをインスタンス化（ホスト関数をリンク）
    pub fn instantiate(&mut self, mod_id: &str) -> Result<(), WasmError> {
        let module = self
            .modules
            .get(mod_id)
            .ok_or_else(|| WasmError::ModNotFound(mod_id.to_string()))?;

        let mut store = Store::new(
            &self.engine,
            ModState {
                mod_id: mod_id.to_string(),
            },
        );

        let mut linker = Linker::new(&self.engine);

        // 全ホスト関数を登録
        api::register_all(&mut linker)?;

        let instance = linker
            .instantiate(&mut store, module)
            .map_err(|e| WasmError::LinkError(e.to_string()))?;

        self.instances
            .insert(mod_id.to_string(), LoadedMod { instance, store });
        Ok(())
    }

    /// mod_init() を呼び出す
    pub fn call_init(&mut self, mod_id: &str) -> Result<i32, WasmError> {
        let loaded = self
            .instances
            .get_mut(mod_id)
            .ok_or_else(|| WasmError::ModNotFound(mod_id.to_string()))?;

        let init_fn = loaded
            .instance
            .get_typed_func::<(), i32>(&mut loaded.store, "mod_init")
            .map_err(|e| WasmError::LinkError(format!("mod_init not found: {}", e)))?;

        init_fn
            .call(&mut loaded.store, ())
            .map_err(|e| WasmError::RuntimeError(e.to_string()))
    }

    /// mod_tick() を呼び出す
    pub fn call_tick(&mut self, mod_id: &str, tick: u64) -> Result<(), WasmError> {
        let loaded = self
            .instances
            .get_mut(mod_id)
            .ok_or_else(|| WasmError::ModNotFound(mod_id.to_string()))?;

        // mod_tickがない場合はスキップ
        if let Ok(tick_fn) = loaded
            .instance
            .get_typed_func::<u64, ()>(&mut loaded.store, "mod_tick")
        {
            tick_fn
                .call(&mut loaded.store, tick)
                .map_err(|e| WasmError::RuntimeError(e.to_string()))?;
        }

        Ok(())
    }

    /// ロード済みMod一覧
    pub fn loaded_mods(&self) -> Vec<&str> {
        self.instances.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create WasmRuntime")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = WasmRuntime::new();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_wasm_error_display() {
        let errors = vec![
            WasmError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "test")),
            WasmError::CompileError("compile failed".to_string()),
            WasmError::LinkError("link failed".to_string()),
            WasmError::RuntimeError("runtime failed".to_string()),
            WasmError::ModNotFound("test_mod".to_string()),
        ];

        for error in errors {
            // Display should not panic
            let _ = format!("{}", error);
        }
    }

    #[test]
    fn test_mod_not_found_error() {
        let mut runtime = WasmRuntime::new().unwrap();
        let result = runtime.call_init("nonexistent_mod");
        assert!(matches!(result, Err(WasmError::ModNotFound(_))));
    }
}
