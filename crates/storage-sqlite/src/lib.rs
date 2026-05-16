pub mod instance_repo;
pub mod migrations;
pub mod profile_mod_repo;
pub mod profile_repo;

use anyhow::Context;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::{path::Path, str::FromStr};
use tracing::info;

/// The main handle to the SQLite database.
/// Clone it freely — the pool manages connections internally.
#[derive(Clone, Debug)]
pub struct Db {
    pub pool: SqlitePool,
}

impl Db {
    /// Open (or create) the database at `path` and run all pending migrations.
    pub async fn open(path: &Path) -> anyhow::Result<Self> {
        let url = format!("sqlite://{}?mode=rwc", path.display());

        let options = SqliteConnectOptions::from_str(&url)
            .context("invalid database URL")?
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal) // better concurrency
            .foreign_keys(true); // enforce FK constraints

        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await
            .context("failed to open SQLite database")?;

        info!(path = %path.display(), "database opened");

        migrations::run(&pool).await?;

        Ok(Self { pool })
    }
}
