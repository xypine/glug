pub mod database;
pub mod models;

use sqlx::sqlite::SqlitePoolOptions;

pub type DbConn = sqlx::SqlitePool;

pub async fn connect_db() -> Result<DbConn, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}

pub async fn init(db: &DbConn) -> Result<(), sqlx::Error> {
    sqlx::migrate!("../migrations").run(db).await?;

    Ok(())
}
