use thiserror::Error;

pub mod device;
pub mod contact_sensor;
pub mod watcher;

#[derive(Error, Debug, PartialEq)]
pub enum DeviceError {
    #[error("No device `{0}` connected")]
    NoConnection(String)
}


/// A custom error type using the DeviceError defined above
pub type Result<T> = std::result::Result<T, DeviceError>;
