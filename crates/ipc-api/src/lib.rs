use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "method", content = "params", rename_all = "snake_case")]
pub enum Request {
    Ping,
    DetectGame,
    RegisterGame   { path: String },
    CreateProfile  { instance_id: i64, name: String },
    ListProfiles   { instance_id: i64 },
    DeleteProfile  { profile_id: i64 },
    IngestMod      { archive_path: String },     // ← new
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum Response {
    Pong           { version: String },
    GameDetected   { install_path: String, source: String },
    ProfileCreated { profile_id: i64 },
    ProfileList    { profiles: Vec<ProfileInfo> },
    ProfileDeleted { profile_id: i64 },
    ModIngested    { mod_id: i64, name: String, file_count: usize },   // ← new
    Ok,
    Error          { code: ErrorCode, message: String },
}

/// Lightweight profile summary sent over IPC (no need for the full entity).
#[derive(Debug, Deserialize, Serialize)]
pub struct ProfileInfo {
    pub id:          i64,
    pub name:        String,
    pub auto_deploy: bool,
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