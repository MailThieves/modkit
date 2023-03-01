#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use log::*;

use tokio::sync::Mutex;

mod drivers;
mod model;
mod server;
mod store;
mod watchdog;

fn init_logging() {
    env_logger::Builder::new()
        .format_timestamp(None)
        .filter(None, LevelFilter::Info)
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    // Try to connect to the DB so we get a nice error message at boot when it fails
    match store::Store::connect().await {
        Ok(_) => info!("DB connected successfully"),
        Err(e) => {
            error!("Database couldn't be reached");
            error!("{e}");
            error!("use `export DATABASE_URL=sqlite:my_db_file.db` to set the environment variable, and make sure the file exists");
            info!("Note that when running in release mode, this program will always use `sqlite:modkit.db` as the DB URL");
            error!("No DB connection, so I'll run anyway without recording anything.");
        }
    }

    let args: Vec<String> = std::env::args().collect();

    let ws_clients: server::Clients = Arc::new(Mutex::new(HashMap::new()));

    if let Some(arg1) = args.get(1) {
        match arg1.as_str() {
            "watchdog" => {
                info!("You provided the argument `watchdog`, I'll only run the watchdog client");
                watchdog::watch(&ws_clients).await?;
            }
            "ws" => {
                info!("You provided the argument `ws`, I'll only run the WebSocket server");
                server::run(&ws_clients).await;
            }
            _ => info!("You provided an argument (`{arg1}`), but I don't know that argument"),
        }
        return Ok(());
    }

    info!("No valid arguments provided, running both the WebSocket server and the watchdog client");
    tokio::join!(server::run(&ws_clients), watchdog::watch(&ws_clients)).1?;

    Ok(())
}
