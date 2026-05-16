use anyhow::Context;
use sqlx::SqlitePool;
use tracing::info;

use domain_core::entities::Profile;

pub struct ProfileRepo<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> ProfileRepo<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new profile for an instance. Returns the new profile's id.
    pub async fn create(
        &self,
        instance_id: i64,
        name: &str,
    ) -> anyhow::Result<i64> {
        let id = sqlx::query!(
            r#"
            INSERT INTO profiles (instance_id, name)
            VALUES (?1, ?2)
            "#,
            instance_id,
            name,
        )
        .execute(self.pool)
        .await
        .context("failed to create profile")?
        .last_insert_rowid();

        info!(profile_id = id, name, "profile created");
        Ok(id)
    }

    /// List all profiles for an instance.
    pub async fn list(&self, instance_id: i64) -> anyhow::Result<Vec<Profile>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, instance_id, name, pinned_runner_id, auto_deploy
            FROM profiles
            WHERE instance_id = ?1
            ORDER BY created_at
            "#,
            instance_id,
        )
        .fetch_all(self.pool)
        .await
        .context("failed to list profiles")?;

        Ok(rows
            .into_iter()
            .map(|r| Profile {
                id:               r.id.expect("id is always set for persisted rows"),
                instance_id:      r.instance_id,
                name:             r.name,
                pinned_runner_id: r.pinned_runner_id,
                auto_deploy:      r.auto_deploy != 0,
            })
            .collect())
    }

    /// Delete a profile and cascade all its mod/plugin state.
    pub async fn delete(&self, profile_id: i64) -> anyhow::Result<()> {
        sqlx::query!("DELETE FROM profiles WHERE id = ?1", profile_id)
            .execute(self.pool)
            .await
            .context("failed to delete profile")?;

        info!(profile_id, "profile deleted");
        Ok(())
    }
}