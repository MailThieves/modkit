/// The operating logic for the WebSocket, ie. this is what the WebSocket can do
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use futures::{FutureExt, StreamExt};
use log::*;
use serde::{Deserialize, Serialize};
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
pub type Clients = Arc<Mutex<HashMap<String, Client>>>;

/// A single client
#[derive(Clone, Debug)]
pub struct Client {
    pub client_id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

/// Functions of the websocket
pub mod ws {
    use crate::defaults;

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
        match Store::connect().await {
            Ok(store) => {
                store
                    .write_event(event.clone())
                    .await
                    .expect("Couldn't write event to the database");
            }
            Err(e) => {
                error!(
                    "Couldn't connect to database when handling an incoming event. It was not recorded!"
                );
                error!("{e}");
            }
        }

        match event.kind() {
            EventKind::HealthCheck => handle_health_check(&event),
            EventKind::PollDevice => handle_poll_device(&mut event),
            EventKind::EventHistory => handle_event_history().await,
            EventKind::PinCheck => handle_pin_check(&event),
            EventKind::MailStatus => handle_mail_status().await,
            // We already filtered out outgoing events, so this must mean we added a new
            // type of incoming event and didn't write a handler for it
            _ => {
                error!("Need an event handler for an incoming event that we didn't plan for");
                error!(
                    "Either an outgoing event slipped through the cracks or we don't have a plan"
                );
                error!("Event Kind: {}", event.kind());
                Event::error(
                    "Unplanned incoming event or rogue outgoing event, this shouldn't happen!",
                )
            }
        }
    }

    pub fn wrong_way() -> Event {
        Event::error(&format!(
            "this Event type should only be sent from the server, not the from the client. Try another event type."
        ))
    }

    // TODO: Add tests for all the handlers

    /// When we receive a health check, just send it back.
    /// This just lets the client know that it's still connected ok!
    pub fn handle_health_check(_: &Event) -> Event {
        Event::new(EventKind::HealthCheck, None, None)
    }

    pub fn handle_pin_check(event: &Event) -> Event {
        info!("Handing pin check!");
        info!("Got event: {:?}", event);
        // There should be a Bundle, let's make sure
        if let Some(bundle) = event.data() {
            // If we can destructure it to a PinCheck bundle
            if let Bundle::PinCheck { pin } = bundle {
                // Return a PinResult bundle with a authorized bool
                return Event::new(
                    EventKind::PinResult,
                    None,
                    Some(Bundle::PinResult {
                        authorized: pin == &defaults::pin(),
                    }),
                );
            }
        }

        // If there's any error, just return an error
        return Event::error("Couldn't get modkit PIN to login!");
    }

    pub fn handle_poll_device(event: &mut Event) -> Event {
        // If they didn't provide a device type, return with an error
        let dev_type = match event.device_type().copied() {
            Some(d) => d,
            None => {
                return Event::error(
                    "Please provide a device type to poll (`Camera`, `Light`, `ContactSensor`)",
                )
            }
        };

        // Otherwise, poll the device and return the data bundle
        match event.poll_device() {
            Ok(bundle) => Event::new(EventKind::PollDeviceResult, Some(dev_type), Some(bundle)),
            // If we get a device error, then just return that error
            // wrapped in an event
            Err(e) => e.into(),
        }
    }

    pub async fn handle_event_history() -> Event {
        let db = Store::connect()
            .await
            .expect("Couldn't access the database!");
        let events = db
            .get_all_events()
            .await
            .expect("Couldn't get events from the db");

        Event::new(
            EventKind::EventHistory,
            None,
            Some(Bundle::EventHistory { events }),
        )
    }

    pub async fn handle_mail_status() -> Event {
        let db = Store::connect()
            .await
            .expect("Couldn't access the database!");
        match db.get_mail_status().await {
            Ok(event) => return event,
            Err(e) => return Event::error(&format!("{e}")),
        }
    }
}

/// Methods for starting the webserver, and handling registration
/// and connection to the websocket
pub mod http {
    use local_ip_address::linux::local_ip;

    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) struct RegisterResponse {
        url: String,
    }

    pub fn register_route(
        ws_clients: &Clients,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
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
        register_routes
    }

    pub fn ws_route(
        ws_clients: &Clients,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("ws")
            .and(warp::ws())
            .and(warp::path::param())
            .and(with_clients(ws_clients.clone()))
            .and_then(connect_client)
    }

    /// Starts up the webserver
    pub async fn run(ws_clients: &Clients) {
        info!("Running the WebSocket server");

        let routes = register_route(&ws_clients).or(ws_route(&ws_clients)).with(
            warp::cors()
                .allow_any_origin()
                .allow_headers(vec![
                    "Content-Type",
                    "Accept",
                    "Accept-Encoding",
                    "Accept-Language",
                    "Cache-Control",
                    "Connection",
                    "Host",
                    "Origin",
                    "Pragma",
                    "Referer",
                    "Sec-Fetch-Dest",
                    "Sec-Fetch-Mode",
                    "Sec-Fetch-Site",
                    "User-Agent",
                ])
                .allow_methods(vec!["GET", "OPTIONS", "POST", "DELETE"]),
        );

        warp::serve(routes).run(([0, 0, 0, 0], 3012)).await
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

        // Get our local IP so that the interface knows where to connect
        // Default to 0.0.0.0 if we're running locally
        let ip = local_ip().unwrap_or(std::net::Ipv4Addr::from([0, 0, 0, 0]).into());

        Ok(json(&RegisterResponse {
            url: format!("ws://{ip}:3012/ws/{}", uuid),
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
            Some(c) => {
                Ok(ws.on_upgrade(move |socket| spawn_client_connection(socket, id, clients, c)))
            }
            None => Err(warp::reject::not_found()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::drivers::device::DeviceType;

    use super::*;

    // Help function, just makes an empty client set
    fn clients() -> Clients {
        return Arc::new(Mutex::new(HashMap::new()));
    }

    // Helper function, gets the ws url
    // Not actually using this right now
    #[allow(unused)]
    async fn register_ws_url() -> http::RegisterResponse {
        let filter = http::register_route(&clients());
        let response = warp::test::request().path("/register").reply(&filter).await;

        // Make sure the response is ok
        assert_eq!(response.status(), 200);
        // Get the response body as a string
        let body = std::str::from_utf8(response.body()).unwrap();
        serde_json::from_str(&body).unwrap()
    }

    #[tokio::test]
    async fn test_register_route() {
        let filter = http::register_route(&clients());

        let response = warp::test::request().path("/register").reply(&filter).await;

        // Make sure the response is ok
        assert_eq!(response.status(), 200);
        // Get the response body as a string
        let body = std::str::from_utf8(response.body()).unwrap();
        // Make sure the websocket url is in it
        assert!(body.contains("ws://") || body.contains("wss://"));
    }

    // I tried to write a test for the websocket but goddamn it's complicated
    // Or i'm just a dumbass

    #[test]
    fn test_proper_health_check_response() {
        let incoming = Event::new(EventKind::HealthCheck, None, None);
        let outgoing = ws::handle_health_check(&incoming);
        assert_eq!(outgoing.kind(), &EventKind::HealthCheck);
        assert!(outgoing.timestamp() > 0);
    }

    #[test]
    fn test_handle_poll_device_response() {
        let mut incoming = Event::new(EventKind::PollDevice, Some(DeviceType::ContactSensor), None);
        let outgoing = ws::handle_poll_device(&mut incoming);
        assert_eq!(outgoing.kind(), &EventKind::PollDeviceResult);
        assert!(outgoing.data().is_some());
    }

    #[test]
    fn test_handle_pin_check_authorized() {
        let incoming = Event::new(
            EventKind::PinCheck,
            None,
            Some(Bundle::PinCheck { pin: 6245 }),
        );
        let outgoing = ws::handle_pin_check(&incoming);
        assert_eq!(outgoing.kind(), &EventKind::PinResult);
        assert!(outgoing.data().is_some());
        if let Some(Bundle::PinResult { authorized }) = outgoing.data() {
            assert!(authorized);
        } else {
            // Want to make sure we execute the above asserting in the if
            assert!(false);
        }
    }

    #[test]
    fn test_handle_pin_check_not_authorized() {
        let incoming = Event::new(
            EventKind::PinCheck,
            None,
            Some(Bundle::PinCheck { pin: 8888 }),
        );
        let outgoing = ws::handle_pin_check(&incoming);
        assert_eq!(outgoing.kind(), &EventKind::PinResult);
        assert!(outgoing.data().is_some());
        if let Some(Bundle::PinResult { authorized }) = outgoing.data() {
            assert!(!authorized);
        } else {
            // Want to make sure we execute the above asserting in the if
            assert!(false);
        }
    }

    #[tokio::test]
    async fn test_handle_event_history_when_db_empty() {
        let store = Store::connect().await.unwrap();

        store.nuke().await.unwrap();

        // let incoming = Event::new(EventKind::EventHistory, None, None);
        // This handler is called when we get an EventHistory event but we don't actually
        // need to pass it to the function since it doesn't use it.
        let outgoing = ws::handle_event_history().await;

        assert_eq!(outgoing.kind(), &EventKind::EventHistory);
        assert!(outgoing.data().is_some());

        let data = outgoing.data().unwrap();
        match data {
            Bundle::EventHistory { events } => assert!(events.is_empty()),
            _ => assert_eq!("Event history should be an empty vec, not None", ""),
        }
    }

    #[tokio::test]
    async fn test_event_history_when_db_not_empty() {
        let store = Store::connect().await.unwrap();
        store.nuke().await.unwrap();
        // Write some event
        store
            .write_event(Event::new(EventKind::HealthCheck, None, None))
            .await
            .unwrap();

        let outgoing = ws::handle_event_history().await;

        assert_eq!(outgoing.kind(), &EventKind::EventHistory);
        assert!(outgoing.data().is_some());

        let data = outgoing.data().unwrap();
        match data {
            Bundle::EventHistory { events } => assert_eq!(events.len(), 1),
            _ => assert_eq!("Events history should have 1 event", ""),
        }
    }

    #[tokio::test]
    async fn test_handle_mail_status_when_db_empty() {
        let store = Store::connect().await.unwrap();
        store.nuke().await.unwrap();

        let outgoing = ws::handle_mail_status().await;
        assert_eq!(outgoing.kind(), &EventKind::Error);
    }

    #[tokio::test]
    async fn test_handle_mail_status_when_db_not_empty() {
        let store = Store::connect().await.unwrap();
        store.nuke().await.unwrap();

        store
            .write_event(Event::new(EventKind::MailDelivered, None, None))
            .await
            .unwrap();

        let outgoing = ws::handle_mail_status().await;
        assert_eq!(outgoing.kind(), &EventKind::MailDelivered);
    }

    #[test]
    fn test_handle_wrong_way() {
        let outgoing = ws::wrong_way();
        assert_eq!(outgoing.kind(), &EventKind::Error);
    }

    // TODO: Write some test for handle_message() that create a ws::Message (from warp) as input
}
