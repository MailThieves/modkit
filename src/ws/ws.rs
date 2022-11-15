use std::collections::HashMap;
use std::sync::Arc;

use futures::{FutureExt, StreamExt};
use log::*;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

use crate::drivers::contact_sensor::ContactSensor;
use crate::drivers::device::{Bundle, DeviceType, Device};
use crate::ws::event::{Event, EventKind};

pub(crate) type Clients = Arc<Mutex<HashMap<String, Client>>>;

#[derive(Clone, Debug)]
pub struct Client {
    pub client_id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

/// Connects a websocket client
pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            error!("error sending websocket msg: {}", e);
        }
    }));

    client.sender = Some(client_sender);
    clients.lock().await.insert(id.clone(), client.clone());

    info!("{} connected", id);

    // For each message
    while let Some(result) = client_ws_rcv.next().await {
        // Retrieve the message
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                error!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };

        // Call the handler and get the response
        let response = handle_message(msg).await.to_msg();

        // If the client is still connected, send the response
        let c = clients.lock().await;
        match c.get(&id) {
            Some(client) => {
                if let Some(sender) = &client.sender {
                    sender.send(Ok(response)).unwrap();
                }
            }
            None => {
                error!("Couldn't find client registered with that id: {}", id);
                return;
            }
        };
    }

    clients.lock().await.remove(&id);
    info!("{} disconnected", id);
}

/// handles an incoming Event through the websocket
/// and returns a response Event
async fn handle_message(msg: Message) -> Event {
    let msg = match msg.to_str() {
        // Capture the msg if we can get one
        Ok(m) => m,
        // Otherwise, return a message containing the error
        Err(e) => {
            error!("Couldn't convert websocket message to string: {:?}", msg);
            return Event::new(
                EventKind::Error,
                None,
                Some(Bundle::error(&format!("{:?}", e))),
            );
        }
    };

    info!("Got a message from the client: {:?}", msg);

    let event: Event = match serde_json::from_str(&msg) {
        // If we can get an event from the message, do so
        Ok(event) => event,
        // Otherwise return an error event
        Err(e) => {
            error!("Error: couldn't deserialize the message into an event. You likely provided a bad message.");
            return Event::error(&format!("Bad message: {}", e));
        }
    };

    // Only certain kinds of events are "incoming events"
    match event.kind() {
        EventKind::HealthCheck => return Event::new(EventKind::HealthCheck, None, None),
        EventKind::PollDevice => {
            // If they send a PollDevice event, make sure they provided a DeviceType
            if let Some(dev_type) = event.device_type() {
                // If so, return a PollDeviceResult with the data bundle from that device
                match poll_device(&dev_type) {
                    Ok(bundle) => return Event::new(EventKind::PollDeviceResult, Some((*dev_type).clone()), Some(bundle)),
                    // If we encounter an error, return an error bundle
                    Err(e) => return Event::error(&e)
                }
            } else {
                // Otherwise return an error
                return Event::error(r#"Please provide a device type to poll ("device": "Camera" for example)"#);
            }
        }
        // If it's not an incoming Event, return an error Event
        _ => return wrong_way()
    }
}

fn wrong_way() -> Event {
    Event::error(&format!(
        "this Event type should only be sent from the server, not the from the client"
    ))
}

fn poll_device(dev_type: &DeviceType) -> Result<Bundle, String> {
    let bundle = match dev_type {
        DeviceType::ContactSensor => {
            let sensor = ContactSensor::new("Door Sensor", 0, "sensor.txt");
            sensor.poll()
        },
        DeviceType::Camera => {
            // get camera stuff here
            Ok(Bundle::Camera { placeholder: format!("Placeholder data") })
        },
        DeviceType::Light => {
            // Get light state
            Ok(Bundle::Light { on: true })
        },
    };

    bundle.map_err(|e| format!("{}", e) )
}