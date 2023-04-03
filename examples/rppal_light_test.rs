use log::*;
use rppal::gpio::Gpio;
use std::time::Duration;
use std::thread::sleep;

const LIGHT_GPIO_PIN: u8 = 21;

fn main() {
    let mut light_pin = Gpio::new()
        .unwrap()
        .get(LIGHT_GPIO_PIN)
        .unwrap()
        .into_output();

    info!("light pin is low? {}", light_pin.is_set_low());
    light_pin.set_high();
    sleep(Duration::from_millis(200));
    info!("light pin is low? {}", light_pin.is_set_low());
    light_pin.set_low();
    sleep(Duration::from_millis(200));
    info!("light pin is low? {}", light_pin.is_set_low());
    light_pin.set_high();
    sleep(Duration::from_millis(200));
    info!("light pin is low? {}", light_pin.is_set_low());
    light_pin.set_low();
    sleep(Duration::from_millis(200));
    info!("light pin is low? {}", light_pin.is_set_low());
    light_pin.set_high();
    sleep(Duration::from_millis(200));
    info!("light pin is low? {}", light_pin.is_set_low());
    light_pin.set_low();
    sleep(Duration::from_millis(200));
}
