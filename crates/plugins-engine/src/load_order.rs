use anyhow::Context;
use std::path::Path;
use tracing::info;

use storage_sqlite::Db;

use crate::parser::parse_plugin_header;

/// Scan a deployed Data directory for plugins and sync them into the DB
/// for the given profile. New plugins get appended at the end of the load order.
pub async fn sync_plugins(profile_id: i64, data_dir: &Path, db: &Db) -> anyhow::Result<usize> {
    let mut added = 0usize;

    let mut read_dir = tokio::fs::read_dir(data_dir)
        .await
        .context("failed to read data directory")?;

    while let Some(entry) = read_dir.next_entry().await? {
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !matches!(ext.as_str(), "esp" | "esm" | "esl") {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Parse header for kind and masters
        let header = match parse_plugin_header(&path).await {
            Ok(h) => h,
            Err(e) => {
                tracing::warn!(file = %filename, err = %e, "skipping invalid plugin");
                continue;
            }
        };

        let kind_str = format!("{:?}", header.kind).to_lowercase();
        let masters_json = serde_json::to_string(&header.masters).unwrap_or_else(|_| "[]".into());

        // Upsert plugin record
        let plugin_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO plugins (filename, kind, masters_json)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(filename) DO UPDATE SET
                kind = excluded.kind,
                masters_json = excluded.masters_json
            RETURNING id
            "#,
        )
        .bind(&filename)
        .bind(&kind_str)
        .bind(&masters_json)
        .fetch_one(&db.pool)
        .await
        .context("failed to upsert plugin")?;

        // Add to profile if not already present
        let already = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM profile_plugins WHERE profile_id=?1 AND plugin_id=?2",
            profile_id,
            plugin_id,
        )
        .fetch_one(&db.pool)
        .await?
            > 0;

        if !already {
            // Append at end of load order
            let next_index = sqlx::query_scalar!(
                "SELECT COALESCE(MAX(load_index), -1) + 1 FROM profile_plugins WHERE profile_id=?1",
                profile_id,
            )
            .fetch_one(&db.pool)
            .await?;

            sqlx::query!(
                "INSERT INTO profile_plugins (profile_id, plugin_id, enabled, load_index)
                 VALUES (?1, ?2, 1, ?3)",
                profile_id,
                plugin_id,
                next_index,
            )
            .execute(&db.pool)
            .await?;

            added += 1;
        }
    }

    info!(profile_id, added, "plugin sync complete");
    Ok(added)
}

/// Write the profile's load order to plugins.txt and loadorder.txt.
pub async fn write_load_order(profile_id: i64, profile_dir: &Path, db: &Db) -> anyhow::Result<()> {
    let rows = sqlx::query!(
        r#"
        SELECT p.filename, pp.enabled
        FROM profile_plugins pp
        JOIN plugins p ON p.id = pp.plugin_id
        WHERE pp.profile_id = ?1
        ORDER BY pp.load_index
        "#,
        profile_id,
    )
    .fetch_all(&db.pool)
    .await
    .context("failed to load plugin order")?;

    // plugins.txt: enabled plugins prefixed with *
    let plugins_txt = rows
        .iter()
        .map(|r| {
            if r.enabled != 0 {
                format!("*{}", r.filename)
            } else {
                r.filename.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // loadorder.txt: all plugins in order, no prefix
    let loadorder_txt = rows
        .iter()
        .map(|r| r.filename.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    tokio::fs::write(profile_dir.join("plugins.txt"), plugins_txt)
        .await
        .context("failed to write plugins.txt")?;

    tokio::fs::write(profile_dir.join("loadorder.txt"), loadorder_txt)
        .await
        .context("failed to write loadorder.txt")?;

    info!(profile_id, "load order written");
    Ok(())
}
