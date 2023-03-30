#![allow(dead_code)]
pub mod drivers;
pub mod model;
pub mod store;
pub mod server;
pub mod watchdog;

pub mod prelude {
    pub use crate::drivers::{
        DeviceError,
        device::DeviceType,
        contact_sensor::ContactSensor,
        light::light,
        camera::camera,
        hardware_enabled
    };
    pub use crate::watchdog;
    pub use crate::server;
    pub use crate::store::Store;
}
