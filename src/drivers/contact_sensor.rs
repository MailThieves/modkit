use cfg_if::cfg_if;
use log::*;

use crate::drivers::Result;

cfg_if! {
    if #[cfg(feature = "hardware")] {
        // Hardware enabled imports
        use rppal::gpio::Gpio;

        const CONTACT_SENSOR_GPIO_PIN: u8 = 18;
    } else {
        // Hardware disabled imports
        use std::{fs::File, io::Read};

    }
}

#[derive(Debug)]
// true = door open
pub struct ContactSensor(bool);

impl ContactSensor {
    /// Creates a new ContactSensor
    pub fn new() -> Self {
        ContactSensor(false)
    }

    pub fn is_open(&self) -> bool {
        self.0
    }

    pub fn changed(&mut self) -> Result<bool> {
        let new = self.poll()?;

        // if the old and new state are different
        if self.0 != new {
            // Then update the old state
            self.0 = new;
            // and return that there was a change
            return Ok(true);
        }

        // Otherwise return false
        Ok(false)
    }


    #[cfg(not(feature = "hardware"))]
    /// Returns Ok(true) is the door is open.
    /// 
    /// This is the non-hardware version.
    pub fn poll(&self) -> Result<bool> {
        trace!("Trying to read a 1 or 0 from ./sensor.txt (temporary placeholder until we get the hardware)");
        let mut buffer = String::new();
        File::open("./sensor.txt")
            .unwrap()
            .read_to_string(&mut buffer)
            .unwrap();
        Ok(buffer.trim() == String::from("1"))
    }

    #[cfg(feature = "hardware")]
    pub fn poll(&self) -> Result<bool> {
        let pin = Gpio::new()?
            .get(CONTACT_SENSOR_GPIO_PIN)?
            .into_input_pullup();
        // low = 0 = closed
        Ok(pin.is_high())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    // 1 for open, 0 for closed
    fn set_door(open: &str) {
        let mut file = File::create("./sensor.txt").unwrap();
        file.write_all(open.as_bytes()).unwrap();
    }

    #[test]
    fn test_contact_sensor_poll() {
        let cs = ContactSensor::new();
        assert!(cs.poll().is_ok());
    }

    #[test]
    fn test_door_is_open() {
        set_door("1");
        let cs = ContactSensor::new();

        let res = cs.poll();
        assert!(res.is_ok());
        assert!(res.unwrap(), true);

        set_door("0");
        let res2 = cs.poll();
        assert!(res2.is_ok());
        assert!(res2.unwrap(), false);
    }

    #[test]
    fn test_changed() {
        set_door("0");
        let mut cs = ContactSensor::new();

        assert_eq!(cs.changed(), Ok(true));
        assert_eq!(cs.changed(), Ok(false));
        assert_eq!(cs.changed(), Ok(false));

        set_door("1");
        assert_eq!(cs.changed(), Ok(true));
        assert_eq!(cs.changed(), Ok(false));

        set_door("0");
        assert_eq!(cs.changed(), Ok(true));
        assert_eq!(cs.changed(), Ok(false));
    }
}
