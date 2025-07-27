use std::{
    fs::File,
    io::{BufReader, Read, Write},
    ptr::copy_nonoverlapping,
};

use glug_glug_core::{connect_db, database::drinks::import_drinks};
use glug_glug_importer::parse;

#[tokio::main]
async fn main() {
    let mut file = File::open("./result.json").unwrap();

    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();

    let drinks = parse(buffer).unwrap();

    println!("{}", drinks.len());

    let db_conn = connect_db()
        .await
        .expect("Failed to acquire database connection");
    glug_glug_core::init(&db_conn)
        .await
        .expect("Failed to init core");
    let total = import_drinks(
        &db_conn,
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
    .unwrap();

    println!("TOTAL {total}");

    // let mut file = File::create("output.txt").unwrap();
    // file.write_all(results.as_bytes()).unwrap();
}
