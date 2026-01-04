//! External updater for Idle Factory
//!
//! This is a standalone binary that downloads and installs updates.
//! It is launched by the main game when an update is available.
//!
//! Usage: updater.exe <version> <download_url>
//!
//! After download completes, it replaces idle_factory.exe and restarts the game.

use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

const GAME_EXE_NAME: &str = if cfg!(windows) {
    "idle_factory.exe"
} else {
    "idle_factory"
};

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: updater <version> <download_url>");
        eprintln!("This tool is meant to be launched by idle_factory, not directly.");
        wait_and_exit(1);
        return;
    }

    let version = &args[1];
    let download_url = &args[2];

    println!("=================================");
    println!("  Idle Factory Updater");
    println!("=================================");
    println!();
    println!("Updating to version: v{}", version);
    println!();

    // Wait a moment for the game to fully exit
    println!("Waiting for game to exit...");
    thread::sleep(Duration::from_secs(2));

    // Get the directory where updater is located
    let exe_path = match env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: Failed to get executable path: {}", e);
            wait_and_exit(1);
            return;
        }
    };

    let exe_dir = match exe_path.parent() {
        Some(p) => p.to_path_buf(),
        None => {
            eprintln!("Error: Failed to get executable directory");
            wait_and_exit(1);
            return;
        }
    };

    let game_exe_path = exe_dir.join(GAME_EXE_NAME);
    let temp_dir = exe_dir.join("_update_temp");
    let archive_path = temp_dir.join("update.archive");

    // Create temp directory
    if let Err(e) = fs::create_dir_all(&temp_dir) {
        eprintln!("Error: Failed to create temp directory: {}", e);
        wait_and_exit(1);
        return;
    }

    // Download the update
    println!("Downloading update...");
    if let Err(e) = download_file(download_url, &archive_path) {
        eprintln!("Error: Download failed: {}", e);
        cleanup(&temp_dir);
        wait_and_exit(1);
        return;
    }
    println!("Download complete!");

    // Extract the archive
    println!("Extracting update...");
    if let Err(e) = extract_archive(&archive_path, &temp_dir) {
        eprintln!("Error: Extraction failed: {}", e);
        cleanup(&temp_dir);
        wait_and_exit(1);
        return;
    }
    println!("Extraction complete!");

    // Find and copy the new game executable
    println!("Installing update...");
    if let Err(e) = install_update(&temp_dir, &game_exe_path) {
        eprintln!("Error: Installation failed: {}", e);
        cleanup(&temp_dir);
        wait_and_exit(1);
        return;
    }
    println!("Installation complete!");

    // Cleanup
    cleanup(&temp_dir);

    // Restart the game
    println!();
    println!("Update successful! Starting game...");
    thread::sleep(Duration::from_secs(1));

    if let Err(e) = Command::new(&game_exe_path).spawn() {
        eprintln!("Warning: Failed to restart game: {}", e);
        eprintln!("Please start the game manually.");
        wait_and_exit(0);
    }
}

fn download_file(url: &str, dest: &Path) -> Result<(), String> {
    // Use ureq for HTTP requests
    let response = ureq::get(url)
        .set("User-Agent", "IdleFactoryUpdater/1.0")
        .call()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let total_size = response
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let mut file = File::create(dest).map_err(|e| format!("Failed to create file: {}", e))?;

    let mut reader = response.into_reader();
    let mut buffer = [0u8; 8192];
    let mut downloaded: u64 = 0;
    let mut last_percent = 0;

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read: {}", e))?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(|e| format!("Failed to write: {}", e))?;

        downloaded += bytes_read as u64;

        // Progress indicator
        if total_size > 0 {
            let percent = (downloaded * 100 / total_size) as u32;
            if percent != last_percent && percent.is_multiple_of(10) {
                println!("  {}%", percent);
                last_percent = percent;
            }
        }
    }

    Ok(())
}

fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;

    let filename = archive_path.to_string_lossy();

    if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
        // Extract tar.gz
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        archive
            .unpack(dest_dir)
            .map_err(|e| format!("Failed to extract tar.gz: {}", e))?;
    } else if filename.ends_with(".zip") {
        // Extract zip
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("Failed to open zip: {}", e))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read zip entry: {}", e))?;

            let outpath = dest_dir.join(file.name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                }
                let mut outfile =
                    File::create(&outpath).map_err(|e| format!("Failed to create file: {}", e))?;
                io::copy(&mut file, &mut outfile)
                    .map_err(|e| format!("Failed to write file: {}", e))?;
            }

            // Set permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).ok();
                }
            }
        }
    } else {
        return Err("Unknown archive format".to_string());
    }

    Ok(())
}

fn install_update(temp_dir: &Path, game_exe_path: &Path) -> Result<(), String> {
    // Find the game executable in the extracted files
    let new_exe = find_game_exe(temp_dir)?;

    // Backup old executable (optional)
    let backup_path = game_exe_path.with_extension("exe.bak");
    if game_exe_path.exists() {
        // On Windows, we might need to rename instead of delete
        if backup_path.exists() {
            fs::remove_file(&backup_path).ok();
        }
        fs::rename(game_exe_path, &backup_path)
            .map_err(|e| format!("Failed to backup old executable: {}", e))?;
    }

    // Copy new executable
    fs::copy(&new_exe, game_exe_path)
        .map_err(|e| format!("Failed to copy new executable: {}", e))?;

    // Set executable permission on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(game_exe_path, fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    // Also copy assets directory if it exists
    let assets_src = find_assets_dir(temp_dir);
    if let Some(src) = assets_src {
        let assets_dest = game_exe_path.parent().unwrap().join("assets");
        copy_dir_recursive(&src, &assets_dest)?;
    }

    // Remove backup (cleanup)
    fs::remove_file(&backup_path).ok();

    Ok(())
}

fn find_game_exe(dir: &Path) -> Result<PathBuf, String> {
    // Look for idle_factory executable recursively
    fn search(dir: &Path) -> Option<PathBuf> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name() {
                        let name = name.to_string_lossy();
                        if name == GAME_EXE_NAME {
                            return Some(path);
                        }
                    }
                } else if path.is_dir() {
                    if let Some(found) = search(&path) {
                        return Some(found);
                    }
                }
            }
        }
        None
    }

    search(dir).ok_or_else(|| "Game executable not found in archive".to_string())
}

fn find_assets_dir(dir: &Path) -> Option<PathBuf> {
    fn search(dir: &Path) -> Option<PathBuf> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        if name == "assets" {
                            return Some(path);
                        }
                    }
                    if let Some(found) = search(&path) {
                        return Some(found);
                    }
                }
            }
        }
        None
    }

    search(dir)
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    if !dest.exists() {
        fs::create_dir_all(dest).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    if let Ok(entries) = fs::read_dir(src) {
        for entry in entries.flatten() {
            let path = entry.path();
            let dest_path = dest.join(entry.file_name());

            if path.is_dir() {
                copy_dir_recursive(&path, &dest_path)?;
            } else {
                fs::copy(&path, &dest_path).map_err(|e| format!("Failed to copy file: {}", e))?;
            }
        }
    }

    Ok(())
}

fn cleanup(temp_dir: &Path) {
    if temp_dir.exists() {
        fs::remove_dir_all(temp_dir).ok();
    }
}

fn wait_and_exit(code: i32) {
    println!();
    println!("Press Enter to close...");
    let _ = io::stdin().read_line(&mut String::new());
    std::process::exit(code);
}
