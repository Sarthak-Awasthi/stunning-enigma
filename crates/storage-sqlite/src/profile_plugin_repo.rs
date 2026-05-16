use anyhow::Context;
use sqlx::{Row, SqlitePool};
use tracing::info;

pub struct ProfilePluginRepo<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> ProfilePluginRepo<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, profile_id: i64) -> anyhow::Result<Vec<ipc_api::ProfilePluginInfo>> {
        let rows = sqlx::query(
            r#"
            SELECT pp.plugin_id, p.filename, p.kind, pp.enabled, pp.load_index
            FROM profile_plugins pp
            JOIN plugins p ON p.id = pp.plugin_id
            WHERE pp.profile_id = ?1
            ORDER BY pp.load_index ASC
            "#,
        )
        .bind(profile_id)
        .fetch_all(self.pool)
        .await
        .context("failed to list profile plugins")?;

        rows.into_iter()
            .map(|row| {
                Ok(ipc_api::ProfilePluginInfo {
                    plugin_id: row.try_get("plugin_id")?,
                    filename: row.try_get("filename")?,
                    kind: row.try_get("kind")?,
                    enabled: row.try_get::<i64, _>("enabled")? != 0,
                    load_index: row.try_get("load_index")?,
                })
            })
            .collect()
    }

    pub async fn set_enabled(
        &self,
        profile_id: i64,
        plugin_id: i64,
        enabled: bool,
    ) -> anyhow::Result<()> {
        sqlx::query("UPDATE profile_plugins SET enabled = ?1 WHERE profile_id = ?2 AND plugin_id = ?3")
            .bind(if enabled { 1_i64 } else { 0_i64 })
            .bind(profile_id)
            .bind(plugin_id)
            .execute(self.pool)
            .await
            .context("failed to update plugin enabled state")?;

        info!(profile_id, plugin_id, enabled, "plugin enabled state updated");
        Ok(())
    }
}