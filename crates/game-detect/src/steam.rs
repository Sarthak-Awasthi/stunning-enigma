use std::path::{Path, PathBuf};

use tracing::{debug, info, warn};

use crate::DetectError;

// Steam app ID for Fallout 4
const FO4_APP_ID: &str = "377160";

pub struct SteamDetector;

impl SteamDetector {
    /// Try to find the Fallout 4 install directory via Steam's VDF manifests.
    pub fn detect() -> Result<PathBuf, DetectError> {
        let steam_root = Self::find_steam_root()?;
        info!(path = %steam_root.display(), "found Steam root");

        let libraries = Self::find_library_folders(&steam_root)?;
        debug!(count = libraries.len(), "Steam library folders found");

        for lib in &libraries {
            debug!(path = %lib.display(), "scanning library");
            match Self::find_fo4_in_library(lib) {
                Some(fo4_path) => {
                    info!(path = %fo4_path.display(), "Fallout 4 found");
                    return Ok(fo4_path);
                }
                None => continue,
            }
        }

        Err(DetectError::Fo4NotFound)
    }

    /// Locate the Steam root directory (~/.steam/steam or ~/.local/share/Steam).
    fn find_steam_root() -> Result<PathBuf, DetectError> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());

        let candidates = [
            PathBuf::from(&home).join(".steam/steam"),
            PathBuf::from(&home).join(".local/share/Steam"),
            PathBuf::from("/usr/share/steam"),
        ];

        for path in &candidates {
            if path.join("steamapps").exists() {
                return Ok(path.clone());
            }
        }

        Err(DetectError::SteamNotFound)
    }

    /// Parse libraryfolders.vdf to find all Steam library roots.
    fn find_library_folders(steam_root: &Path) -> Result<Vec<PathBuf>, DetectError> {
        let vdf_path = steam_root.join("steamapps/libraryfolders.vdf");

        // The steam root itself is always a library
        let mut libraries = vec![steam_root.join("steamapps")];

        if !vdf_path.exists() {
            warn!("libraryfolders.vdf not found, using Steam root only");
            return Ok(libraries);
        }

        let content = std::fs::read_to_string(&vdf_path)?;

        // Parse the VDF and extract "path" fields
        use keyvalues_parser::Vdf;
        if let Ok(vdf) = Vdf::parse(&content) {
            if let Some(obj) = vdf.value.get_obj() {
                for (_key, values) in obj.iter() {
                    for value in values {
                        if let Some(inner) = value.get_obj() {
                            if let Some(path_vals) = inner.get("path") {
                                if let Some(path_str) = path_vals
                                    .first()
                                    .and_then(|v| v.get_str())
                                {
                                    let lib = PathBuf::from(path_str).join("steamapps");
                                    if lib.exists() {
                                        libraries.push(lib);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(libraries)
    }

    /// Look for Fallout 4's appmanifest inside a steamapps folder.
    fn find_fo4_in_library(steamapps: &Path) -> Option<PathBuf> {
        let manifest = steamapps.join(format!("appmanifest_{FO4_APP_ID}.acf"));
        if !manifest.exists() {
            return None;
        }

        // The game lives in steamapps/common/<InstallDir>
        // Parse install dir from manifest to be safe
        let content = std::fs::read_to_string(&manifest).ok()?;
        let install_dir = Self::extract_install_dir(&content)?;

        let game_path = steamapps.join("common").join(install_dir);
        if game_path.exists() {
            Some(game_path)
        } else {
            None
        }
    }

    /// Extract the "installdir" value from an ACF manifest.
    fn extract_install_dir(content: &str) -> Option<String> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("\"installdir\"") {
                // line looks like: "installdir"		"Fallout 4"
                let parts: Vec<&str> = line.splitn(2, '\t').collect();
                if let Some(val) = parts.last() {
                    return Some(val.trim().trim_matches('"').to_string());
                }
            }
        }
        None
    }
}