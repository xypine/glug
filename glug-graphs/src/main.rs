use std::{fs::File, io::Write as _};

use glug_glug_core::{connect_db, database::drinks::day_stats};
use glug_glug_graphs::graph;

#[tokio::main]
async fn main() {
    let db_conn = connect_db()
        .await
        .expect("Failed to acquire database connection");
    let stats = day_stats(&db_conn)
        .await
        .expect("Failed to fetch stats")
        .expect("No stats to graph");
    let bytes = graph(stats).unwrap();
    let mut file = File::create("./sample.png").expect("failed to open output file");
    // Write a slice of bytes to the file
    file.write_all(&bytes)
        .expect("failed to write to output file");
}
