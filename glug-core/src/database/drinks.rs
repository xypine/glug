use std::collections::BTreeSet;

use chrono::{DateTime, NaiveDate, NaiveTime};
use sqlx::{query, query_scalar};

use crate::{
    DbConn,
    database::user::fetch_user_or_create,
    models::user::{NewUser, UserId},
};

pub async fn drink(
    conn: &DbConn,
    tg_msg_id: Option<String>,
    automation: Option<String>,
    user_id: UserId,
    amount: u32,
) -> Result<u32, sqlx::Error> {
    let mut tx = conn.begin().await?;
    let call_id = query_scalar!(
        "INSERT INTO calls (user_id, tg_msg_id, automation) values (?, ?, ?) RETURNING id",
        user_id,
        tg_msg_id,
        automation
    )
    .fetch_one(&mut *tx)
    .await?;
    for _ in 0..amount {
        query!(
            r#"
                INSERT INTO drinks_archive (user_id, call_id) VALUES (?, ?)
            "#,
            user_id,
            call_id
        )
        .execute(&mut *tx)
        .await?;
    }

    let new_total = query_scalar!("SELECT COUNT(*) FROM drinks")
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(new_total as u32)
}

pub async fn import_drinks(
    conn: &DbConn,
    trigger_msg_id: Option<String>,
    drinks: Vec<(String, String, usize)>,
) -> Result<u32, sqlx::Error> {
    query!(
        r#"
            UPDATE drinks_archive
            SET deleted_at = CURRENT_TIMESTAMP
            WHERE user_id = 0
        "#,
    )
    .execute(conn)
    .await?;
    query!(
        r#"
            UPDATE drinks_archive
            SET deleted_at = CURRENT_TIMESTAMP
            WHERE call_id IN (SELECT id FROM calls WHERE automation = 'import')
        "#,
    )
    .execute(conn)
    .await?;
    let user_ids: BTreeSet<(String, String)> = drinks
        .iter()
        .map(|(uid, nick, _)| (uid.clone(), nick.clone()))
        .collect();
    for (tg_id, tg_nick) in &user_ids {
        fetch_user_or_create(
            conn,
            NewUser {
                tg_id: tg_id.clone(),
                tg_nick: tg_nick.clone(),
            },
        )
        .await?;
    }
    let mut tx = conn.begin().await?;
    for (tg_id, _) in user_ids {
        let call = query!(
            r#"
                WITH user AS (
                   SELECT id FROM users WHERE tg_id = ?
                )
                INSERT INTO calls (user_id, tg_msg_id, automation)
                VALUES (
                    (SELECT id FROM user),
                    ?,
                    'import'
                )
                RETURNING calls.id, calls.user_id
            "#,
            tg_id,
            trigger_msg_id
        )
        .fetch_one(&mut *tx)
        .await?;
        let user_drinks = drinks.iter().filter(|(uid, _, _)| uid.clone() == tg_id);
        for (_, _, ts) in user_drinks {
            let ts = *ts as i64;
            let uid = call.user_id;
            let cid = call.id;
            query!(
                r#"
                    INSERT INTO drinks_archive (user_id, call_id, created_at) VALUES (?, ?, ?)
                "#,
                uid,
                cid,
                ts
            )
            .execute(&mut *tx)
            .await?;
        }
        println!("imported user {} ({})", tg_id, call.user_id);
    }

    let new_total = query_scalar!("SELECT COUNT(*) FROM drinks")
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(new_total as u32)
}

pub async fn undrink(conn: &DbConn, user_id: UserId) -> Result<u64, sqlx::Error> {
    let result = query!(
        r#"
        WITH last_call AS (
            SELECT * FROM calls WHERE user_id = ?
            AND (
                SELECT id FROM drinks_archive
                WHERE call_id = calls.id 
                AND deleted_at IS NULL
            ) IS NOT NULL
            ORDER BY created_at DESC
            LIMIT 1
        )
        UPDATE drinks_archive
        SET deleted_at = CURRENT_TIMESTAMP
        WHERE drinks_archive.call_id = (SELECT id FROM last_call)
    "#,
        user_id,
    )
    .execute(conn)
    .await?;

    println!("UNDRANK ${result:#?}");

    Ok(result.rows_affected())
}

pub struct DateStat {
    pub date: NaiveDate,
    pub drinks_total: u32,
}
pub struct DateStats {
    pub x_min: NaiveDate,
    pub x_max: NaiveDate,
    pub y_min: u32,
    pub y_max: u32,
    pub stats: Vec<DateStat>,
}
impl DateStats {
    pub fn linear_approx(&self, target_drinks: u32) -> DateStat {
        if self.y_max >= target_drinks {
            return DateStat {
                date: self.x_max,
                drinks_total: self.y_max,
            };
        }

        let x_min = (self
            .x_min
            .and_time(NaiveTime::MIN)
            .and_utc()
            .timestamp_nanos_opt()
            .unwrap()
            / 100_000_000_000) as f64;
        let x_max = (self
            .x_max
            .and_time(NaiveTime::MIN)
            .and_utc()
            .timestamp_nanos_opt()
            .unwrap()
            / 100_000_000_000) as f64;

        let y_min = self.y_min as f64;
        let y_max = self.y_max as f64;

        dbg!(x_min, x_max, y_min, y_max);

        let slope = (y_max - y_min) / (x_max - x_min);
        println!("slope {slope}");

        // x*slope + y_min = target
        // x*slope = target - y_min
        // x = (target - y_min) / slope

        let date_nanos = (target_drinks as f64 - y_min) / slope;
        println!("date_nanos {date_nanos}");

        let date =
            DateTime::from_timestamp_nanos(((x_min + date_nanos) * 100_000_000_000.0) as i64)
                .date_naive();
        println!("date {date}");

        DateStat {
            date,
            drinks_total: target_drinks,
        }
    }
}
pub async fn day_stats(conn: &DbConn) -> Result<Option<DateStats>, sqlx::Error> {
    let stats = query!(
        r#"
        SELECT date(drinks.created_at) AS "date: NaiveDate", COUNT(drinks.id) AS "drinks" FROM drinks
        GROUP BY "date: NaiveDate"
        ORDER BY created_at ASC
    "#
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|r| (r.date, r.drinks as u32))
    .collect::<Vec<_>>();

    let mut y_min = 0;
    let mut drinks_total = 0;

    let stats = stats
        .into_iter()
        .filter_map(|(date, drinks)| {
            if drinks_total == 0 {
                y_min = drinks;
            }
            drinks_total += drinks;
            date.map(|date| DateStat { date, drinks_total })
        })
        .collect::<Vec<_>>();

    let Some(first_stat) = stats.first() else {
        return Ok(None);
    };
    let Some(last_stat) = stats.last() else {
        return Ok(None);
    };

    let x_min = first_stat.date;
    let x_max = last_stat.date;

    Ok(Some(DateStats {
        x_min,
        x_max,
        y_min,
        y_max: drinks_total,
        stats,
    }))
}
