//! Version checking logic using GitHub Releases API.

use super::state::UpdateCheckResult;

/// GitHub repository owner.
const REPO_OWNER: &str = "GS-Bacon";
/// GitHub repository name.
const REPO_NAME: &str = "idle_factory";
/// Binary name for update detection.
const BIN_NAME: &str = "idle_factory";

/// Current application version (from Cargo.toml at compile time).
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check for updates using self_update crate.
///
/// This function is blocking and should be called from a background task.
pub fn check_for_update() -> UpdateCheckResult {
    use self_update::backends::github::Update;

    tracing::info!("Checking for updates... (current: v{})", CURRENT_VERSION);

    let updater = match Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .current_version(CURRENT_VERSION)
        .build()
    {
        Ok(u) => u,
        Err(e) => {
            tracing::warn!("Failed to configure updater: {}", e);
            return UpdateCheckResult::Error(format!("Configuration error: {}", e));
        }
    };

    // Get latest release info
    let latest = match updater.get_latest_release() {
        Ok(release) => release,
        Err(e) => {
            tracing::warn!("Failed to fetch latest release: {}", e);
            return UpdateCheckResult::Error(format!("Network error: {}", e));
        }
    };

    let latest_version = &latest.version;
    tracing::info!("Latest version: v{}", latest_version);

    // Compare versions
    let current = match semver::Version::parse(CURRENT_VERSION) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Invalid current version format: {}", e);
            return UpdateCheckResult::Error(format!("Invalid version: {}", e));
        }
    };

    let latest_semver = match semver::Version::parse(latest_version) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Invalid latest version format: {}", e);
            return UpdateCheckResult::Error(format!("Invalid remote version: {}", e));
        }
    };

    if latest_semver > current {
        tracing::info!("Update available: v{} -> v{}", current, latest_semver);

        // Get download URL for current platform
        let download_url = get_platform_download_url(&latest);

        UpdateCheckResult::Available {
            version: latest_version.to_string(),
            release_notes: latest.body.clone().unwrap_or_default(),
            download_url,
        }
    } else {
        tracing::info!("Already up to date (v{})", current);
        UpdateCheckResult::UpToDate
    }
}

/// Get the download URL for the current platform.
fn get_platform_download_url(release: &self_update::update::Release) -> String {
    let platform_suffix = if cfg!(target_os = "linux") {
        "_linux.tar.gz"
    } else if cfg!(target_os = "windows") {
        "_windows.zip"
    } else {
        return String::new();
    };

    // Find matching asset
    for asset in &release.assets {
        if asset.name.ends_with(platform_suffix) {
            return asset.download_url.clone();
        }
    }

    // Fallback: return release page URL
    format!(
        "https://github.com/{}/{}/releases/tag/{}",
        REPO_OWNER, REPO_NAME, &release.version
    )
}

/// Perform the actual update (download and install).
///
/// This function is blocking and should be called from a background task.
/// Returns Ok(()) on success, or an error message on failure.
pub fn perform_update() -> Result<(), String> {
    use self_update::backends::github::Update;

    tracing::info!("Starting update process...");

    let status = Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .current_version(CURRENT_VERSION)
        .show_download_progress(true)
        .no_confirm(true) // Don't prompt, we handle UI ourselves
        .build()
        .map_err(|e| format!("Configuration error: {}", e))?
        .update()
        .map_err(|e| format!("Update failed: {}", e))?;

    tracing::info!("Update status: {:?}", status);

    match status {
        self_update::Status::UpToDate(_) => {
            tracing::info!("Already up to date");
            Ok(())
        }
        self_update::Status::Updated(version) => {
            tracing::info!("Updated to v{}", version);
            Ok(())
        }
    }
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
