use std::path::Path;
use anyhow::Context;
use sha2::{Digest, Sha256};

/// Compute SHA-256 of a file and return it as a lowercase hex string.
pub async fn sha256_file(path: &Path) -> anyhow::Result<String> {
    let bytes = tokio::fs::read(path)
        .await
        .with_context(|| format!("failed to read {} for hashing", path.display()))?;

    let hash = Sha256::digest(&bytes);
    Ok(hex::encode(hash))
}