use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Sender, channel};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MessageData {
    from: String,
    message: String,
}

type SharedUsers = Arc<Mutex<HashMap<SocketAddr, String>>>;

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
    online_users: SharedUsers,
) -> Result<(), Box<dyn Error + Send + Sync>> {

    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(text) {
                                match ws_msg.message_type {
                                    MsgTypes::Register => {
                                        let username = ws_msg.data.unwrap_or_default();
                                        println!("User terdaftar: {} dari {:?}", username, addr);
                                        
                                        let mut users = online_users.lock().unwrap();
                                        users.insert(addr, username);
                                        
                                        let all_users: Vec<String> = users.values().cloned().collect();
                                        
                                        let response = WebSocketMessage {
                                            message_type: MsgTypes::Users,
                                            data_array: Some(all_users),
                                            data: None,
                                        };
                                        let _ = bcast_tx.send(serde_json::to_string(&response).unwrap());
                                    }
                                    MsgTypes::Message => {
                                        let chat_content = ws_msg.data.unwrap_or_default();
                                        
                                        let sender_name = {
                                            let users = online_users.lock().unwrap();
                                            users.get(&addr).cloned().unwrap_or_else(|| "Anonymous".to_string())
                                        };

                                        let inner_data = MessageData {
                                            from: sender_name,
                                            message: chat_content,
                                        };

                                        let response = WebSocketMessage {
                                            message_type: MsgTypes::Message,
                                            data_array: None,
                                            data: Some(serde_json::to_string(&inner_data).unwrap()),
                                        };
                                        
                                        let _ = bcast_tx.send(serde_json::to_string(&response).unwrap());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => break, 
                }
            }

            msg = bcast_rx.recv() => {
                match msg {
                    Ok(text) => {
                        ws_stream.send(Message::text(text)).await?;
                    }
                    Err(_) => break,
                }
            }
        }
    }

    {
        let mut users = online_users.lock().unwrap();
        if users.remove(&addr).is_some() {
            let all_users: Vec<String> = users.values().cloned().collect();
            let response = WebSocketMessage {
                message_type: MsgTypes::Users,
                data_array: Some(all_users),
                data: None,
            };
            let _ = bcast_tx.send(serde_json::to_string(&response).unwrap());
            println!("Connection closed for {:?}", addr);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(32);
    let online_users: SharedUsers = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening on port 8080 (Rust Server for YewChat Activated!)");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {addr:?}");
        let bcast_tx = bcast_tx.clone();
        let online_users = online_users.clone();
        
        tokio::spawn(async move {
            match ServerBuilder::new().accept(socket).await {
                Ok((_req, ws_stream)) => {
                    if let Err(e) = handle_connection(addr, ws_stream, bcast_tx, online_users).await {
                        println!("Error handling connection for {:?}: {:?}", addr, e);
                    }
                }
                Err(e) => {
                    println!("WebSocket handshake failed for {:?}: {:?}", addr, e);
                }
            }
        });
    }
}