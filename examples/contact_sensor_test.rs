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

    let pin = Gpio::new()
        .expect("GPIO connection")
        .get(18)
        .expect("Pin connection")
        .into_input();

    loop {
        info!("State = {:?}", cs.poll());
        info!("Pin is low? {}", pin.is_low());
        sleep(Duration::from_millis(200));
    }
}
