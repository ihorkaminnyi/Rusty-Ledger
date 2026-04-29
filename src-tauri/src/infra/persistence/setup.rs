use std::{str::FromStr, time::Duration};
use tauri::Manager;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};

use crate::error::DbError;

pub async fn init_db(db_url: &str) -> Result<SqlitePool, DbError> {
    let connect_options = SqliteConnectOptions::from_str(db_url)
        .map_err(DbError::Connect)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await
        .map_err(DbError::Connect)?;

    sqlx::migrate!("./src/infra/persistence/migrations")
        .run(&pool)
        .await
        .map_err(DbError::Migrate)?;

    Ok(pool)
}

pub fn resolve_db_url(app: &tauri::AppHandle) -> Result<String, Box<dyn std::error::Error>> {
    let app_data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data_dir)?;
    let db_path = app_data_dir.join("rusty_ledger.db");
    Ok(format!("sqlite:{}", db_path.to_string_lossy()))
}
