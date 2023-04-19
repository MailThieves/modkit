#![allow(unused)]
use log::*;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use modkit::prelude::*;

fn main() {
    env_logger::init();
    info!("Starting camera example");

    info!("Running the raspistill command");
    info!("Set MODKIT_IMG_DIR env variable to override default img location");

    println!("{:?}", camera::capture_still());

    info!("Capturing video");
    println!("{:?}", camera::capture_video());
    info!("done");
}
