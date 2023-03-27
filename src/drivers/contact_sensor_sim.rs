use std::fs::File;
use std::io::Read;

use log::*;

use crate::drivers::Result;
use crate::drivers::device::Device;
use crate::model::Bundle;

#[derive(Debug)]
pub struct ContactSensorSim {
    name: String,
    /// Storing the state can be useful when watching for changes
    state: Option<Bundle>
}

impl ContactSensorSim {
    /// Creates a new ContactSensor
    pub fn new(name: &str) -> Self {
        ContactSensorSim {
            name: String::from(name),
            state: None
        }
    }
}

impl ContactSensorSim {
    pub fn changed(&mut self) -> Result<bool> {
        let new = self.poll()?;

        // If there is no old state, or is the old and new state is different
        if self.state.is_none() || *self.state.as_ref().unwrap() != new {
            // Then update the old state 
            self.state = Some(new);
            // and return that there was a change
            return Ok(true);
        }

        // Otherwise return false
        Ok(false)
    }

    pub fn state(&self) -> Option<&Bundle> {
        self.state.as_ref()
    }
}

impl Device for ContactSensorSim {
    /// Returns the given name for the sensor
    fn name(&self) -> &str {
        &self.name
    }

    /// Returns Ok() if the device is connected, Err(e) otherwise
    fn connected(&self) -> Result<()> {
        Ok(())
    }

    /// Returns a Bundle of data, in this case just { open: bool }, wrapped in a result
    fn poll(&self) -> Result<Bundle> {
        // This is temporary, I'm using the contents of a file to simulate the switch
        // since I don't have a physical switch yet
        trace!("Trying to read a 1 or 0 from ./sensor.txt (temporary placeholder until we get the hardware)");
        let mut buffer = String::new();
        File::open("./sensor.txt")
            .unwrap()
            .read_to_string(&mut buffer)
            .unwrap();
        let state: bool = if buffer.trim() == String::from("1") { true } else { false };
        
        Ok(Bundle::ContactSensor { open: state })
    }

    // Calls `poll()` and return Ok(true), Ok(false), or Err(e)
    fn is_active(&self) -> Result<bool> {
        match self.poll() {
            Ok(Bundle::ContactSensor { open }) => return Ok(open),
            Err(e) => return Err(e),
            // other types of data bundles will never be returned
            _ => panic!()
        }
    }

    /// What to do when the watcher determines the device is activated
    fn on_activate(&self) -> Result<()> {
        info!("===== Contact Sensor is activated!!! =====");
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn cs() -> ContactSensorSim {
        ContactSensorSim::new("Door Sensor")
    }

    // 1 for open, 0 for closed
    fn set_door(open: &str) {
        let mut file = File::create("./sensor.txt").unwrap();
        file.write_all(open.as_bytes()).unwrap();
    }

    #[test]
    fn test_contact_sensor_connection() {
        let cs = ContactSensorSim::new("Door Sensor");
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

    #[test]
    fn test_changed() {
        set_door("0");
        let mut cs = cs();

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