use std::collections::HashMap;
use std::sync::Arc;

use futures::{FutureExt, StreamExt};
use log::*;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

use crate::drivers::device::Bundle;
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
            return Event::new(EventKind::Error, Some(Bundle::error(&format!("{:?}", e))));
        }
    };

    info!("Got a message from the client: {:?}", msg);

    let event: Event = match serde_json::from_str(&msg) {
        // If we can get an event from the message, do so
        Ok(event) => event,
        // Otherwise return an error event
        Err(e) => {
            error!("Error: couldn't deserialize the message into an event. You likely provided a bad message.");
            return Event::new(
                EventKind::Error,
                Some(Bundle::error(&format!("Bad message: {}", e))),
            );
        }
    };

    match event.kind() {
        EventKind::HealthCheck => return Event::new(EventKind::HealthCheck, None),
        EventKind::DoorOpened => return wrong_way(),
        EventKind::MailDelivered => return wrong_way(),
        EventKind::MailPickedUp => return wrong_way(),
        EventKind::PollDevice => return Event::new(EventKind::PollDeviceResult, Some(Bundle::ContactSensor { open: true })),
        EventKind::PollDeviceResult => return wrong_way(),
        EventKind::Error => return wrong_way(),
    }
}

fn wrong_way() -> Event {
    Event::new(
        EventKind::Error,
        Some(Bundle::Error {
            msg: format!("this Event type should only be sent from the server, not the from the client")
        })
    )
}