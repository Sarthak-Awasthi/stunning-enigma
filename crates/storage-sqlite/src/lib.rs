//! SQLite storage layer for the Mod Manager.
//!
//! This crate provides persistent storage using SQLite with SQLx, including:
//! - Database connection pooling with WAL mode for better concurrency
//! - Automatic migrations on startup
//! - Repository pattern for data access
//!
//! # Example
//!
//! ```rust,no_run
//! use storage_sqlite::Db;
//! use std::path::Path;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let db = Db::open(Path::new("state.db")).await?;
//!     // Use db.pool for queries or repositories for structured access
//!     Ok(())
//! }
//! ```

pub mod instance_repo;
pub mod migrations;
pub mod profile_env_repo;
pub mod profile_mod_repo;
pub mod profile_repo;
pub mod profile_plugin_repo;

use anyhow::Context;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::{path::Path, str::FromStr};
use tracing::info;

/// The maximum number of concurrent connections in the pool.
const MAX_CONNECTIONS: u32 = 4;

/// The main handle to the SQLite database.
///
/// Clone it freely — the pool manages connections internally.
#[derive(Clone, Debug)]
pub struct Db {
    pub pool: SqlitePool,
}

impl Db {
    /// Open (or create) the database at `path` and run all pending migrations.
    ///
    /// Uses WAL journal mode for better concurrent read/write performance.
    pub async fn open(path: &Path) -> anyhow::Result<Self> {
        let url = format!("sqlite://{}?mode=rwc", path.display());

        let options = SqliteConnectOptions::from_str(&url)
            .context("invalid database URL")?
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal) // better concurrency
            .foreign_keys(true); // enforce FK constraints

        let pool = SqlitePoolOptions::new()
            .max_connections(MAX_CONNECTIONS)
            .connect_with(options)
            .await
            .context("failed to open SQLite database")?;

        info!(path = %path.display(), "database opened");

        migrations::run(&pool).await?;

        Ok(Self { pool })
    }
}
