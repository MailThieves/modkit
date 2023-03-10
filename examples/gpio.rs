use log::LevelFilter;
use log::*;
use std::time::Duration;
use std::thread::sleep;

use rppal::gpio::Gpio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new()
        .format_timestamp(None)
        .filter(Some("gpio"), LevelFilter::Info)
        .init();

    info!("GPIO Starting!");

    let gpio = Gpio::new()?;
    let door_sensor = gpio.get(23)?.into_input();

    info!("found pin {door_sensor:?}");

    info!("Pin #{}", door_sensor.pin());
    info!("");
    
    loop {
        info!("Pin is {}", door_sensor.read());
        info!("Pin is low? {}", door_sensor.is_low());
        info!("Pin is high? {}", door_sensor.is_high());
        info!("");

        sleep(Duration::from_secs(1));
    }

}