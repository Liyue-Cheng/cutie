use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use tauri::{AppHandle, Manager};

const DB_URL: &str = "sqlite://cutie.db";

pub type DbPool = SqlitePool;

pub async fn init_db(app_handle: &AppHandle) -> Result<DbPool, sqlx::Error> {
    let db_path = app_handle
        .path()
        .resolve(
            DB_URL.strip_prefix("sqlite://").unwrap(),
            tauri::path::BaseDirectory::AppLocalData,
        )
        .unwrap();

    let db_url = db_path.to_str().unwrap();

    if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
        Sqlite::create_database(db_url).await?;
    }

    let pool = SqlitePool::connect(db_url).await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
