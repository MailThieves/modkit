#![allow(dead_code)]

mod drivers;
mod api;

use drivers::{contact_sensor::ContactSensor};

use crate::drivers::device::Device;


fn main() {
    // let devices = vec![ContactSensor::new("Door sensor", 0x01, "./sensor.txt")];
    // drivers::watcher::watch(&devices);
    let cs = ContactSensor::new("Door Sensor", 0x00, "sensor.txt");
    println!("{}", cs.poll().unwrap());
}
