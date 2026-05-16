use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("profile '{name}' already exists for this instance")]
    ProfileAlreadyExists { name: String },

    #[error("priority {priority} is already taken in profile {profile_id}")]
    PriorityConflict { profile_id: i64, priority: i32 },

    #[error("runner is not verified and cannot be pinned")]
    RunnerNotVerified,

    #[error("mod '{name}' has no priority assigned in this profile")]
    ModNotInProfile { name: String },

    #[error("load index {index} is already taken in profile {profile_id}")]
    LoadIndexConflict { profile_id: i64, index: i32 },

    #[error("install path does not exist or is not a directory: {path}")]
    InvalidInstallPath { path: String },
}
