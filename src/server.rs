/// The operating logic for the WebSocket, ie. this is what the WebSocket can do
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use futures::{FutureExt, StreamExt};
use log::*;
use serde::Serialize;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};
use warp::Filter;
use warp::{hyper::StatusCode, reply::json, Rejection, Reply};

use crate::model::*;
use crate::store::Store;

pub use http::*;
pub use ws::*;

/// A list of clients
pub(crate) type Clients = Arc<Mutex<HashMap<String, Client>>>;

/// A single client
#[derive(Clone, Debug)]
pub struct Client {
    pub client_id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

/// Functions of the websocket
pub mod ws {
    use super::*;

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

    /// handles an incoming Event through the websocket
    /// and returns a response Event
    pub async fn handle_message(msg: Message) -> Event {

        // Capture the msg if we can get one
        let msg = match msg.to_str() {
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

        // Parse a Message to an Event
        let mut event: Event = match serde_json::from_str(&msg) {
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

        // Populate a timestamp so that incoming events have one when saving
        // to the database
        event.populate_timestamp();

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

        // TODO: Make a handler function for each kind of event kind
        match event.kind() {
            EventKind::HealthCheck => handle_health_check(&event),
            EventKind::PollDevice => handle_poll_device(&event),
            // We already filtered out outgoing events, so this must mean we added a new
            // type of incoming event and didn't write a handler for it
            _ => {
                error!("Need an event handler for an incoming event that we didn't plan for");
                error!("Either an outgoing event slipped through the cracks or we don't have a plan");
                error!("Event Kind: {}", event.kind());
                Event::error("Unplanned incoming event or rogue outgoing event, this shouldn't happen!")
            }
        }
    }

    fn wrong_way() -> Event {
        Event::error(&format!(
            "this Event type should only be sent from the server, not the from the client. Try another event type."
        ))
    }

    /// When we receive a health check, just send it back.
    /// This just lets the client know that it's still connected ok!
    fn handle_health_check(_: &Event) -> Event {
        Event::new(EventKind::HealthCheck, None, None)
    }

    fn handle_poll_device(event: &Event) -> Event {
        // If they didn't provide a device type, return with an error
        let dev_type = match event.device_type() {
            Some(d) => d,
            None => return Event::error("Please provide a device type to poll (`Camera`, `Light`, `ContactSensor`)")
        };

        // Otherwise, poll the device and return the data bundle
        match event.poll_device() {
            Ok(bundle) => {
                Event::new(
                    EventKind::PollDeviceResult,
                    Some((*dev_type).clone()),
                    Some(bundle),
                )
            },
            // If we get a device error, then just return that error
            // wrapped in an event
            Err(e) => e.into()
        }
    }
}

/// Methods for starting the webserver, and handling registration
/// and connection to the websocket
pub mod http {
    use super::*;

    #[derive(Debug, Serialize)]
    struct RegisterResponse {
        url: String,
    }

    /// Starts up the webserver
    pub async fn run(ws_clients: &Clients) {
        info!("Running the WebSocket server");

        let register = warp::path("register");
        let register_routes = register
            .and(warp::get())
            .and(with_clients(ws_clients.clone()))
            .and_then(register_handler)
            .or(register
                .and(warp::delete())
                .and(warp::path::param())
                .and(with_clients(ws_clients.clone()))
                .and_then(unregister_handler));

        let ws_routes = warp::path("ws")
            .and(warp::ws())
            .and(warp::path::param())
            .and(with_clients(ws_clients.clone()))
            .and_then(connect_client);

        let routes = register_routes
            .or(ws_routes)
            .with(warp::cors().allow_any_origin());

        warp::serve(routes).run(([127, 0, 0, 1], 3012)).await
    }

    // Attaches Clients to a warp route
    pub(crate) fn with_clients(
        clients: Clients,
    ) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
        warp::any().map(move || clients.clone())
    }

    // Register a new client and return the ws address with the client id in it
    pub async fn register_handler(clients: Clients) -> Result<impl Reply, Rejection> {
        let uuid = Uuid::new_v4().simple().to_string();
        register_client(uuid.clone(), clients.clone()).await;
        info!("Just registered a client with id: {}", uuid);
        info!("All clients: {:#?}", clients);
        Ok(json(&RegisterResponse {
            url: format!("ws://127.0.0.1:3012/ws/{}", uuid),
        }))
    }

    // Registers a client, adding them to the client list
    pub async fn register_client(uuid: String, clients: Clients) {
        clients.lock().await.insert(
            uuid.clone(),
            Client {
                client_id: uuid,
                sender: None,
            },
        );
    }

    /// Connects a websocket client
    pub async fn spawn_client_connection(
        ws: WebSocket,
        id: String,
        clients: Clients,
        mut client: Client,
    ) {
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

    // Delete a client from the client list
    pub async fn unregister_handler(id: String, clients: Clients) -> Result<impl Reply, Rejection> {
        clients.lock().await.remove(&id);
        Ok(StatusCode::OK)
    }

    // Attempt to find a client from the list, and if so connect a websocket
    pub async fn connect_client(
        ws: warp::ws::Ws,
        id: String,
        clients: Clients,
    ) -> Result<impl Reply, Rejection> {
        let client = clients.lock().await.get(&id).cloned();
        match client {
            Some(c) => Ok(ws.on_upgrade(move |socket| spawn_client_connection(socket, id, clients, c))),
            None => Err(warp::reject::not_found()),
        }
    }
}
