//! An event passed through websockets
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sqlx::{Row, FromRow};
use sqlx::sqlite::SqliteRow;
use warp::ws::Message;

use crate::drivers::device::DeviceType;
use crate::model::Bundle;
use crate::store::StoreError;

/// The kind of event being sent
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum EventKind {
    // Incoming events
    HealthCheck,
    PollDevice,
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
            // Outgoing events
            Self::MailDelivered => true,
            Self::MailPickedUp => true,
            Self::DoorOpened => true,
            Self::PollDeviceResult => true,
            Self::Error => true,
            // Incoming events
            Self::HealthCheck => false,
            Self::PollDevice => false, // note that i'm not using _ as a catch all; don't want to accidentally miss a
                                       // new event type that may be outgoing
        }
    }

    pub fn is_incoming(&self) -> bool {
        !self.is_outgoing()
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    /// The event type
    kind: EventKind,
    /// Timestamp of event creation
    ///
    /// This is typically only created by the ws server, not the client
    #[serde(skip_deserializing)]
    timestamp: String,
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
            data
        })
    }
}

impl Event {
    pub fn new(kind: EventKind, device: Option<DeviceType>, data: Option<Bundle>) -> Self {
        Self {
            kind,
            timestamp: chrono::Local::now().to_string(),
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

    pub fn timestamp(&self) -> &str {
        &self.timestamp
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event_strings() -> Vec<String> {
        vec![
            r#"{"kind":"HealthCheck","timestamp":"timestamp goes here"}"#.to_string(),
            r#"{"kind":"MailDelivered","timestamp":"timestamp goes here"}"#.to_string(),
            r#"{"kind":"MailPickedUp","timestamp":"timestamp goes here"}"#.to_string(),
            r#"{"kind":"DoorOpened","timestamp":"timestamp goes here"}"#.to_string(),
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

        assert!(EventKind::PollDevice.is_incoming());
        assert!(EventKind::HealthCheck.is_incoming());
    }

    #[test]
    fn test_event_is_incoming_outgoing() {
        assert!(Event::new(EventKind::DoorOpened, None, None)
            .kind()
            .is_outgoing());

        assert!(Event::new(EventKind::PollDevice, None, None)
            .kind()
            .is_incoming());
    }
}
