#![allow(dead_code)]

use std::sync::Arc;
use std::collections::HashMap;

use log::*;

use tokio::sync::Mutex;

mod drivers;
mod ws;
mod watchdog;
mod store;

fn init_logging() {
    env_logger::Builder::new()
        .format_timestamp(None)
        .filter(None, LevelFilter::Info)
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    let args: Vec<String> = std::env::args().collect();

    let ws_clients: ws::ws::Clients = Arc::new(Mutex::new(HashMap::new()));

    if let Some(arg1) = args.get(1) {
        match arg1.as_str() {
            "watchdog" => {
                info!("You provided the argument `watchdog`, I'll only run the watchdog client");
                watchdog::watch(&ws_clients).await;
            }
            "ws" => {
                info!("You provided the argument `ws`, I'll only run the WebSocket server");
                ws::run(&ws_clients).await;
            },
            _ => info!("You provided an argument (`{arg1}`), but I don't know that argument")
        }
        return Ok(());
    }

    info!("No valid arguments provided, running both the WebSocket server and the watchdog client");
    tokio::join!(
        ws::run(&ws_clients),
        watchdog::watch(&ws_clients)
    );

    Ok(())
}
