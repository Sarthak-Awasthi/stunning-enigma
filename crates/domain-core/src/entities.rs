//! Core domain entities for the Mod Manager.
//!
//! This module defines all the primary data structures used to represent
//! games, instances, profiles, mods, plugins, runners, and deployment state.

use serde::{Deserialize, Serialize};

// ── Game ────────────────────────────────────────────────────────────────────

/// A supported game (e.g., Fallout 4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: i64,
    pub name: String,
    pub canonical_id: String, // "fallout4"
}

// ── Instance ─────────────────────────────────────────────────────────────────

/// A specific installation of a game on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: i64,
    pub game_id: i64,
    pub label: String,
    pub source_type: SourceType,
    pub install_path: String,
}

/// The source from which a game instance was detected or registered.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Steam,
    Gog,
    Manual,
}

// ── Profile ──────────────────────────────────────────────────────────────────

/// A profile represents an isolated mod/plugin configuration environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: i64,
    pub instance_id: i64,
    pub name: String,
    pub pinned_runner_id: Option<i64>,
    pub auto_deploy: bool,
}

// ── Mod ───────────────────────────────────────────────────────────────────────

/// A mod archive that has been ingested into the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mod {
    pub id: i64,
    pub name: String,
    pub version: Option<String>,
    pub source_hash: String, // SHA-256 of the original archive
    pub install_path: String,
}

/// A mod's association with a profile, including enablement and priority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMod {
    pub profile_id: i64,
    pub mod_id: i64,
    pub enabled: bool,
    pub priority: i32, // lower = lower priority; higher wins conflicts
}

// ── Plugin ───────────────────────────────────────────────────────────────────

/// A plugin file (ESM, ESP, or ESL) extracted from a mod or loose in Data/.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: i64,
    pub mod_id: Option<i64>, // None = loose / unmanaged plugin
    pub filename: String,
    pub kind: PluginKind,
}

/// The type of a plugin file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginKind {
    Esm, // master file
    Esp, // regular plugin
    Esl, // light plugin
}

/// A plugin's association with a profile, including enablement and load order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilePlugin {
    pub profile_id: i64,
    pub plugin_id: i64,
    pub enabled: bool,
    pub load_index: i32,
}

// ── Runner ───────────────────────────────────────────────────────────────────

/// A compatibility layer runner (Proton or Wine) for launching the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runner {
    pub id: i64,
    pub kind: RunnerKind,
    pub version: String,
    pub source_url: Option<String>,
    pub install_path: String,
    pub verified: bool,
}

/// The type of runner.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RunnerKind {
    Proton,
    Wine,
}

// ── Deploy manifest ──────────────────────────────────────────────────────────

/// A record of a deployment operation, including all symlinks created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployManifest {
    pub id: i64,
    pub profile_id: i64,
    pub symlink_plan: Vec<SymlinkEntry>, // decoded from the JSON blob
    pub status: DeployStatus,
}

/// A single symlink entry in a deployment plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkEntry {
    pub source: String, // absolute path inside mod's install dir
    pub target: String, // absolute path inside game's Data dir
}

/// The status of a deployment manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeployStatus {
    Active,
    RolledBack,
}
