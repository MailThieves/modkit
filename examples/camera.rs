#![allow(unused)]
use log::*;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use modkit::prelude::*;

fn main() {
    env_logger::init();
    error!("Be sure to use info log level");
    info!("Starting camera example");

    info!("Running the raspistill command");

    let output = Command::new("raspistill")
        .args(["-o", "test.jpg"])
        .output();

    match output {
        Ok(out) => {
            error!("{:#?}", out.stderr);
            info!("{:#?}", out.stdout);
        },
        Err(e) => error!("couldn't use the raspistill command. Are you on the pi? {e}")
    }

}
