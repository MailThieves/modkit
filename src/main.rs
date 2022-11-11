#![allow(dead_code)]

use log::*;

mod drivers;
mod ws;
mod watchdog;

fn init_logging() {
    env_logger::Builder::new()
        .format_timestamp(None)
        .filter(None, LevelFilter::Info)
        .init();
}

#[tokio::main]
async fn main() {
    init_logging();
    ws::run().await;
}
