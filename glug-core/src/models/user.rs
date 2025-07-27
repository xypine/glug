use chrono::NaiveDateTime;
use sqlx::{Decode, Encode};

use crate::models::DateTime;

pub type UserId = i64;
pub type UserTgId = String;

#[derive(Debug)]
pub struct NewUser {
    pub tg_id: UserTgId,
    pub tg_nick: String,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct UserRaw {
    pub id: UserId,
    pub tg_id: UserTgId,
    pub tg_nick: String,
    pub admin: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub drinks: i64,
}

impl From<UserRaw> for User {
    fn from(raw: UserRaw) -> Self {
        Self {
            id: raw.id,
            tg_id: raw.tg_id,
            tg_nick: raw.tg_nick,
            admin: raw.admin,
            drinks: raw.drinks,
            created_at: raw.created_at.and_utc(),
            updated_at: raw.updated_at.and_utc(),
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct User {
    pub id: UserId,
    pub tg_id: UserTgId,
    pub tg_nick: String,
    pub admin: bool,
    pub drinks: i64,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl User {
    pub fn is_super_admin(&self) -> bool {
        self.tg_nick == std::env::var("GG_ROOT_ADMIN").unwrap_or_default()
    }
    pub fn is_admin(&self) -> bool {
        self.admin || self.is_super_admin()
    }
}
