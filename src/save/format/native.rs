//! Native file I/O functions for save/load operations

use super::timer::SaveSlotInfo;
use super::v2::SaveDataV2;
use super::SAVE_DIR;
use std::fs;

/// Get the saves directory path
pub fn get_save_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(SAVE_DIR)
}

/// Ensure save directory exists
pub fn ensure_save_dir() -> std::io::Result<()> {
    let dir = get_save_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(())
}

/// Save game data in V2 format
pub fn save_game_v2(data: &SaveDataV2, filename: &str) -> Result<(), String> {
    ensure_save_dir().map_err(|e| format!("Failed to create save directory: {}", e))?;

    let path = get_save_dir().join(format!("{}.json", filename));
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize save data: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("Failed to write save file: {}", e))?;

    Ok(())
}

/// Load game data in V2 format
pub fn load_game_v2(filename: &str) -> Result<SaveDataV2, String> {
    let path = get_save_dir().join(format!("{}.json", filename));

    if !path.exists() {
        return Err(format!("Save file not found: {}", filename));
    }

    let json = fs::read_to_string(&path).map_err(|e| format!("Failed to read save file: {}", e))?;

    serde_json::from_str(&json).map_err(|e| format!("Failed to parse save data: {}", e))
}

/// List all save files
#[allow(dead_code)]
pub fn list_saves() -> Result<Vec<SaveSlotInfo>, String> {
    let dir = get_save_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut saves = Vec::new();
    let entries =
        fs::read_dir(&dir).map_err(|e| format!("Failed to read save directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                // Try to read timestamp from file
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(data) = serde_json::from_str::<SaveDataV2>(&json) {
                        saves.push(SaveSlotInfo {
                            filename: stem.to_string(),
                            timestamp: data.timestamp,
                        });
                    }
                }
            }
        }
    }

    // Sort by timestamp (newest first)
    saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(saves)
}

/// Delete a save file
#[allow(dead_code)]
pub fn delete_save(filename: &str) -> Result<(), String> {
    let path = get_save_dir().join(format!("{}.json", filename));

    if !path.exists() {
        return Err(format!("Save file not found: {}", filename));
    }

    fs::remove_file(&path).map_err(|e| format!("Failed to delete save file: {}", e))?;

    Ok(())
}
