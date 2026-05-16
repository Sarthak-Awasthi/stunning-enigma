use anyhow::Context;
use sqlx::SqlitePool;
use tracing::info;

/// Run all embedded migrations in order.
pub async fn run(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations") // path is relative to this crate's Cargo.toml
        .run(pool)
        .await
        .context("database migration failed")?;

    info!("migrations applied");
    Ok(())
}
