#![allow(dead_code)]

mod drivers;
mod api;

use drivers::{contact_sensor::ContactSensor, device::Device};


fn main() {
    let devices = vec![ContactSensor::new("Door sensor", 0x01, "./sensor.txt")];
    drivers::watcher::watch(&devices);
}
