use std::fmt;

use serde::{Serialize};
use chrono::Utc;

use crate::drivers::Result;


/// A bundle of data. This could take multiple formats, depending on which device the data is taken from.
#[derive(Debug, PartialEq, Serialize)]
pub enum Bundle {
    /// The data from a contact sensor. Just open or closed.
    ContactSensor {
        open: bool
    },
}

impl Bundle {
    // Bundles should be able to:
    //      1. go to json format for the web api
    //      2. Be written with a timestamp to a format on the local box
    fn to_json(&self) -> std::result::Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl fmt::Display for Bundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\t", Utc::now().format("%Y-%m-%d %H:%M:%S")).expect("Couldn't write output to buffer");
        match self {
            Self::ContactSensor { open } => {
                return writeln!(f, "ContactSensor({})", open);
            }
        }
    }
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
            Bundle::ContactSensor { open: is_opened } => assert_eq!(is_opened, true)
        }
    }

}