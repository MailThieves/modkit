use std::thread::sleep;
use std::time::Duration;

use crate::drivers::device::Device;

pub fn watch<T>(devices: &Vec<T>) where T: Device + std::fmt::Debug {
    println!("Watching the following devices");
    for dev in devices {
        println!("{:?}", dev);
    }
    loop {
        for dev in devices {
            if dev.is_active().unwrap_or(false) {
                if let Err(e) = dev.on_activate() {
                    eprintln!("{}", e);
                }
            }
        }
        sleep(Duration::from_millis(1_000));
    }
}
