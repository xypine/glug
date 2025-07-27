mod util;

use glug_glug_core::{
    connect_db,
    database::{
        drinks::{drink, import_drinks, undrink},
        user::{LB, fetch_user_or_create, leaderboard, make_admin},
    },
    models::user::NewUser,
};
use log::LevelFilter;
use teloxide::{prelude::*, sugar::request::RequestReplyExt, utils::command::BotCommands};

use crate::util::{format_with_spaces, progress_bar};

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .init();

    log::info!("Setting up...");
    let db_conn = connect_db()
        .await
        .expect("Failed to acquire database connection");
    glug_glug_core::init(&db_conn)
        .await
        .expect("Failed to init core");

    log::info!("Starting command bot...");

    let bot = Bot::from_env();

    bot.set_my_commands(Command::bot_commands())
        .await
        .expect("Failed to set commands");

    Command::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Tuetut komennot:")]
enum Command {
    #[command(alias = "help", description = "näytä tämä ohje")]
    Apua,
    #[command(description = "näytä tietoja itsestäsi")]
    Omat,
    #[command(
        aliases = ["j", "drink", "lörs"],
        description = "tallenna juoma, tai useampi lisäämällä perään numero"
    )]
    Juo(String),
    #[command(
        aliases = ["uj", "undrink", "eiku"],
        description = "peruuta viimeisin lisäys"
    )]
    Hups,
    #[command(description = "pimeä tie, hyvää matkaa", alias = "mittari")]
    Mittari,
    #[command(hide, alias = "op")]
    MakeAdmin(String),
    #[command(hide, alias = "deop")]
    StripAdmin(String),
    #[command(hide)]
    Import,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let Some(tg_user) = msg.from.clone() else {
        // messages from channels shouldn't be handled
        return Ok(());
    };
    let db_conn = connect_db().await.unwrap();
    let send_msg = async |msg_txt: String| {
        bot.send_message(msg.chat.id, msg_txt)
            .reply_to(msg.id)
            .await
            .map(|_sent| ())
    };
    let send_help_msg = || send_msg(Command::descriptions().to_string());

    let Ok(user) = fetch_user_or_create(
        &db_conn,
        NewUser {
            tg_id: tg_user.id.to_string(),
            tg_nick: tg_user.username.clone().unwrap_or("UNKNOWN".to_owned()),
        },
    )
    .await
    else {
        return send_msg("Tietokantavirhe :)".to_owned()).await;
    };

    let Some(user) = user else {
        return send_msg(format!(
            "Virhe haettaessa käyttäjää {}, {:?} 🥴",
            tg_user.id, tg_user.username
        ))
        .await;
    };

    match cmd {
        Command::Apua => return send_help_msg().await,
        Command::MakeAdmin(new_admin_nick) => {
            if !user.is_admin() {
                return send_help_msg().await;
            }
            let result = make_admin(&db_conn, &new_admin_nick, true).await;
            match result {
                Ok(Some(canonical_nick)) => {
                    send_msg(format!("@{canonical_nick} on nyt pääkäyttäjä")).await
                }
                Ok(None) | Err(_) => send_msg("Virhe :)".to_owned()).await,
            }
        }
        Command::StripAdmin(old_admin_nick) => {
            if !user.is_admin() {
                return send_help_msg().await;
            }
            let result = make_admin(&db_conn, &old_admin_nick, false).await;
            match result {
                Ok(Some(canonical_nick)) => {
                    send_msg(format!("@{canonical_nick} ei ole enää pääkäyttäjä")).await
                }
                Ok(None) | Err(_) => send_msg("Virhe :)".to_owned()).await,
            }
        }
        Command::Omat => {
            return send_msg(format!(
                r#"{} "{}"
========={}
Luotu {}
{} {} yhteensä
            "#,
                match (user.admin, user.is_admin()) {
                    (true, true) => "Järjestelmäkäyttäjä",
                    (true, false) => "Pääkäyttäjä",
                    (false, true) => "Järjestelmäkäyttäjä*",
                    _ => "Käyttäjä",
                },
                user.tg_nick,
                "=".repeat(user.tg_nick.len()),
                user.created_at.format("%d/%m/%Y"),
                user.drinks,
                if user.drinks == 1 { "juoma" } else { "juomaa" }
            ))
            .await;
        }
        Command::Juo(input) => {
            let input = input.trim();
            let Some(drink_count) = (if input.is_empty() {
                Some(1)
            } else {
                input.parse::<u8>().ok()
            }) else {
                // invalid drink count
                return send_msg(format!(
                    "🥴 no älä ainakaan juo seuraavaa. \"{input}\" ei käy juomien määrästä"
                ))
                .await;
            };
            let new_total = drink(
                &db_conn,
                Some(msg.id.to_string()),
                None,
                user.id,
                drink_count as u32,
            )
            .await
            .unwrap();
            return send_msg(format!(
                "🍻 lisättiin {drink_count} {}, yhteensä {new_total}",
                if drink_count == 1 { "juoma" } else { "juomaa" }
            ))
            .await;
        }
        Command::Hups => {
            let result = undrink(&db_conn, user.id).await.unwrap();
            return send_msg(format!(
                "🕊 peruutettiin {result} {}",
                if result == 1 { "juoma" } else { "juomaa" }
            ))
            .await;
        }
        Command::Mittari => {
            let LB {
                scores,
                drinks_total,
            } = leaderboard(&db_conn).await.unwrap();

            let mut response = format!(
                "{} / 10 000 juomaa yhteensä",
                format_with_spaces(drinks_total)
            );
            response = format!("{response}\n{}", "=".repeat(response.len()));
            for (a, b) in &scores {
                let drinks = if *b == 1 { "juoma" } else { "juomaa" };
                response = format!("{response}\n{a}: {b} {drinks}");
            }
            response = format!(
                "{response}\n\n0 {} 10k",
                progress_bar(drinks_total as usize, 10_000)
            );

            if scores.is_empty() {
                response = "Ei juomia".to_owned();
            }

            return send_msg(response).await;
        }
        Command::Import => {
            if !user.is_super_admin() {
                return send_help_msg().await;
            }

            let Ok(file_path) = std::env::var("GG_IMPORT_SRC") else {
                return send_msg("import path not configured".to_owned()).await;
            };

            let Ok(mut file) = std::fs::File::open(file_path) else {
                return send_msg("failed to read file".to_owned()).await;
            };

            let mut buffer = String::new();
            std::io::Read::read_to_string(&mut file, &mut buffer).unwrap();

            let Ok(drinks) = glug_glug_importer::parse(buffer) else {
                return send_msg("failed to parse file".to_owned()).await;
            };

            println!("IMPORTING {} drinks", drinks.len());

            let Ok(total) = import_drinks(
                &db_conn,
                Some(msg.id.to_string()),
                drinks
                    .into_iter()
                    .map(|d| {
                        (
                            d.user_tg_id.strip_prefix("user").unwrap().to_owned(),
                            d.user_tg_nick,
                            d.timestamp,
                        )
                    })
                    .collect(),
            )
            .await
            .inspect_err(|e| log::error!("failed to import drinks: {}", e)) else {
                return send_msg("failed to import drinks".to_owned()).await;
            };

            println!("TOTAL OK {total}");

            return send_msg("OK".to_owned()).await;
        }
    }
}
