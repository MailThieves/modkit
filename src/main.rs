#![allow(dead_code)]

use std::sync::Arc;
use std::collections::HashMap;

use log::*;

use tokio::sync::Mutex;

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

    let ws_clients: ws::ws::Clients = Arc::new(Mutex::new(HashMap::new()));

    // This will work! I think! We just need to let them share data
    tokio::join!(
        ws::run(&ws_clients),
        watchdog::watch(&ws_clients)
    );
}
