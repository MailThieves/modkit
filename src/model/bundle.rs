use std::fmt;

use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, Row};

use crate::store::StoreError;

use super::Event;

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
        file_name: String,
    },
    Light {
        on: bool,
    },
    EventHistory {
        events: Vec<Event>
    }
}

impl Bundle {
    // Bundles should be able to:
    //      1. go to json format for the web api
    //      2. Be written with a timestamp to a format on the local box
    pub fn to_json(&self) -> std::result::Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn error(msg: &str) -> Self {
        Self::Error {
            msg: String::from(msg),
        }
    }
}

impl<'r> sqlx::FromRow<'r, SqliteRow> for Bundle {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let json_data = row.try_get("data")?;
        let bundle: Bundle = serde_json::from_str(json_data)
            // This is a bit messy but it works for now
            .map_err(|e|
                StoreError::DecodeError(format!("{e}")).into_sqlx_decode_error()
            )?;
        Ok(bundle)
    }
}

impl fmt::Display for Bundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\t", chrono::Local::now().to_string())
            .expect("Couldn't write output to buffer");
        match self {
            Self::ContactSensor { open } => {
                return write!(f, "ContactSensor({})", open);
            }
            Self::Camera { file_name } => return write!(f, "Camera({file_name})"),
            Self::Light { on } => return write!(f, "Light(on: {on})"),
            Self::Error { msg } => return write!(f, "Error({msg})"),
            Self::EventHistory { events } => {
                // This is a little bit fucked but oh well
                for e in events {
                    write!(f, "{:?}", e).unwrap();
                }
                write!(f, "")
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_sensor_bundle() {
        let bundle = Bundle::ContactSensor { open: true };
        match bundle {
            Bundle::ContactSensor { open: is_opened } => assert_eq!(is_opened, true),
            _ => assert!(false),
        }
    }
}
