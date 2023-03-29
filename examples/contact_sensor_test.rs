#![allow(unused)]

use log::*;
use modkit::prelude::*;
use rppal::gpio::Gpio;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    env_logger::init();

    let cs = ContactSensor::new();

    info!("connected to contact sensor");
    info!("Using pin BCM 18");

    loop {
        info!("pin is low? {:?}", cs.poll());
        sleep(Duration::from_millis(400));
    }
}
