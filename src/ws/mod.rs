use log::*;
use warp::Filter;
use std::sync::Arc;
use std::collections::HashMap;

use tokio::sync::Mutex;


pub mod event;
pub mod ws;
mod handlers;

pub async fn run() {
    info!("Running the WebSocket server");

    let ws_clients: ws::Clients = Arc::new(Mutex::new(HashMap::new()));

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


    // crate::watchdog::watch(&mut ws_clients).await;

    warp::serve(routes).run(([127, 0, 0, 1], 3012)).await;
}
