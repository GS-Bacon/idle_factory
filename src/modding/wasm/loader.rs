//! WASMファイルローダー

use super::WasmError;
use std::path::Path;

/// WASMファイルローダー
pub struct WasmModLoader;

impl WasmModLoader {
    /// ファイルパスからWASMバイトを読み込む
    pub fn load_from_path(path: &Path) -> Result<Vec<u8>, WasmError> {
        std::fs::read(path).map_err(WasmError::from)
    }

    /// WASMバイトを検証
    pub fn validate_wasm(bytes: &[u8]) -> Result<(), WasmError> {
        // WASMマジックナンバーをチェック
        if bytes.len() < 8 {
            return Err(WasmError::CompileError("File too small".to_string()));
        }

        // WASM magic: \0asm
        if &bytes[0..4] != b"\0asm" {
            return Err(WasmError::CompileError(
                "Invalid WASM magic number".to_string(),
            ));
        }

        // バージョンチェック（1が標準）
        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        if version != 1 {
            return Err(WasmError::CompileError(format!(
                "Unsupported WASM version: {}",
                version
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_wasm_magic() {
        // 有効なWASMヘッダー
        let valid = b"\0asm\x01\x00\x00\x00";
        assert!(WasmModLoader::validate_wasm(valid).is_ok());

        // 無効なヘッダー
        let invalid = b"invalid!";
        assert!(WasmModLoader::validate_wasm(invalid).is_err());

        // 短すぎる
        let short = b"\0asm";
        assert!(WasmModLoader::validate_wasm(short).is_err());
    }

    #[test]
    fn test_validate_wasm_version() {
        // バージョン1（有効）
        let v1 = b"\0asm\x01\x00\x00\x00";
        assert!(WasmModLoader::validate_wasm(v1).is_ok());

        // バージョン2（無効）
        let v2 = b"\0asm\x02\x00\x00\x00";
        let result = WasmModLoader::validate_wasm(v2);
        assert!(matches!(result, Err(WasmError::CompileError(_))));
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = WasmModLoader::load_from_path(Path::new("/nonexistent/path/to/file.wasm"));
        assert!(matches!(result, Err(WasmError::IoError(_))));
    }
}
