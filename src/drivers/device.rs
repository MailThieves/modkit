use serde::{Deserialize, Serialize};

use crate::drivers::Result;
use crate::model::Bundle;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum DeviceType {
    Camera,
    Light,
    ContactSensor,
}

/// A Device trait, which all devices should implement
pub trait Device {
    fn name(&self) -> &str;
    /// Tests the connection status of the device
    fn connected(&self) -> Result<()>;
    /// Polls the device for a data bundle
    fn poll(&self) -> Result<Bundle>;
    // returns Ok(true) if the device is active
    fn is_active(&self) -> Result<bool>;
    /// A function to be called when the device is activated, for example when the contact sensor opens.
    fn on_activate(&self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_sensor_bundle() {
        let bundle = Bundle::ContactSensor { open: true };
        match bundle {
            Bundle::ContactSensor { open: is_opened } => assert_eq!(is_opened, true),
            _ => assert!(false)
        }
    }
}
