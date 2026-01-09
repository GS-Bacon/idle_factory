//! Modホットリロード機能（開発用）
//!
//! .wasmファイルの変更を検出して自動リロード

#[cfg(debug_assertions)]
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
#[cfg(debug_assertions)]
use std::collections::HashSet;
#[cfg(debug_assertions)]
use std::path::{Path, PathBuf};
#[cfg(debug_assertions)]
use std::sync::mpsc::{channel, Receiver, TryRecvError};
#[cfg(debug_assertions)]
use std::time::Duration;

/// ホットリロードエラー
#[derive(Debug)]
pub enum HotReloadError {
    WatchError(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for HotReloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HotReloadError::WatchError(e) => write!(f, "Watch error: {}", e),
            HotReloadError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for HotReloadError {}

impl From<std::io::Error> for HotReloadError {
    fn from(e: std::io::Error) -> Self {
        HotReloadError::IoError(e)
    }
}

/// 変更されたModの情報
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct ModChange {
    pub mod_id: String,
    pub wasm_path: PathBuf,
}

/// Modホットリローダー
#[cfg(debug_assertions)]
pub struct ModHotReloader {
    #[allow(dead_code)]
    watcher: RecommendedWatcher,
    rx: Receiver<Result<Event, notify::Error>>,
    mods_dir: PathBuf,
    known_mods: HashSet<String>,
}

#[cfg(debug_assertions)]
impl ModHotReloader {
    /// 新しいホットリローダーを作成
    pub fn new(mods_dir: &Path) -> Result<Self, HotReloadError> {
        let (tx, rx) = channel();

        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_secs(1)),
        )
        .map_err(|e| HotReloadError::WatchError(e.to_string()))?;

        Ok(Self {
            watcher,
            rx,
            mods_dir: mods_dir.to_path_buf(),
            known_mods: HashSet::new(),
        })
    }

    /// 監視を開始
    pub fn start_watching(&mut self) -> Result<(), HotReloadError> {
        // mods/ディレクトリを再帰的に監視
        self.watcher
            .watch(&self.mods_dir, RecursiveMode::Recursive)
            .map_err(|e| HotReloadError::WatchError(e.to_string()))?;

        tracing::info!("Hot reload watching: {:?}", self.mods_dir);
        Ok(())
    }

    /// 変更を取得（ノンブロッキング）
    pub fn poll_changes(&mut self) -> Vec<ModChange> {
        let mut changes = Vec::new();

        loop {
            match self.rx.try_recv() {
                Ok(Ok(event)) => {
                    for path in event.paths {
                        if let Some(change) = self.process_path(&path) {
                            // 重複を避ける
                            if !changes
                                .iter()
                                .any(|c: &ModChange| c.mod_id == change.mod_id)
                            {
                                changes.push(change);
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    tracing::warn!("Watch error: {}", e);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    tracing::error!("Watch channel disconnected");
                    break;
                }
            }
        }

        changes
    }

    /// パスを処理してModChangeを生成
    fn process_path(&mut self, path: &Path) -> Option<ModChange> {
        // .wasmファイルのみ対象
        if path.extension()?.to_str()? != "wasm" {
            return None;
        }

        // mod_idを推定（親ディレクトリ名）
        let mod_dir = path.parent()?;
        let mod_id = mod_dir.file_name()?.to_str()?.to_string();

        // modsディレクトリ配下か確認
        if !mod_dir.starts_with(&self.mods_dir) {
            return None;
        }

        self.known_mods.insert(mod_id.clone());

        Some(ModChange {
            mod_id,
            wasm_path: path.to_path_buf(),
        })
    }

    /// 既知のMod一覧を取得
    pub fn known_mods(&self) -> &HashSet<String> {
        &self.known_mods
    }
}

/// リリースビルドではスタブ実装
#[cfg(not(debug_assertions))]
pub struct ModChange {
    pub mod_id: String,
    pub wasm_path: std::path::PathBuf,
}

#[cfg(not(debug_assertions))]
pub struct ModHotReloader;

#[cfg(not(debug_assertions))]
impl ModHotReloader {
    pub fn new(_mods_dir: &std::path::Path) -> Result<Self, HotReloadError> {
        Ok(Self)
    }

    pub fn start_watching(&mut self) -> Result<(), HotReloadError> {
        // リリースビルドでは何もしない
        Ok(())
    }

    pub fn poll_changes(&mut self) -> Vec<ModChange> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    #[cfg(debug_assertions)]
    fn test_hot_reloader_creation() {
        let temp_dir = tempdir().unwrap();
        let result = ModHotReloader::new(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_process_wasm_path() {
        let temp_dir = tempdir().unwrap();
        let mods_dir = temp_dir.path().join("mods");
        fs::create_dir_all(&mods_dir).unwrap();

        let mod_dir = mods_dir.join("test_mod");
        fs::create_dir_all(&mod_dir).unwrap();

        let wasm_path = mod_dir.join("test_mod.wasm");
        fs::write(&wasm_path, b"fake wasm").unwrap();

        let mut reloader = ModHotReloader::new(&mods_dir).unwrap();
        let change = reloader.process_path(&wasm_path);

        assert!(change.is_some());
        let change = change.unwrap();
        assert_eq!(change.mod_id, "test_mod");
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_ignore_non_wasm() {
        let temp_dir = tempdir().unwrap();
        let mods_dir = temp_dir.path().join("mods");
        fs::create_dir_all(&mods_dir).unwrap();

        let mod_dir = mods_dir.join("test_mod");
        fs::create_dir_all(&mod_dir).unwrap();

        let txt_path = mod_dir.join("readme.txt");
        fs::write(&txt_path, "readme").unwrap();

        let mut reloader = ModHotReloader::new(&mods_dir).unwrap();
        let change = reloader.process_path(&txt_path);

        assert!(change.is_none());
    }
}
