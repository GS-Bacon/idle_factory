// src/core/encryption.rs
//! セーブデータ暗号化システム (C3: AES-256-GCM)
//!
//! - セーブデータを暗号化してチート防止
//! - Steam実績の保護
//! - キーはアプリケーション固有のシードから生成

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::Rng;
use std::fs;
use std::path::Path;

/// 暗号化キーのシード（本番環境では環境変数や別ファイルから読み込む）
const KEY_SEED: &[u8; 32] = b"InfiniteVoxelFactory_SaveKey123!";

/// Nonce サイズ (96 bits = 12 bytes)
const NONCE_SIZE: usize = 12;

/// 暗号化エラー
#[derive(Debug)]
pub enum EncryptionError {
    /// 暗号化失敗
    EncryptionFailed(String),
    /// 復号化失敗
    DecryptionFailed(String),
    /// ファイルI/Oエラー
    IoError(String),
    /// Base64デコードエラー
    DecodeError(String),
}

impl std::fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionError::EncryptionFailed(msg) => write!(f, "Encryption failed: {}", msg),
            EncryptionError::DecryptionFailed(msg) => write!(f, "Decryption failed: {}", msg),
            EncryptionError::IoError(msg) => write!(f, "IO error: {}", msg),
            EncryptionError::DecodeError(msg) => write!(f, "Decode error: {}", msg),
        }
    }
}

impl std::error::Error for EncryptionError {}

/// データを暗号化
///
/// # Arguments
/// * `data` - 暗号化する平文データ
///
/// # Returns
/// * 暗号化されたデータ（Nonce + 暗号文）のBase64エンコード文字列
pub fn encrypt_data(data: &[u8]) -> Result<String, EncryptionError> {
    let cipher = Aes256Gcm::new(KEY_SEED.into());

    // ランダムなNonceを生成
    let mut rng = rand::thread_rng();
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rng.fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 暗号化
    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

    // Nonce + 暗号文を結合してBase64エンコード
    let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&combined))
}

/// 暗号化されたデータを復号化
///
/// # Arguments
/// * `encrypted` - Base64エンコードされた暗号データ
///
/// # Returns
/// * 復号化された平文データ
pub fn decrypt_data(encrypted: &str) -> Result<Vec<u8>, EncryptionError> {
    let combined = BASE64
        .decode(encrypted)
        .map_err(|e| EncryptionError::DecodeError(e.to_string()))?;

    if combined.len() < NONCE_SIZE {
        return Err(EncryptionError::DecryptionFailed(
            "Invalid encrypted data: too short".to_string(),
        ));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(KEY_SEED.into());

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))
}

/// 暗号化してファイルに保存
///
/// # Arguments
/// * `path` - 保存先パス
/// * `data` - 保存するデータ
pub fn save_encrypted<P: AsRef<Path>>(path: P, data: &[u8]) -> Result<(), EncryptionError> {
    let encrypted = encrypt_data(data)?;
    fs::write(path, encrypted).map_err(|e| EncryptionError::IoError(e.to_string()))
}

/// 暗号化ファイルを読み込んで復号化
///
/// # Arguments
/// * `path` - 読み込むファイルパス
///
/// # Returns
/// * 復号化されたデータ
pub fn load_encrypted<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, EncryptionError> {
    let encrypted =
        fs::read_to_string(path).map_err(|e| EncryptionError::IoError(e.to_string()))?;
    decrypt_data(&encrypted)
}

/// JSONデータを暗号化して保存
pub fn save_encrypted_json<T: serde::Serialize, P: AsRef<Path>>(
    path: P,
    data: &T,
) -> Result<(), EncryptionError> {
    let json = serde_json::to_string(data)
        .map_err(|e| EncryptionError::EncryptionFailed(format!("JSON serialize error: {}", e)))?;
    save_encrypted(path, json.as_bytes())
}

/// 暗号化されたJSONを読み込んで復号化
pub fn load_encrypted_json<T: serde::de::DeserializeOwned, P: AsRef<Path>>(
    path: P,
) -> Result<T, EncryptionError> {
    let decrypted = load_encrypted(path)?;
    let json_str = String::from_utf8(decrypted)
        .map_err(|e| EncryptionError::DecryptionFailed(format!("UTF-8 decode error: {}", e)))?;
    serde_json::from_str(&json_str)
        .map_err(|e| EncryptionError::DecryptionFailed(format!("JSON parse error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = b"Hello, World! This is a test.";
        let encrypted = encrypt_data(original).unwrap();
        let decrypted = decrypt_data(&encrypted).unwrap();
        assert_eq!(original.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_produces_different_output() {
        let data = b"Same data";
        let encrypted1 = encrypt_data(data).unwrap();
        let encrypted2 = encrypt_data(data).unwrap();
        // ランダムNonceなので毎回異なる暗号文になる
        assert_ne!(encrypted1, encrypted2);
    }

    #[test]
    fn test_decrypt_invalid_data() {
        let result = decrypt_data("invalid_base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_tampered_data() {
        let encrypted = encrypt_data(b"test data").unwrap();
        // Base64文字列を改ざん
        let tampered = format!("{}X", &encrypted[..encrypted.len() - 1]);
        let result = decrypt_data(&tampered);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_roundtrip() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestData {
            name: String,
            value: i32,
        }

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let json = serde_json::to_string(&data).unwrap();
        let encrypted = encrypt_data(json.as_bytes()).unwrap();
        let decrypted = decrypt_data(&encrypted).unwrap();
        let restored: TestData = serde_json::from_slice(&decrypted).unwrap();

        assert_eq!(data, restored);
    }
}
