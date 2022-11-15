//! An event passed through websockets
use serde::{Serialize, Deserialize};
use warp::ws::Message;

use crate::drivers::device::{Bundle, DeviceType};

/// The kind of event being sent
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum EventKind {
    // Incoming events
    HealthCheck,
    PollDevice,
    // Outgoing events
    MailDelivered,
    MailPickedUp,
    DoorOpened,
    PollDeviceResult,
    Error
}

/// An Event struct, that can be sent to or recieved from a websocket client
#[derive(Debug, Serialize, Deserialize)]
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
    data: Option<Bundle>
}

impl Event {
    pub fn new(kind: EventKind, device: Option<DeviceType>, data: Option<Bundle>) -> Self {
        Self {
            kind,
            timestamp: chrono::Local::now().to_string(),
            device,
            data
        }
    }

    pub fn error(msg: &str) -> Self {
        Self::new(EventKind::Error, None, Some(
            Bundle::Error {
                msg: msg.to_string()
            }
        ))
    }

    pub fn kind(&self) -> &EventKind {
        &self.kind
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
}