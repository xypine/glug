use sqlx::query;

use crate::{
    DbConn,
    models::user::{NewUser, User},
};

pub async fn make_admin(conn: &DbConn, tg_nick: &str, admin: bool) -> Result<bool, sqlx::Error> {
    let tg_nick = tg_nick.strip_prefix("@").unwrap_or(tg_nick);
    let result = query!(
        r#"
        UPDATE users
        SET admin = $2
        WHERE tg_nick = $1
    "#,
        tg_nick,
        admin
    )
    .execute(conn)
    .await?;

    Ok(result.rows_affected() == 1)
}

pub async fn fetch_user_or_create(
    conn: &DbConn,
    vals: NewUser,
) -> Result<Option<User>, sqlx::Error> {
    println!("CHECK {vals:?}");
    let tg_id = vals.tg_id;
    let tg_nick = vals.tg_nick;

    let u = query!(
        r#"
        INSERT OR IGNORE INTO users (tg_id, tg_nick) 
        VALUES ($1, $2);

        SELECT users.*, COUNT(drinks.id) AS "drinks" FROM users
        LEFT JOIN drinks ON drinks.user_id = users.id
        WHERE tg_id = $1
    "#,
        tg_id,
        tg_nick,
        tg_id
    )
    .fetch_optional(conn)
    .await?
    .and_then(|r| {
        println!("RAW {r:?}");
        let id = r.id?;
        let tg_id = r.tg_id?;
        let tg_nick = r.tg_nick?;
        let admin = r.admin?;
        let created_at = r.created_at?;
        let updated_at = r.updated_at?;
        Some(User {
            id,
            tg_id,
            tg_nick,
            admin,
            drinks: r.drinks,
            created_at: created_at.and_utc(),
            updated_at: updated_at.and_utc(),
        })
    });

    println!("USER {u:?}");

    Ok(u)
}

pub async fn leaderboard(conn: &DbConn) -> Result<Vec<(String, u8)>, sqlx::Error> {
    let users = query!(
        r#"
        SELECT users.tg_nick, COUNT(drinks.id) AS "drinks" FROM users
        LEFT JOIN drinks ON drinks.user_id = users.id
    "#
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .flat_map(|r| match (r.tg_nick, r.drinks) {
        (Some(n), d) => Some((n, d)),
        (_, _) => None,
    })
    .map(|(n, d)| (n, d as u8))
    .collect::<Vec<_>>();

    Ok(users)
}
