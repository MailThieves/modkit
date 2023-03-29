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

    camera::capture_into("./temp_img");

}
