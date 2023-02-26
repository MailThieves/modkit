use std::fmt;

use chrono::Utc;
use serde::{Serialize, Deserialize};

// TODO: Maybe convert bundle to a Trait? we could have 3 separate implementations
// although maybe that's a stupid idea. You wouldn't know at compile time which bundle you're working
// with.

/// A bundle of data. This could take multiple formats, depending on which device the data is taken from.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Bundle {
    /// The data from a contact sensor. Just open or closed.
    ContactSensor {
        open: bool,
    },
    Error {
        msg: String,
    },
    Camera {
        placeholder: String,
    },
    Light {
        on: bool,
    },
}

impl Bundle {
    // Bundles should be able to:
    //      1. go to json format for the web api
    //      2. Be written with a timestamp to a format on the local box
    fn to_json(&self) -> std::result::Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn error(msg: &str) -> Self {
        Self::Error {
            msg: String::from(msg),
        }
    }
}

impl fmt::Display for Bundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\t", Utc::now().format("%Y-%m-%d %H:%M:%S"))
            .expect("Couldn't write output to buffer");
        match self {
            Self::ContactSensor { open } => {
                return writeln!(f, "ContactSensor({})", open);
            }
            Self::Camera { placeholder } => return writeln!(f, "Camera({placeholder})"),
            Self::Light { on } => return writeln!(f, "Light(on: {on})"),
            Self::Error { msg } => return writeln!(f, "Error({msg})"),
        }
    }
}