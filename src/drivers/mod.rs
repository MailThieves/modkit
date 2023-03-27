use thiserror::Error;
use rppal::gpio;

use self::device::DeviceType;

use crate::model::Event;

pub mod device;
pub mod contact_sensor;

#[allow(unused)]
#[derive(Error, Debug, PartialEq)]
pub enum DeviceError {
    #[error("No device `{0}` connected")]
    NoConnection(String),
    #[error("Communication error: {0}")]
    CommunicationError(String),
    #[error("You provided `{0:?}` which is not a valid device type")]
    DeviceNotFound(Option<DeviceType>),
    #[error("GPIO Error: {0}")]
    GpioError(String)
}

/// gpio::Error doesn't implement PartialEq, so it can't be automatically
/// converted. I'll open a pull request.
impl From<gpio::Error> for DeviceError {
    fn from(error: gpio::Error) -> Self {
        Self::GpioError(format!("{error}"))
    }
}

impl Into<Event> for DeviceError {
    fn into(self) -> Event {
        Event::error(&format!("{}", self))
    }
}

/// A custom error type using the DeviceError defined above
pub type Result<T> = std::result::Result<T, DeviceError>;
