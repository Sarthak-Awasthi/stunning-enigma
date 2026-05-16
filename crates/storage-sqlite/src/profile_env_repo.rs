use anyhow::Context;
use sqlx::SqlitePool;
use tracing::info;

pub struct ProfileEnvRepo<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> ProfileEnvRepo<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// List all environment variables for a profile.
    pub async fn list(&self, profile_id: i64) -> anyhow::Result<Vec<(String, String)>> {
        let rows = sqlx::query!(
            r#"
            SELECT key, value
            FROM profile_env_vars
            WHERE profile_id = ?1
            ORDER BY key
            "#,
            profile_id,
        )
        .fetch_all(self.pool)
        .await
        .context("failed to list profile env vars")?;

        Ok(rows.into_iter().map(|r| (r.key, r.value)).collect())
    }

    /// Set or update an environment variable for a profile.
    pub async fn set(&self, profile_id: i64, key: &str, value: &str) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO profile_env_vars (profile_id, key, value)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(profile_id, key) DO UPDATE SET value = excluded.value
            "#,
            profile_id,
            key,
            value,
        )
        .execute(self.pool)
        .await
        .context("failed to set profile env var")?;

        info!(profile_id, key, "profile env var set");
        Ok(())
    }

    /// Delete an environment variable from a profile.
    pub async fn delete(&self, profile_id: i64, key: &str) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM profile_env_vars
            WHERE profile_id = ?1 AND key = ?2
            "#,
            profile_id,
            key,
        )
        .execute(self.pool)
        .await
        .context("failed to delete profile env var")?;

        info!(profile_id, key, "profile env var deleted");
        Ok(())
    }

    /// Clear all environment variables for a profile.
    pub async fn clear_all(&self, profile_id: i64) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM profile_env_vars
            WHERE profile_id = ?1
            "#,
            profile_id,
        )
        .execute(self.pool)
        .await
        .context("failed to clear profile env vars")?;

        info!(profile_id, "all profile env vars cleared");
        Ok(())
    }
}
