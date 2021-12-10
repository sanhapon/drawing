// https://tms-dev-blog.com/build-basic-rust-websocket-server/
// https://tms-dev-blog.com/warp-data-update-loop-easy-how-to/

use std::collections::LinkedList;
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use warp::{ws::Message, Filter, Rejection};
use serde::{Serialize, Deserialize};
mod ws_handlers;
mod ws;

pub const MAX_MESSAGES: usize = 2000;

#[derive(Debug, Clone)]
pub struct Client {
    pub client_id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingMsg  {
    line: Option<Line>,
    messages: Option<usize>,
    msg_type: String ,
}

impl DrawingMsg {
    fn new(line: Option<Line>, messages: Option<usize>, msg_type: String) -> DrawingMsg {
        DrawingMsg {
            line: line,
            messages: messages,
            msg_type:msg_type
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub last_x: u16,
    pub last_y: u16,
    pub new_x: u16,
    pub new_y: u16,
}

type Clients = Arc<Mutex<HashMap<String, Client>>>;
type Result<T> = std::result::Result<T, Rejection>;
type Lines = Arc<Mutex<LinkedList<Line>>>;

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let lines: Lines = Arc::new(Mutex::new(LinkedList::new()));

    let index_route =warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("./index.html"));

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and(with_lines(lines.clone()))
        .and_then(ws_handlers::ws_handler);

    let routes = ws_route
                    .with(warp::cors().allow_any_origin())
                    .or(index_route);

    println!("Starting server");
    warp::serve(routes).run(([0, 0, 0, 0], 80)).await;

}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients, ), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

fn with_lines(lines: Lines) -> impl Filter<Extract = (Lines, ), Error = Infallible> + Clone {
    warp::any().map(move || lines.clone())
}
