pub mod repositories;

use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub async fn connect_sqlite(url: &str) -> Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(url)
        .await?;

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;

    Ok(pool)
}
