use crate::{Client, Clients, Lines};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{WebSocket};

pub async fn client_connection(ws: WebSocket, clients: Clients, lines: Lines) {
    println!("establishing client connection... {:?}", ws);

    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            println!("error sending websocket msg: {}", e);
        }
    }));

    let uuid = Uuid::new_v4().to_simple().to_string();
    let new_client = Client {
        client_id: uuid.clone(),
        sender: Some(client_sender),
    };

    clients.lock().await.insert(uuid.clone(), new_client);
    //TODO: Send what we in lines to new client

    while let Some(result) = client_ws_rcv.next().await {
        match result {
            Ok(msg) => {
                //TODO: Insert into lines
                send_to_all(msg, &clients).await;
            },
            Err(e) => {
                println!("error receiving message for id {}): {}", &uuid.clone(), e);
                break;
            }
        }
    }

    clients.lock().await.remove(&uuid);
    println!("{} disconnected", uuid);
}

async fn send_to_all(msg: warp::ws::Message, clients: &Clients) {
    for (_, client) in &clients.lock().await.clone() {
        if let Some(sender) = &client.sender {
            // println!("send to {}: {:?}", client_id, msg);
            let _ = sender.send(Ok(msg.clone()));
        }
    }
}
