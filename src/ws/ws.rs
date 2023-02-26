/// The operating logic for the WebSocket, ie. this is what the WebSocket can do
use std::collections::HashMap;
use std::sync::Arc;

use futures::{FutureExt, StreamExt};
use log::*;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

use crate::drivers::contact_sensor::ContactSensor;
use crate::drivers::device::{Device, DeviceType};
use crate::model::*;
use crate::store::Store;

pub(crate) type Clients = Arc<Mutex<HashMap<String, Client>>>;

#[derive(Clone, Debug)]
pub struct Client {
    pub client_id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

// Sends an event to every client in the clients list
pub async fn send_to_clients(event: &Event, clients: &Clients) {
    let lock = clients.lock().await;
    for (id, client) in lock.iter() {
        info!("Sending to client {id}");
        if let Some(sender) = &client.sender {
            sender
                .send(Ok(event.clone().to_msg()))
                .expect("Couldn't send message to client");
        }
    }
    drop(lock);
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

    // Filter out outgoing events; they shouldn't be allowed
    if event.kind().is_outgoing() {
        return wrong_way();
    }

    // Write it to the DB if possible, but prefer to just skip recording it
    // rather than crash
    if let Ok(store) = Store::connect().await {
        store
            .write_event(event.clone())
            .await
            .expect("Couldn't write event to the database");
    } else {
        error!(
            "Couldn't connect to database when handling an incoming event. It was not recorded!"
        );
    }

    match event.kind() {
        EventKind::HealthCheck => return Event::new(EventKind::HealthCheck, None, None),
        EventKind::PollDevice => {
            // If they send a PollDevice event, make sure they provided a DeviceType
            if let Some(dev_type) = event.device_type() {
                // If so, return a PollDeviceResult with the data bundle from that device
                match poll_device(&dev_type) {
                    Ok(bundle) => {
                        return Event::new(
                            EventKind::PollDeviceResult,
                            Some((*dev_type).clone()),
                            Some(bundle),
                        )
                    }
                    // If we encounter an error, return an error bundle
                    Err(e) => return Event::error(&e),
                }
            } else {
                // Otherwise return an error
                return Event::error(
                    r#"Please provide a device type to poll ("device": "Camera" for example)"#,
                );
            }
        }
        // We already filtered out outgoing events, so this must mean we added a new
        // type of incoming event and didn't write a handler for it
        _ => panic!("Incoming event with no flight plan. This shouldn't happen."),
    }
}

fn wrong_way() -> Event {
    Event::error(&format!(
        "this Event type should only be sent from the server, not the from the client. Try another event type."
    ))
}

fn poll_device(dev_type: &DeviceType) -> Result<Bundle, String> {
    let bundle = match dev_type {
        DeviceType::ContactSensor => {
            let sensor = ContactSensor::new("Door Sensor", 0, "sensor.txt");
            sensor.poll()
        }
        DeviceType::Camera => {
            // get camera stuff here
            Ok(Bundle::Camera {
                placeholder: format!("Placeholder data"),
            })
        }
        DeviceType::Light => {
            // Get light state
            Ok(Bundle::Light { on: true })
        }
    };

    bundle.map_err(|e| format!("{}", e))
}
