use sqlx::query;

use crate::{
    DbConn,
    models::user::{NewUser, User},
};

pub async fn make_admin(
    conn: &DbConn,
    tg_nick: &str,
    admin: bool,
) -> Result<Option<String>, sqlx::Error> {
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

    Ok(if result.rows_affected() == 1 {
        Some(tg_nick.to_owned())
    } else {
        None
    })
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

pub struct LB {
    pub scores: Vec<(String, u32)>,
    pub drinks_total: u32,
}
pub async fn leaderboard(conn: &DbConn) -> Result<LB, sqlx::Error> {
    let users = query!(
        r#"
        SELECT users.tg_nick, COUNT(drinks.id) AS "drinks" FROM users
        LEFT JOIN drinks ON drinks.user_id = users.id
        GROUP BY users.id, users.tg_nick
        ORDER BY COUNT(drinks.id) DESC
    "#
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .inspect(|u| println!("LB {u:?}"))
    .map(|r| (r.tg_nick, r.drinks as u32))
    .collect::<Vec<_>>();

    let drinks_total: u32 = users.iter().map(|(_u, d)| d).sum();

    Ok(LB {
        scores: users,
        drinks_total,
    })
}
