use crate::{Client, Clients, Lines, Line, DrawingMsg, MAX_MESSAGES};
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
            return;
        }
    }));

    let uuid = Uuid::new_v4().to_simple().to_string();
    let new_client = Client {
        client_id: uuid.clone(),
        sender: Some(client_sender),
    };

    let copied_lines = lines.lock().await.clone();
    read_existing_lines_for_new_client(copied_lines, &new_client).await;

    // Add new client into map
    clients.lock().await.insert(uuid.clone(), new_client);

    println!("{} connected", uuid);
    send_number_of_clients(&uuid, &clients, true).await;

    // Listen for incomming message
    while let Some(result) = client_ws_rcv.next().await {
        match result {
            Ok(msg) => {
                if msg.is_close() {
                    break;
                }

                if msg.is_text() {
                    let msg_str = msg.to_str().unwrap_or_default();
                    if msg_str.is_empty() {
                        println!("Received empty message from {}", uuid);
                        continue;
                    }

                    // Attempt to parse as generic JSON to check msg_type
                    match serde_json::from_str::<serde_json::Value>(msg_str) {
                        Ok(json_value) => {
                            if let Some(msg_type) = json_value.get("msg_type").and_then(|v| v.as_str()) {
                                if msg_type == "clear" {
                                    println!("Received clear message from {}", uuid);
                                    lines.lock().await.clear(); // Clear the history

                                    // Broadcast clear message to all clients
                                    let clear_broadcast_msg = DrawingMsg::new(None, Some(0), String::from("clear"));
                                    send_to_all(&uuid, &clear_broadcast_msg, &clients, true).await;
                                    
                                    // Also send the updated (zero) message count
                                    let messages_count_msg = DrawingMsg::new(None, Some(0), String::from("messages"));
                                    send_to_all(&uuid, &messages_count_msg, &clients, true).await;
                                    continue; // Skip further processing for this message
                                }
                            }
                            
                            // If not a "clear" message, assume it's a "line" message
                            // (This is the original logic for handling line messages)
                            match serde_json::from_str::<Line>(msg_str) {
                                Ok(line) => {
                                    lines.lock().await.push_back(line.clone());

                                    if lines.lock().await.len() > MAX_MESSAGES {
                                        lines.lock().await.clear();
                                        // Inform clients that canvas was cleared due to message limit
                                        let auto_clear_msg = DrawingMsg::new(None, Some(0), String::from("clear"));
                                        send_to_all(&uuid, &auto_clear_msg, &clients, true).await;
                                    }

                                    let drawing_msg = DrawingMsg::new(Some(line), None, String::from("line"));
                                    send_to_all(&uuid, &drawing_msg, &clients, false).await;

                                    let current_line_count = lines.lock().await.len();
                                    let drawing_msg2 = DrawingMsg::new(None,  Some(current_line_count), String::from("messages"));
                                    send_to_all(&uuid, &drawing_msg2, &clients, true).await;
                                }
                                Err(e) => {
                                    println!("Failed to parse Line from message: {:?}, error: {}", msg_str, e);
                                }
                            }
                        }
                        Err(e) => {
                            // This case handles if the message is not even valid JSON.
                            // It could be the original Line message format which doesn't have "msg_type"
                            // Try to parse as Line directly for backward compatibility or simple line messages
                            match serde_json::from_str::<Line>(msg_str) {
                                Ok(line) => {
                                    lines.lock().await.push_back(line.clone());

                                    if lines.lock().await.len() > MAX_MESSAGES {
                                        lines.lock().await.clear();
                                        let auto_clear_msg = DrawingMsg::new(None, Some(0), String::from("clear"));
                                        send_to_all(&uuid, &auto_clear_msg, &clients, true).await;
                                    }

                                    let drawing_msg = DrawingMsg::new(Some(line), None, String::from("line"));
                                    send_to_all(&uuid, &drawing_msg, &clients, false).await;
                                    
                                    let current_line_count = lines.lock().await.len();
                                    let drawing_msg2 = DrawingMsg::new(None,  Some(current_line_count), String::from("messages"));
                                    send_to_all(&uuid, &drawing_msg2, &clients, true).await;
                                }
                                Err(e_line) => {
                                     println!("Failed to parse message as JSON Value or Line: {:?}, json_err: {}, line_err: {}", msg_str, e, e_line);
                                }
                            }
                        }
                    }
                } else if msg.is_binary() {
                    println!("Received binary message from {}: not handled", uuid);
                }
            },
            Err(e) => {
                println!("error receiving message for id {}): {}", &uuid.clone(), e);
                break;
            }
        }
    }

    println!("{} discontected", uuid);
    clients.lock().await.remove(&uuid);
    send_number_of_clients(&uuid, &clients, false).await;
}

async fn send_to_all(current_uuid: &str, drawing_msg: &DrawingMsg, clients: &Clients, self_included: bool) {
    for (client_uuid, client) in &clients.lock().await.clone() {
        if self_included || current_uuid != client_uuid {
            if let Some(sender) = &client.sender {
                let text = serde_json::to_string(drawing_msg).unwrap();
                let _ = sender.send(Ok(Message::text(text)));
            }
        }
    }
}

async fn send_number_of_clients(client_uuid: &str, clients: &Clients, self_included: bool) {
    let len = clients.lock().await.len();
    let msg = DrawingMsg::new(None, Some(len), String::from("n_clients"));
    println!("n_clients: {}", len);
    send_to_all(client_uuid, &msg, clients, self_included).await;
}

async fn read_existing_lines_for_new_client(lines: LinkedList<Line>, new_client: &Client) {
    //Send what we have in lines to new client
    let mut iter = lines.iter();

    // send number of messages
    let drawing_msg = DrawingMsg::new(None, Some(lines.len()), String::from("messages"));
    let json = serde_json::to_string(&drawing_msg).unwrap();
    new_client.sender.as_ref().unwrap().send(Ok(Message::text(json))).unwrap();

    // send lines
    loop {
        let line : std::option::Option<&Line> = iter.next();

        match line {
            Some(line) => {
                let drawing_msg = DrawingMsg::new( Some(line.clone()), None, String::from("line"));
                let json = serde_json::to_string(&drawing_msg).unwrap();
                if let Some(sender) = &new_client.sender {
                    // See if can send successfully
                    sender.send(Ok(Message::text(json))).unwrap();
                }
            },
            None => break
        }
    }
}
