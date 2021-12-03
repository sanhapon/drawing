// https://tms-dev-blog.com/build-basic-rust-websocket-server/
// https://tms-dev-blog.com/warp-data-update-loop-easy-how-to/

use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use warp::{ws::Message, Filter, Rejection};
mod handlers;
mod ws;

#[derive(Debug, Clone)]
pub struct Client {
    pub client_id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

type Clients = Arc<Mutex<HashMap<String, Client>>>;
type Result<T> = std::result::Result<T, Rejection>;

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    println!("Configuring websocket route");

    let index_route =warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("./index.html"));

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and_then(handlers::ws_handler);


    let routes = ws_route.with(warp::cors().allow_any_origin())
                    .or(index_route);

    println!("Starting server");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;

}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients, ), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}
