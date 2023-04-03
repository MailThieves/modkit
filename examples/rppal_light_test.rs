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

    light_pin.set_high();
    sleep(Duration::from_millis(200));
    light_pin.set_low();
    sleep(Duration::from_millis(200));
    light_pin.set_high();
    sleep(Duration::from_millis(200));
    light_pin.set_low();
    sleep(Duration::from_millis(200));
    light_pin.set_high();
    sleep(Duration::from_millis(200));
    light_pin.set_low();
    sleep(Duration::from_millis(200));
}
