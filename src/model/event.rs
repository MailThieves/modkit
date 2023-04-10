//! An event passed through websockets
use std::fmt::Display;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row};
use warp::ws::Message;

use crate::drivers::contact_sensor::ContactSensor;
use crate::drivers::device::DeviceType;
use crate::drivers::{camera::camera, light::light, DeviceError};
use crate::model::Bundle;
use crate::store::StoreError;

/// The kind of event being sent
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum EventKind {
    // Incoming events
    HealthCheck,
    PollDevice,
    EventHistory,
    MailStatus,
    // Outgoing events
    MailDelivered,
    MailPickedUp,
    DoorOpened,
    PollDeviceResult,
    Error,
}

impl EventKind {
    pub fn is_outgoing(&self) -> bool {
        match self {
            // Incoming events
            Self::HealthCheck => false,
            Self::PollDevice => false,
            Self::EventHistory => false,
            Self::MailStatus => false,
            // Outgoing events
            Self::MailDelivered => true,
            Self::MailPickedUp => true,
            Self::DoorOpened => true,
            Self::PollDeviceResult => true,
            Self::Error => true,
            // note that i'm not using _ as a catch all; don't want to accidentally miss a
            // new event type that may be outgoing
        }
    }
}

impl Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'r> sqlx::FromRow<'r, SqliteRow> for EventKind {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let kind = match row.try_get("kind")? {
            "HealthCheck" => EventKind::HealthCheck,
            "PollDevice" => EventKind::PollDevice,
            "EventHistory" => EventKind::EventHistory,
            "MailStatus" => EventKind::MailStatus,
            "MailDelivered" => EventKind::MailDelivered,
            "MailPickedUp" => EventKind::MailPickedUp,
            "DoorOpened" => EventKind::DoorOpened,
            "PollDeviceResult" => EventKind::PollDeviceResult,
            "Error" => EventKind::Error,
            _ => {
                return Err(
                    StoreError::DecodeError("Could not decode EventKind".to_string())
                        .into_sqlx_decode_error(),
                )
            }
        };

        Ok(kind)
    }
}

/// An Event struct, that can be sent to or recieved from a websocket client
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Event {
    /// The event type
    kind: EventKind,
    /// Timestamp of event creation
    ///
    /// This is typically only created by the ws server, not the client
    #[serde(skip_deserializing)]
    timestamp: u32,
    /// Which device this event references, if any
    device: Option<DeviceType>,
    /// The optional data bundle being sent
    data: Option<Bundle>,
}

impl<'r> FromRow<'r, SqliteRow> for Event {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let kind = EventKind::from_row(&row)?;
        let timestamp = row.try_get("timestamp")?;
        let device = DeviceType::from_row(&row).ok();
        let data = Bundle::from_row(&row).ok();

        Ok(Event {
            kind,
            timestamp,
            device,
            data,
        })
    }
}

impl Event {
    pub fn new(kind: EventKind, device: Option<DeviceType>, data: Option<Bundle>) -> Self {
        // This is a little messy but ok
        let timestamp: u32 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        Self {
            kind,
            timestamp,
            device,
            data,
        }
    }

    pub fn error(msg: &str) -> Self {
        Self::new(
            EventKind::Error,
            None,
            Some(Bundle::Error {
                msg: msg.to_string(),
            }),
        )
    }

    pub fn kind(&self) -> &EventKind {
        &self.kind
    }

    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }

    pub fn device_type(&self) -> Option<&DeviceType> {
        self.device.as_ref()
    }

    pub fn data(&self) -> Option<&Bundle> {
        self.data.as_ref()
    }

    pub fn to_msg(self) -> Message {
        Message::text(serde_json::to_string(&self).unwrap())
    }

    // Incoming events are never expected to have a timestamp.
    // This will generate one. It's called after an Event is deserialized
    // from a WS message
    pub fn populate_timestamp(&mut self) {
        // TODO: Extract this format string to a crate-wide const? It's used in bundle printing
        self.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
    }

    // Finds the device type associated with this event, and poll that device,
    // returning a data bundle and setting that data bundle to itself
    pub fn poll_device(&mut self) -> Result<Bundle, DeviceError> {
        // Returning a String error is kind of ugly here but it's fine for now
        if self.device.is_none() {
            return Err(DeviceError::DeviceNotFound(self.device.clone()));
        }

        let bundle = match self.device.as_ref().unwrap() {
            DeviceType::ContactSensor => {
                let sensor = ContactSensor::new();
                Bundle::ContactSensor {
                    open: sensor.poll()?,
                }
            }
            DeviceType::Camera => {
                let file_path = camera::capture_still()?;
                Bundle::Camera {
                    file_name: format!("{:?}", file_path.file_name().expect("image file name")),
                }
            }
            DeviceType::Light => {
                // Get light state
                Bundle::Light {
                    on: light::is_on()?,
                }
            }
        };

        self.data = Some(bundle.clone());
        Ok(bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event_strings() -> Vec<String> {
        vec![
            r#"{"kind":"HealthCheck","timestamp":12345}"#.to_string(),
            r#"{"kind":"MailDelivered","timestamp":12345}"#.to_string(),
            r#"{"kind":"MailPickedUp","timestamp":12345}"#.to_string(),
            r#"{"kind":"DoorOpened","timestamp":12345}"#.to_string(),
        ]
    }

    #[test]
    fn test_build_event() {
        let event = Event::new(EventKind::HealthCheck, Some(DeviceType::Camera), None);
        assert_eq!(event.kind(), &EventKind::HealthCheck);
        assert_eq!(event.device_type().unwrap(), &DeviceType::Camera);
        assert_eq!(event.data(), None);
    }

    #[test]
    fn test_de_serialize() {
        let events = event_strings();
        for e_string in events {
            assert!(serde_json::from_str::<Event>(&e_string).is_ok());
        }
    }

    #[test]
    fn test_event_is_kind_incoming_outgoing() {
        assert!(EventKind::MailDelivered.is_outgoing());
        assert!(EventKind::MailPickedUp.is_outgoing());
        assert!(EventKind::Error.is_outgoing());
        assert!(EventKind::DoorOpened.is_outgoing());
        assert!(EventKind::PollDeviceResult.is_outgoing());

        assert!(!EventKind::PollDevice.is_outgoing());
        assert!(!EventKind::HealthCheck.is_outgoing());
    }

    #[test]
    fn test_event_is_incoming_outgoing() {
        assert!(Event::new(EventKind::DoorOpened, None, None)
            .kind()
            .is_outgoing());

        assert!(!Event::new(EventKind::PollDevice, None, None)
            .kind()
            .is_outgoing());
    }
}
