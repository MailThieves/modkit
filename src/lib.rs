#![allow(dead_code)]
pub mod drivers;
pub mod model;
pub mod store;

pub mod prelude {
    pub use crate::drivers::device::DeviceType;
    pub use crate::drivers::DeviceError;
    pub use crate::drivers::contact_sensor::ContactSensor;
    pub use crate::drivers::light::light;
    pub use crate::drivers::camera::camera;
}

// Only used in main()
// mod server;
// mod watchdog;

