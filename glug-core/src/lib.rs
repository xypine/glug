pub mod database;
pub mod models;

use sqlx::sqlite::SqlitePoolOptions;

use crate::database::{
    drinks::drink,
    user::{LB, leaderboard},
};

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

    let starting_point = std::env::var("GG_STARTING_POINT")
        .ok()
        .and_then(|str| str.parse::<u32>().ok())
        .unwrap_or_default();

    let LB { drinks_total, .. } = leaderboard(db).await?;

    let difference = (starting_point as i32) - (drinks_total as i32);
    if difference > 0 {
        println!("System will drink {difference} drinks!");
        drink(
            db,
            None,
            Some("starting_point".to_owned()),
            0,
            difference as u32,
        )
        .await?;
    } else {
        println!("No drinks for the system tonight :(");
    }

    Ok(())
}
