use log::*;
use modkit::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

fn init_logging() {
    // env_logger::Builder::new()
    //     .format_timestamp(None)
    //     .filter(Some("modkit"), LevelFilter::Info)
    //     .init();
    env_logger::init();
}

fn warn_env_variables() {
    use std::env::var;
    info!("Some environment variables need to be set");
    warn!("The following environment variables are possible, these are their current values:");
    warn!(
        "\tMODKIT_IMG_DIR = \t{:?} \t\t(recommended `./img`)",
        var("MODKIT_IMG_DIR")
    );
    warn!(
        "\tDATABASE_URL = \t\t{:?} \t\t(recommended `sqlite:modkit.db`)",
        var("DATABASE_URL")
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    warn_env_variables();

    if hardware_enabled() {
        info!("Hardware enabled");
    } else {
        warn!("Hardware disabled. This means that either (a) you're not running on the raspberry pi or (b) the GPIO is unavailable");
    }

    // Try to connect to the DB so we get a nice error message at boot when it fails
    match Store::connect().await {
        Ok(_) => info!("DB connected successfully"),
        Err(e) => {
            error!("Database couldn't be reached");
            error!("{e}");
        }
    }

    let ws_clients: server::Clients = Arc::new(Mutex::new(HashMap::new()));

    tokio::join!(server::run(&ws_clients), watchdog::watch(&ws_clients)).1?;

    Ok(())
}
