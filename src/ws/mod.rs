/// The logistics of the web server and WebSocket server
use log::*;

use warp::Filter;

pub mod ws;
pub use ws::send_to_clients;

/// Starts up the webserver
pub async fn run(ws_clients: &ws::Clients) {
    info!("Running the WebSocket server");

    let register = warp::path("register");
    let register_routes = register
        .and(warp::get())
        .and(handlers::with_clients(ws_clients.clone()))
        .and_then(handlers::register_handler)
        .or(register
            .and(warp::delete())
            .and(warp::path::param())
            .and(handlers::with_clients(ws_clients.clone()))
            .and_then(handlers::unregister_handler));

    let ws_routes = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(handlers::with_clients(ws_clients.clone()))
        .and_then(handlers::ws_handler);

    let routes = register_routes
        .or(ws_routes)
        .with(warp::cors().allow_any_origin());

    
    warp::serve(routes).run(([127, 0, 0, 1], 3012)).await
}


mod handlers {
    use std::convert::Infallible;
    use serde::Serialize;

    use crate::ws::ws::{Clients, Client, client_connection};
    use warp::{Filter, Rejection, Reply, reply::json, hyper::StatusCode};
    use uuid::Uuid;
    use log::*;

    #[derive(Debug, Serialize)]
    struct RegisterResponse {
        url: String
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

    // Delete a client from the client list
    pub async fn unregister_handler(id: String, clients: Clients) -> Result<impl Reply, Rejection> {
        clients.lock().await.remove(&id);
        Ok(StatusCode::OK)
    }



    // Attempt to find a client from the list, and if so connect a websocket
    pub async fn ws_handler(ws: warp::ws::Ws, id: String, clients: Clients) -> Result<impl Reply, Rejection> {
        let client = clients.lock().await.get(&id).cloned();
        match client {
            Some(c) => Ok(ws.on_upgrade(move |socket| client_connection(socket, id, clients, c))),
            None => Err(warp::reject::not_found()),
        }
    }

}