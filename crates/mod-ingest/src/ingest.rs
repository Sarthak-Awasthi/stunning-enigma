use std::path::Path;

use anyhow::Context;
use tracing::info;

use storage_sqlite::Db;

use crate::{archive, fomod, hasher};

#[derive(Debug)]
pub struct IngestResult {
    pub mod_id: i64,
    pub name: String,
    pub source_hash: String,
    pub file_count: usize,
}

/// Full pipeline: hash → dedup check → extract → index → record in DB.
pub async fn ingest_mod(
    archive_path: &Path,
    mods_dir: &Path,
    db: &Db,
) -> anyhow::Result<IngestResult> {
    // 1. Hash the archive for deduplication
    let hash = hasher::sha256_file(archive_path).await?;
    info!(hash = %hash, "archive hashed");

    // 2. Check if already installed
    let existing = sqlx::query!("SELECT id, name FROM mods WHERE source_hash = ?1", hash)
        .fetch_optional(&db.pool)
        .await
        .context("dedup check failed")?;

    if let Some(row) = existing {
        info!(
            mod_id = row.id,
            "mod already installed, skipping extraction"
        );
        let file_count =
            sqlx::query_scalar!("SELECT COUNT(*) FROM file_index WHERE mod_id = ?1", row.id)
                .fetch_one(&db.pool)
                .await? as usize;

        return Ok(IngestResult {
            mod_id: row.id.expect("id is always set for persisted rows"),
            name: row.name,
            source_hash: hash,
            file_count,
        });
    }

    // 3. Derive mod name from archive filename
    let name = archive_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown_mod")
        .to_string();

    // 4. Extract into mods/<hash>/
    let install_path = mods_dir.join(&hash);
    let install_path_str = install_path.display().to_string();

    info!(dest = %install_path.display(), "extracting archive");
    let files = tokio::task::spawn_blocking({
        let ap = archive_path.to_path_buf();
        let ip = install_path.clone();
        move || archive::extract(&ap, &ip)
    })
    .await
    .context("extraction task panicked")?
    .context("extraction failed")?;

    let files = tokio::task::spawn_blocking({
        let ip = install_path.clone();
        move || fomod::apply_if_present(&ip, files)
    })
    .await
    .context("FOMOD processing task panicked")?
    .context("FOMOD processing failed")?;

    // 5. Record mod in DB
    let mod_id = sqlx::query!(
        r#"
        INSERT INTO mods (name, source_hash, install_path)
        VALUES (?1, ?2, ?3)
        "#,
        name,
        hash,
        install_path_str,
    )
    .execute(&db.pool)
    .await
    .context("failed to record mod")?
    .last_insert_rowid();

    // 6. Index all extracted files
    for rel in &files {
        let rel_str = rel.display().to_string();
        // Normalise path separator to forward slash for consistency
        let rel_str = rel_str.replace('\\', "/");
        let is_ba2 = rel_str.to_lowercase().ends_with(".ba2") as i64;

        sqlx::query!(
            "INSERT OR IGNORE INTO file_index (mod_id, rel_path, is_ba2) VALUES (?1, ?2, ?3)",
            mod_id,
            rel_str,
            is_ba2,
        )
        .execute(&db.pool)
        .await
        .context("failed to index file")?;
    }

    info!(mod_id, name = %name, files = files.len(), "mod ingested");

    Ok(IngestResult {
        mod_id,
        name,
        source_hash: hash,
        file_count: files.len(),
    })
}
