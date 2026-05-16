use std::path::{Path, PathBuf};

use anyhow::Context;
use tracing::info;

/// Default INI content — minimal valid files so FO4 doesn't complain.
const DEFAULT_FALLOUT4_INI: &str = "[General]\n";
const DEFAULT_PREFS_INI: &str = "[Display]\n";
const DEFAULT_CUSTOM_INI: &str = "[General]\n";

/// Materialise the on-disk directory structure for a profile if it
/// doesn't already exist.
pub async fn ensure_profile_dir(data_dir: &Path, profile_id: i64) -> anyhow::Result<PathBuf> {
    let profile_dir = data_dir.join("profiles").join(profile_id.to_string());

    tokio::fs::create_dir_all(&profile_dir)
        .await
        .context("failed to create profile directory")?;

    // Write default files only if they don't exist yet
    write_if_absent(&profile_dir.join("plugins.txt"), "").await?;
    write_if_absent(&profile_dir.join("loadorder.txt"), "").await?;
    write_if_absent(&profile_dir.join("fallout4.ini"), DEFAULT_FALLOUT4_INI).await?;
    write_if_absent(&profile_dir.join("fallout4prefs.ini"), DEFAULT_PREFS_INI).await?;
    write_if_absent(&profile_dir.join("fallout4custom.ini"), DEFAULT_CUSTOM_INI).await?;

    info!(profile_id, path = %profile_dir.display(), "profile directory ready");
    Ok(profile_dir)
}

/// Return the path to a profile's directory without creating it.
pub fn profile_dir(data_dir: &Path, profile_id: i64) -> PathBuf {
    data_dir.join("profiles").join(profile_id.to_string())
}

async fn write_if_absent(path: &Path, content: &str) -> anyhow::Result<()> {
    if !path.exists() {
        tokio::fs::write(path, content)
            .await
            .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(())
}
