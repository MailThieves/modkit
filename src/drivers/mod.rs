use thiserror::Error;

use self::device::DeviceType;

use crate::model::Event;

pub mod device;
pub mod contact_sensor_sim;

#[allow(unused)]
#[derive(Error, Debug, PartialEq)]
pub enum DeviceError {
    #[error("No device `{0}` connected")]
    NoConnection(String),
    #[error("Communication error: {0}")]
    CommunicationError(String),
    #[error("You provided `{0:?}` which is not a valid device type")]
    DeviceNotFound(Option<DeviceType>)
}

impl Into<Event> for DeviceError {
    fn into(self) -> Event {
        Event::error(&format!("{}", self))
    }
}

/// A custom error type using the DeviceError defined above
pub type Result<T> = std::result::Result<T, DeviceError>;
