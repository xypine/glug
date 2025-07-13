use glug_glug_core::{
    connect_db,
    database::user::{fetch_user_or_create, leaderboard},
    models::user::NewUser,
};
use log::LevelFilter;
use teloxide::{prelude::*, utils::command::BotCommands};

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

    Command::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Tuetut komennot:")]
enum Command {
    #[command(alias = "help", description = "näytä tämä ohje")]
    Apua,
    #[command(
        aliases = ["j", "drink"],
        description = "tallenna juoma, tai useampi lisäämällä perään numero"
    )]
    Juo(String),
    #[command()]
    Laske,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let Some(tg_user) = msg.from else {
        // messages from channels shouldn't be handled
        return Ok(());
    };
    let conn = connect_db().await.unwrap();
    let send_error_msg =
        async |err_msg: &str| bot.send_message(msg.chat.id, err_msg).await.map(|_sent| ());
    let user = fetch_user_or_create(
        &conn,
        NewUser {
            tg_id: tg_user.id.to_string(),
            tg_nick: tg_user.username.unwrap_or("UNKNOWN".to_owned()),
        },
    )
    .await
    .unwrap();
    match cmd {
        Command::Apua => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Juo(input) => {
            let input = input.trim();
            let Some(drink_count) = (if input.is_empty() {
                Some(1)
            } else {
                input.parse::<u8>().ok()
            }) else {
                // invalid drink count
                return send_error_msg(&format!(
                    "No älä ainakaan juo seuraavaa. \"{input}\" ei käy juomien määrästä 🥴"
                ))
                .await;
            };
            bot.send_message(msg.chat.id, format!("got '{drink_count:?}'"))
                .await?
        }
        Command::Laske => {
            let mut leaderboard = leaderboard(&conn).await.unwrap();
            leaderboard.sort_by_key(|(_a, b)| -(*b as i16));

            let mut response = "Kaikki juomat".to_owned();
            response = format!("{response}\n{}", "=".repeat(response.len()));
            for (a, b) in &leaderboard {
                let drinks = if *b == 1 { "juoma" } else { "juomaa" };
                response = format!("{response}\n{a}: {b} {drinks}");
            }

            if leaderboard.is_empty() {
                response = "Ei juomia".to_owned();
            }

            bot.send_message(msg.chat.id, response).await?
        }
    };

    Ok(())
}
