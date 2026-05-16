use anyhow::Context;
use std::path::Path;
use tracing::{info, warn};

use domain_core::entities::SymlinkEntry;
use storage_sqlite::Db;

use crate::plan::DeployPlan;

/// Apply a deploy plan: create symlinks and write a manifest for rollback.
pub async fn apply_plan(plan: DeployPlan, db: &Db) -> anyhow::Result<i64> {
    let symlink_json =
        serde_json::to_string(&plan.entries).context("failed to serialise symlink plan")?;

    // Write manifest first — if symlinking fails we still have a record
    let manifest_id = sqlx::query!(
        r#"
        INSERT INTO deploy_manifests (profile_id, symlink_plan, status)
        VALUES (?1, ?2, 'active')
        "#,
        plan.profile_id,
        symlink_json,
    )
    .execute(&db.pool)
    .await
    .context("failed to write deploy manifest")?
    .last_insert_rowid();

    // Create symlinks
    let mut created = 0usize;
    let mut skipped = 0usize;

    for entry in &plan.entries {
        let source = Path::new(&entry.source);
        let target = Path::new(&entry.target);

        if !source.exists() {
            warn!(source = %entry.source, "source file missing, skipping");
            skipped += 1;
            continue;
        }

        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("failed to create dir {}", parent.display()))?;
        }

        // Remove stale symlink if present
        if target.exists() || target.symlink_metadata().is_ok() {
            tokio::fs::remove_file(target).await.ok();
        }

        tokio::fs::symlink(&entry.source, target)
            .await
            .with_context(|| format!("failed to symlink {} -> {}", entry.source, entry.target))?;

        created += 1;
    }

    info!(
        profile_id = plan.profile_id,
        manifest_id, created, skipped, "deploy complete"
    );

    Ok(manifest_id)
}

/// Undo a deployment by removing all symlinks in a manifest.
pub async fn rollback(manifest_id: i64, db: &Db) -> anyhow::Result<()> {
    let row = sqlx::query!(
        "SELECT symlink_plan FROM deploy_manifests WHERE id = ?1",
        manifest_id
    )
    .fetch_one(&db.pool)
    .await
    .context("manifest not found")?;

    let entries: Vec<SymlinkEntry> =
        serde_json::from_str(&row.symlink_plan).context("failed to parse manifest")?;

    for entry in &entries {
        let target = Path::new(&entry.target);
        if target.symlink_metadata().is_ok() {
            tokio::fs::remove_file(target).await.ok();
        }
    }

    sqlx::query!(
        "UPDATE deploy_manifests SET status = 'rolled_back' WHERE id = ?1",
        manifest_id
    )
    .execute(&db.pool)
    .await
    .context("failed to update manifest status")?;

    info!(manifest_id, "rollback complete");
    Ok(())
}
