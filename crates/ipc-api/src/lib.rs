use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "method", content = "params", rename_all = "snake_case")]
pub enum Request {
    Ping,
    DetectGame,
    RegisterGame {
        path: String,
    },
    ListRunners,
    PinRunner {
        profile_id: i64,
        runner_id: i64,
    },
    ListProfilePlugins {
            profile_id: i64,
        },
        SetProfilePluginEnabled {
            profile_id: i64,
            plugin_id: i64,
            enabled: bool,
        },
    ListProfileMods {
        profile_id: i64,
    },
    UpsertProfileMod {
        profile_id: i64,
        mod_id: i64,
        enabled: bool,
        priority: i32,
    },
    SetProfileModEnabled {
        profile_id: i64,
        mod_id: i64,
        enabled: bool,
    },
    SetProfileModPriority {
        profile_id: i64,
        mod_id: i64,
        priority: i32,
    },
    CreateProfile {
        instance_id: i64,
        name: String,
    },
    ListProfiles {
        instance_id: i64,
    },
    DeleteProfile {
        profile_id: i64,
    },
    IngestMod {
        archive_path: String,
    },
    DeployPreview {
        profile_id: i64,
        game_data_dir: String,
    },
    DeployApply {
        profile_id: i64,
        game_data_dir: String,
    },
    DeployRollback {
        manifest_id: i64,
    },
    SyncPlugins {
        profile_id: i64,
        data_dir: String,
    },
    ValidatePlugins {
        profile_id: i64,
    },
    SortWithLoot {
        profile_id: i64,
    },
    LaunchPreflight {
        profile_id: i64,
        use_f4se: bool,
    },
    LaunchGame {
        profile_id: i64,
        use_f4se: bool,
    },
    LaunchLauncher {
        profile_id: i64,
    },
    WriteLoadOrder {
        profile_id: i64,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum Response {
    Pong {
        version: String,
    },
    GameDetected {
        instance_id: i64,
        install_path: String,
        source: String,
    },
    RunnerList {
        runners: Vec<RunnerInfo>,
    },
    RunnerPinned {
        profile_id: i64,
        runner_id: i64,
    },
    ProfilePlugins {
            profile_id: i64,
            plugins: Vec<ProfilePluginInfo>,
        },
        ProfilePluginUpdated {
            profile_id: i64,
            plugin_id: i64,
        },
    ProfileMods {
        profile_id: i64,
        mods: Vec<ProfileModInfo>,
    },
    ProfileModUpdated {
        profile_id: i64,
        mod_id: i64,
    },
    ProfileCreated {
        profile_id: i64,
    },
    ProfileList {
        profiles: Vec<ProfileInfo>,
    },
    ProfileDeleted {
        profile_id: i64,
    },
    ModIngested {
        mod_id: i64,
        name: String,
        file_count: usize,
    },
    DeployPreview {
        profile_id: i64,
        entry_count: usize,
        entries: Vec<String>,
    },
    DeployApplied {
        manifest_id: i64,
    },
    RolledBack {
        manifest_id: i64,
    },
    PluginsSynced {
        added: usize,
    },
    PluginsValid {
        missing_masters: Vec<String>,
    },
    PluginsSorted {
        profile_id: i64,
        engine: String,
        order: Vec<String>,
    },
    LaunchPreflight {
        profile_id: i64,
        runner_kind: String,
        game_install_path: String,
        f4se_available: bool,
    },
    GameLaunched {
        profile_id: i64,
        runner_kind: String,
        executable: String,
        pid: u32,
    },
    LoadOrderWritten,
    Ok,
    Error {
        code: ErrorCode,
        message: String,
    },
}

/// Lightweight profile summary sent over IPC (no need for the full entity).
#[derive(Debug, Deserialize, Serialize)]
pub struct ProfileInfo {
    pub id: i64,
    pub name: String,
    pub auto_deploy: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RunnerInfo {
    pub id: i64,
    pub kind: String,
    pub version: String,
    pub install_path: String,
    pub verified: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProfileModInfo {
    pub mod_id: i64,
    pub mod_name: String,
    pub enabled: bool,
    pub priority: i32,
}

#[derive(Debug, Deserialize, Serialize, thiserror::Error)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    #[error("unknown method")]
    UnknownMethod,
    #[error("invalid request payload")]
    InvalidRequest,
    #[error("internal daemon error")]
    Internal,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProfilePluginInfo {
    pub plugin_id: i64,
    pub filename: String,
    pub kind: String,
    pub enabled: bool,
    pub load_index: i32,
}