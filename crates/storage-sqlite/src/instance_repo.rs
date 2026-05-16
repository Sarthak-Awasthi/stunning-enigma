use anyhow::Context;
use sqlx::SqlitePool;
use tracing::info;

pub struct InstanceRepo<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> InstanceRepo<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Insert or update a Fallout 4 instance by install path and return its id.
    pub async fn upsert_fallout4_instance(
        &self,
        install_path: &str,
        source_type: &str,
        label: &str,
    ) -> anyhow::Result<i64> {
        let game_id: i64 = sqlx::query_scalar("SELECT id FROM games WHERE canonical_id = ?1")
            .bind("fallout4")
            .fetch_one(self.pool)
            .await
            .context("failed to load fallout4 game id")?;

        let instance_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO instances (game_id, label, source_type, install_path)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(install_path) DO UPDATE SET
                label = excluded.label,
                source_type = excluded.source_type
            RETURNING id
            "#,
        )
        .bind(game_id)
        .bind(label)
        .bind(source_type)
        .bind(install_path)
        .fetch_one(self.pool)
        .await
        .context("failed to upsert game instance")?;

        info!(
            instance_id,
            install_path, source_type, "game instance upserted"
        );
        Ok(instance_id)
    }
}
