use serde::{Deserialize, Serialize};

// ── Game ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id:           i64,
    pub name:         String,
    pub canonical_id: String,   // "fallout4"
}

// ── Instance ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id:           i64,
    pub game_id:      i64,
    pub label:        String,
    pub source_type:  SourceType,
    pub install_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Steam,
    Gog,
    Manual,
}

// ── Profile ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id:               i64,
    pub instance_id:      i64,
    pub name:             String,
    pub pinned_runner_id: Option<i64>,
    pub auto_deploy:      bool,
}

// ── Mod ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mod {
    pub id:           i64,
    pub name:         String,
    pub version:      Option<String>,
    pub source_hash:  String,   // SHA-256 of the original archive
    pub install_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMod {
    pub profile_id: i64,
    pub mod_id:     i64,
    pub enabled:    bool,
    pub priority:   i32,    // lower = lower priority; higher wins conflicts
}

// ── Plugin ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id:       i64,
    pub mod_id:   Option<i64>,  // None = loose / unmanaged plugin
    pub filename: String,
    pub kind:     PluginKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginKind {
    Esm,    // master file
    Esp,    // regular plugin
    Esl,    // light plugin
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilePlugin {
    pub profile_id: i64,
    pub plugin_id:  i64,
    pub enabled:    bool,
    pub load_index: i32,
}

// ── Runner ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runner {
    pub id:           i64,
    pub kind:         RunnerKind,
    pub version:      String,
    pub source_url:   Option<String>,
    pub install_path: String,
    pub verified:     bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RunnerKind {
    Proton,
    Wine,
}

// ── Deploy manifest ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployManifest {
    pub id:           i64,
    pub profile_id:   i64,
    pub symlink_plan: Vec<SymlinkEntry>,    // decoded from the JSON blob
    pub status:       DeployStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkEntry {
    pub source: String,     // absolute path inside mod's install dir
    pub target: String,     // absolute path inside game's Data dir
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeployStatus {
    Active,
    RolledBack,
}