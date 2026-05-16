use anyhow::Context;
use std::path::Path;
use tracing::info;

use domain_core::entities::SymlinkEntry;
use storage_sqlite::Db;

use crate::conflict::resolve;

#[derive(Debug)]
pub struct DeployPlan {
    pub profile_id: i64,
    pub entries: Vec<SymlinkEntry>,
}

/// Build a deploy plan for a profile: query enabled mods + file index,
/// resolve conflicts, and produce a list of symlink operations.
pub async fn build_plan(
    profile_id: i64,
    game_data_dir: &Path, // e.g. /path/to/Fallout 4/Data
    db: &Db,
) -> anyhow::Result<DeployPlan> {
    // 1. Load enabled mods for this profile, ordered by priority
    let mod_rows = sqlx::query!(
        r#"
        SELECT m.id, pm.priority, m.install_path
        FROM profile_mods pm
        JOIN mods m ON m.id = pm.mod_id
        WHERE pm.profile_id = ?1 AND pm.enabled = 1
        ORDER BY pm.priority ASC
        "#,
        profile_id,
    )
    .fetch_all(&db.pool)
    .await
    .context("failed to load profile mods")?;

    let mods: Vec<(i64, i32, String)> = mod_rows
        .into_iter()
        .map(|r| (r.id.expect("id set"), r.priority as i32, r.install_path))
        .collect();

    // 2. Load full file index for those mods
    let mod_ids: Vec<i64> = mods.iter().map(|(id, _, _)| *id).collect();

    let mut file_index: Vec<(i64, String)> = Vec::new();
    for mod_id in &mod_ids {
        let files = sqlx::query!(
            "SELECT mod_id, rel_path FROM file_index WHERE mod_id = ?1",
            mod_id
        )
        .fetch_all(&db.pool)
        .await?;
        for f in files {
            file_index.push((f.mod_id, f.rel_path));
        }
    }

    // 3. Resolve conflicts
    let winners = resolve(&mods, &file_index);
    info!(
        profile_id,
        winners = winners.len(),
        "conflict resolution complete"
    );

    // 4. Build symlink entries
    let entries = winners
        .into_iter()
        .map(|w| SymlinkEntry {
            source: w.source_path,
            target: game_data_dir.join(&w.rel_path).display().to_string(),
        })
        .collect();

    Ok(DeployPlan {
        profile_id,
        entries,
    })
}
