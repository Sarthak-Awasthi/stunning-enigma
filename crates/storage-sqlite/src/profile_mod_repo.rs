use anyhow::Context;
use sqlx::{Row, SqlitePool};
use tracing::info;

#[derive(Debug, Clone)]
pub struct ProfileModRow {
    pub mod_id: i64,
    pub mod_name: String,
    pub enabled: bool,
    pub priority: i32,
}

pub struct ProfileModRepo<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> ProfileModRepo<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, profile_id: i64) -> anyhow::Result<Vec<ProfileModRow>> {
        let rows = sqlx::query(
            r#"
            SELECT pm.mod_id, m.name AS mod_name, pm.enabled, pm.priority
            FROM profile_mods pm
            JOIN mods m ON m.id = pm.mod_id
            WHERE pm.profile_id = ?1
            ORDER BY pm.priority ASC
            "#,
        )
        .bind(profile_id)
        .fetch_all(self.pool)
        .await
        .context("failed to list profile mods")?;

        rows.into_iter()
            .map(|row| {
                Ok(ProfileModRow {
                    mod_id: row.try_get("mod_id")?,
                    mod_name: row.try_get("mod_name")?,
                    enabled: row.try_get::<i64, _>("enabled")? != 0,
                    priority: row.try_get("priority")?,
                })
            })
            .collect()
    }

    pub async fn upsert(
        &self,
        profile_id: i64,
        mod_id: i64,
        enabled: bool,
        priority: i32,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO profile_mods (profile_id, mod_id, enabled, priority)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(profile_id, mod_id) DO UPDATE SET
                enabled = excluded.enabled,
                priority = excluded.priority
            "#,
        )
        .bind(profile_id)
        .bind(mod_id)
        .bind(if enabled { 1_i64 } else { 0_i64 })
        .bind(priority)
        .execute(self.pool)
        .await
        .context("failed to upsert profile mod state")?;

        info!(
            profile_id,
            mod_id, enabled, priority, "profile mod state upserted"
        );
        Ok(())
    }

    pub async fn set_enabled(
        &self,
        profile_id: i64,
        mod_id: i64,
        enabled: bool,
    ) -> anyhow::Result<()> {
        let result = sqlx::query(
            "UPDATE profile_mods SET enabled = ?1 WHERE profile_id = ?2 AND mod_id = ?3",
        )
        .bind(if enabled { 1_i64 } else { 0_i64 })
        .bind(profile_id)
        .bind(mod_id)
        .execute(self.pool)
        .await
        .context("failed to update profile mod enabled state")?;

        if result.rows_affected() == 0 {
            anyhow::bail!("mod {mod_id} is not attached to profile {profile_id}");
        }

        info!(profile_id, mod_id, enabled, "profile mod enabled updated");
        Ok(())
    }

    pub async fn set_priority(
        &self,
        profile_id: i64,
        mod_id: i64,
        priority: i32,
    ) -> anyhow::Result<()> {
        let result = sqlx::query(
            "UPDATE profile_mods SET priority = ?1 WHERE profile_id = ?2 AND mod_id = ?3",
        )
        .bind(priority)
        .bind(profile_id)
        .bind(mod_id)
        .execute(self.pool)
        .await
        .context("failed to update profile mod priority")?;

        if result.rows_affected() == 0 {
            anyhow::bail!("mod {mod_id} is not attached to profile {profile_id}");
        }

        info!(profile_id, mod_id, priority, "profile mod priority updated");
        Ok(())
    }
}
