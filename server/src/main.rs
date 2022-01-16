use std::collections::LinkedList;
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use warp::{ws::Message, Filter, Rejection};
use serde::{Serialize, Deserialize};
mod ws_handlers;
mod ws;

pub const MAX_MESSAGES: usize = 20000;

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

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Line {
    pub last_x: f32,
    pub last_y: f32,
    pub new_x: f32,
    pub new_y: f32,
}

type Clients = Arc<Mutex<HashMap<String, Client>>>;
type Result<T> = std::result::Result<T, Rejection>;
type Lines = Arc<Mutex<LinkedList<Line>>>;

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let lines: Lines = Arc::new(Mutex::new(LinkedList::new()));

    let index_route = warp::path::end()
                    .and(warp::fs::file("./index.html"));

    let js_route = warp::path("index.js")
                    .and(warp::fs::file("./index.js"));

    let js_wasm_drawing =  warp::path!("pkg" / "drawing_wasm.js")
                    .and(warp::fs::file("pkg/drawing_wasm.js"));

    let js_wasm_drawing_bg =  warp::path!("pkg" / "drawing_wasm_bg.wasm")
                    .and(warp::fs::file("pkg/drawing_wasm_bg.wasm"));

    let ws_route = warp::path("ws")
                    .and(warp::ws())
                    .and(with_clients(clients.clone()))
                    .and(with_lines(lines.clone()))
                    .and_then(ws_handlers::ws_handler);

    let routes = warp::get().and(
        index_route
        .or(js_route)
        .or(ws_route)
        .or(js_wasm_drawing)
        .or(js_wasm_drawing_bg)
    ).with(warp::cors().allow_any_origin());

    println!("Starting server");
    warp::serve(routes).run(([0, 0, 0, 0], 80)).await;

}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients, ), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

fn with_lines(lines: Lines) -> impl Filter<Extract = (Lines, ), Error = Infallible> + Clone {
    warp::any().map(move || lines.clone())
}
