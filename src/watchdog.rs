use std::thread::sleep;
use std::time::Duration;
use log::*;

use crate::drivers::device::Device;

pub fn watch<T>(devices: &Vec<T>) where T: Device + std::fmt::Debug {
    info!("Watching the following devices");
    for dev in devices {
        info!("{:?}", dev);
    }
    loop {
        for dev in devices {
            if dev.is_active().unwrap_or(false) {
                if let Err(e) = dev.on_activate() {
                    error!("{}", e);
                }
            }
        }
        sleep(Duration::from_secs(1));
    }
}
