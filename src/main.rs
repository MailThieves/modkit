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
        .filter(Some("modkit"), LevelFilter::Info)
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    if cfg!(feature = "hardware") {
        debug!("Compiled with `--features hardware`, using hardware data");
    } else {
        warn!("You have not compiled with `--features hardware`, fake data will be returned, and no real hardware values will be read");
        warn!("Recompile with `cargo build --features hardware` to use the hardware");
    }

    // Try to connect to the DB so we get a nice error message at boot when it fails
    match store::Store::connect().await {
        Ok(_) => debug!("DB connected successfully"),
        Err(e) => {
            error!("Database couldn't be reached");
            error!("{e}");
            error!("No DB connection, so I'll run anyway without recording anything.");
        }
    }

    let ws_clients: server::Clients = Arc::new(Mutex::new(HashMap::new()));

    tokio::join!(server::run(&ws_clients), watchdog::watch(&ws_clients)).1?;

    Ok(())
}
