use sqlx::{query, query_scalar};

use crate::{DbConn, models::user::UserId};

pub async fn drink(conn: &DbConn, user_id: UserId, amount: u32) -> Result<u32, sqlx::Error> {
    let mut tx = conn.begin().await?;
    let call_id = query_scalar!(
        "INSERT INTO calls (user_id) values (?) RETURNING id",
        user_id
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
