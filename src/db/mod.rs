pub mod repositories;

use anyhow::Result;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};
use std::fs;

use crate::config::DatabaseTarget;

pub async fn connect_sqlite(database: &DatabaseTarget) -> Result<SqlitePool> {
    let options = match database {
        DatabaseTarget::Memory => SqliteConnectOptions::new().in_memory(true),
        DatabaseTarget::File(path) => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            SqliteConnectOptions::new()
                .filename(path.as_path())
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
        }
    };

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;

    Ok(pool)
}
