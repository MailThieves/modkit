#![allow(unused)]

use log::*;
use modkit::prelude::*;
use rppal::gpio::Gpio;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    env_logger::init();

    let cs = ContactSensor::new();
    error!("Remember to use RUST_LOG=info");

    info!("Contact sensor connected: {:?}", cs.connected());

    loop {
        info!("State = {:?}", cs.poll());
        sleep(Duration::from_millis(200));
    }
}
