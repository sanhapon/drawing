use crate::{ws, Clients, Lines, Result};
use warp::Reply;

pub async fn ws_handler(ws: warp::ws::Ws, clients: Clients, lines: Lines) -> Result<impl Reply> {
    println!("ws_handler");

    Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, clients, lines)))
}
