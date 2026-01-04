//! Version checking logic using GitHub Releases API.
//!
//! Uses ureq for lightweight HTTP requests instead of self_update.

use super::state::UpdateCheckResult;

/// GitHub repository owner.
const REPO_OWNER: &str = "GS-Bacon";
/// GitHub repository name.
const REPO_NAME: &str = "idle_factory";

/// Current application version (from Cargo.toml at compile time).
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub API URL for latest release (uses 'latest' tag, not the "Latest" marked release)
fn get_releases_url() -> String {
    format!(
        "https://api.github.com/repos/{}/{}/releases/tags/latest",
        REPO_OWNER, REPO_NAME
    )
}

/// Check for updates using GitHub API directly.
///
/// This function is blocking and should be called from a background task.
pub fn check_for_update() -> UpdateCheckResult {
    tracing::info!("Checking for updates... (current: v{})", CURRENT_VERSION);

    // Call GitHub API
    let response = match ureq::get(&get_releases_url())
        .set("User-Agent", "IdleFactory/1.0")
        .set("Accept", "application/vnd.github.v3+json")
        .call()
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Failed to fetch latest release: {}", e);
            return UpdateCheckResult::Error(format!("Network error: {}", e));
        }
    };

    // Parse JSON response
    let body = match response.into_string() {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("Failed to read response: {}", e);
            return UpdateCheckResult::Error(format!("Read error: {}", e));
        }
    };

    let json: serde_json::Value = match serde_json::from_str(&body) {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("Failed to parse release info: {}", e);
            return UpdateCheckResult::Error(format!("Parse error: {}", e));
        }
    };

    // Extract version from asset name (e.g., "idle_factory_0.2.3_linux.tar.gz" -> "0.2.3")
    // We use asset names because the 'latest' tag doesn't contain version info
    let latest_version = match extract_version_from_assets(&json) {
        Some(v) => v,
        None => {
            tracing::warn!("Could not extract version from assets");
            return UpdateCheckResult::Error("No valid assets found".to_string());
        }
    };
    tracing::info!("Latest version: v{}", latest_version);

    // Compare versions
    let current = match semver::Version::parse(CURRENT_VERSION) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Invalid current version format: {}", e);
            return UpdateCheckResult::Error(format!("Invalid version: {}", e));
        }
    };

    let latest_semver = match semver::Version::parse(&latest_version) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Invalid latest version format: {}", e);
            return UpdateCheckResult::Error(format!("Invalid remote version: {}", e));
        }
    };

    if latest_semver > current {
        tracing::info!("Update available: v{} -> v{}", current, latest_semver);

        // Get download URL for current platform
        let download_url = get_platform_download_url(&json);
        let release_notes = json["body"].as_str().unwrap_or("").to_string();

        UpdateCheckResult::Available {
            version: latest_version.to_string(),
            release_notes,
            download_url,
        }
    } else {
        tracing::info!("Already up to date (v{})", current);
        UpdateCheckResult::UpToDate
    }
}

/// Get the download URL for the current platform.
fn get_platform_download_url(release_json: &serde_json::Value) -> String {
    let platform_suffix = if cfg!(target_os = "linux") {
        "_linux.tar.gz"
    } else if cfg!(target_os = "windows") {
        "_windows.zip"
    } else {
        return String::new();
    };

    // Find matching asset
    if let Some(assets) = release_json["assets"].as_array() {
        for asset in assets {
            if let Some(name) = asset["name"].as_str() {
                if name.ends_with(platform_suffix) {
                    if let Some(url) = asset["browser_download_url"].as_str() {
                        return url.to_string();
                    }
                }
            }
        }
    }

    // Fallback: return release page URL
    let tag = release_json["tag_name"].as_str().unwrap_or("latest");
    format!(
        "https://github.com/{}/{}/releases/tag/{}",
        REPO_OWNER, REPO_NAME, tag
    )
}

/// Extract the highest version from asset names.
/// Looks for patterns like "idle_factory_X.Y.Z_platform.ext"
fn extract_version_from_assets(release_json: &serde_json::Value) -> Option<String> {
    let assets = release_json["assets"].as_array()?;

    let mut highest_version: Option<semver::Version> = None;

    for asset in assets {
        if let Some(name) = asset["name"].as_str() {
            // Match pattern: idle_factory_X.Y.Z_
            if let Some(start) = name.strip_prefix("idle_factory_") {
                if let Some(version_end) = start.find('_') {
                    let version_str = &start[..version_end];
                    if let Ok(version) = semver::Version::parse(version_str) {
                        if highest_version.as_ref().is_none_or(|h| version > *h) {
                            highest_version = Some(version);
                        }
                    }
                }
            }
        }
    }

    highest_version.map(|v| v.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_version_is_valid() {
        let version = semver::Version::parse(CURRENT_VERSION);
        assert!(version.is_ok(), "CURRENT_VERSION should be valid semver");
    }
}
