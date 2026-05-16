mod steam;
mod validate;

pub use steam::SteamDetector;
pub use validate::validate_fo4_path;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DetectError {
    #[error("Steam not found at expected paths")]
    SteamNotFound,

    #[error("Fallout 4 not found in any Steam library")]
    Fo4NotFound,

    #[error("path is not a valid Fallout 4 install: {reason}")]
    InvalidPath { reason: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}