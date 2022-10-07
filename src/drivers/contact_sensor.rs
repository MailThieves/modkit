use std::fs::File;
use std::io::Read;

use crate::drivers::Result;
use crate::drivers::device::{Device, Bundle};

#[derive(Debug)]
pub struct ContactSensor {
    name: String,
    addr: u8, 
    port: String   
}

impl ContactSensor {
    /// Creates a new ContactSensor
    pub fn new(name: &str, addr: u8, port: &str) -> Self {
        ContactSensor {
            name: String::from(name),
            addr,
            port: String::from(port),
        }
    }
}

impl Device for ContactSensor {
    /// Returns the given name for the sensor
    fn name(&self) -> &str {
        &self.name
    }

    /// Returns Ok() if the device is connected, Err(e) otherwise
    fn connected(&self) -> Result<()> {
        Ok(())
    }

    /// Returns a Bundle of data, in this case just { open: bool }, wrapped in a result
    fn poll(&self) -> Result<super::device::Bundle> {
        // This is temporary, I'm using the contents of a file to simulate the switch
        // since I don't have a physical switch yet
        let mut buffer = String::new();
        File::open(&self.port)
            .unwrap()
            .read_to_string(&mut buffer)
            .unwrap();
        let state: bool = if buffer.trim() == String::from("1") { true } else { false };
        
        Ok(Bundle::ContactSensor { open: state })
    }

    // Calls poll and return Ok(true), Ok(false), or Err(e)
    fn is_active(&self) -> Result<bool> {
        match self.poll() {
            Ok(Bundle::ContactSensor { open }) => return Ok(open),
            Err(e) => return Err(e),
        }
    }

    /// What the do when the watcher determines the device is activated
    fn on_activate(&self) -> Result<()> {
        println!("===== Contact Sensor got activated!!! =====");
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn cs() -> ContactSensor {
        ContactSensor::new("Door Sensor", 0x01, "./sensor.txt")
    }

    // 1 for open, 0 for closed
    fn set_door(open: &str) {
        let mut file = File::create("./sensor.txt").unwrap();
        file.write_all(open.as_bytes()).unwrap();
    }

    #[test]
    fn test_contact_sensor_connection() {
        let cs = ContactSensor::new("Door Sensor", 0x01, "./sensor.txt");
        assert_eq!(cs.name(), "Door Sensor");
        assert_eq!(cs.connected(), Ok(()));
    }

    #[test]
    fn test_contact_sensor_poll() {
        let cs = cs();
        assert_eq!(cs.connected(), Ok(()));
        assert!(cs.poll().is_ok());
    }

    #[test]
    fn test_door_is_open() {
        set_door("1");
        let cs = cs();
        
        let res = cs.poll();
        assert!(res.is_ok());
        let bundle = res.unwrap();
        assert_eq!(bundle, Bundle::ContactSensor { open: true });

        set_door("0");
        let res2 = cs.poll();
        assert!(res2.is_ok());
        let bundle2 = res2.unwrap();
        assert_eq!(bundle2, Bundle::ContactSensor { open: false });
    }

    #[test]
    fn test_contact_sensor_is_active() {
        let cs = cs();
        set_door("0");
        assert_eq!(cs.is_active(), Ok(false));
        set_door("1");
        assert_eq!(cs.is_active(), Ok(true));
    }
}