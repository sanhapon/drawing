use crate::{Client, Clients, Lines, Line};
use std::collections::LinkedList;
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

    let the_lines = lines.lock().await.clone();
    read_existing_lines_for_new_client(&the_lines, &new_client).await;

    // Add new client into map
    clients.lock().await.insert(uuid.clone(), new_client);

    // Listen for incomming message
    while let Some(result) = client_ws_rcv.next().await {
        match result {
            Ok(msg) => {
                let message = match msg.to_str() {
                    Ok(v) => v,
                    Err(e) => {
                        println!("got error --> {:?}", e);
                        return;
                    }
                };

                println!("message ==> {} ", message);

                let json_option = serde_json::from_str(message);

                // Check if we can deserialize json
                match json_option {
                    Ok(line) => lines.lock().await.push_back(line),
                    Err(e) => println!("error --> {}", e)
                }

                send_to_all(&uuid, &msg, &clients).await;
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
                let _ = sender.send(Ok(msg.clone()));
            }
        }
    }
}

async fn read_existing_lines_for_new_client(lines: &LinkedList<Line>, new_client: &Client) {
    //Send what we in lines to new client
    let mut iter = lines.iter();

    loop {
        let line = iter.next();
        match line {
            Some(line) => {
                let line_json = serde_json::to_string(line).unwrap();
                if let Some(sender) = &new_client.sender {
                    // See if can send successfully
                    sender.send(Ok(Message::text(line_json))).unwrap();
                }
            },
            None => break
        }
    }
}
