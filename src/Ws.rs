use crate::{Client, Clients, Lines, Line};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{WebSocket, Message};

pub async fn client_connection(ws: WebSocket, clients: Clients, lines: Lines) {
    println!("establishing client connection... {:?}", ws);

    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| async {
        if let Err(e) = result {
            println!("error sending websocket msg: {}", e);
        }
    }));

    // println!("clients -->{}", clients.lock().await.len());

    let uuid = Uuid::new_v4().to_simple().to_string();
    let new_client = Client {
        client_id: uuid.clone(),
        sender: Some(client_sender),
    };

    //Send what we in lines to new client
    let the_copy = lines.lock().await.clone();
    let mut iter = the_copy.iter();

    loop {
        let line = iter.next();
        match line {
            Some(line) => {
                let line_json = serde_json::to_string(line).unwrap();
                if let Some(sender) = &new_client.sender {
                    sender.send(Ok(Message::text(line_json))).unwrap();
                }
            },
            None => break
        }
    }

    // Add new client into map
    clients.lock().await.insert(uuid.clone(), new_client);

    // Listen for incomming line
    while let Some(result) = client_ws_rcv.next().await {
        match result {
            Ok(msg) => {
                send_to_all(&uuid, &msg, &clients).await;

                let message = match msg.to_str() {
                    Ok(v) => v,
                    Err(_) => return,
                };

                let line: Line = serde_json::from_str(message).unwrap();
                lines.lock().await.push_back(line);

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

async fn send_to_all(current_uuid: &str, msg: &warp::ws::Message, clients: &Clients) {
    for (client_uuid, client) in &clients.lock().await.clone() {
        if current_uuid != client_uuid {
            if let Some(sender) = &client.sender {
                println!("send to {}: {:?}", client_uuid, msg);
                let _ = sender.send(Ok(msg.clone()));
            }
        }
    }
}
