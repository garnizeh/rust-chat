use chrono::Utc;
use common::{ChatMessage, WebSocketMessage, WebSocketMessageType};
use rocket::futures::{stream::SplitSink, SinkExt, StreamExt};
use rocket::{tokio::sync::Mutex, State};
use rocket_ws::{stream::DuplexStream, Channel, Message, WebSocket};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

static USER_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Default)]
struct ChatRoom {
    connections: Mutex<HashMap<usize, ChatRoomConnection>>,
}

struct ChatRoomConnection {
    username: String,
    sink: SplitSink<DuplexStream, Message>,
}

impl ChatRoom {
    pub async fn add(
        &self,
        id: usize,
        sink: SplitSink<DuplexStream, Message>,
    ) -> Option<ChatMessage> {
        let mut conns = self.connections.lock().await;
        let username = format!("User #{}", id);
        let conn = ChatRoomConnection {
            username: username.clone(),
            sink: sink,
        };
        conns.insert(id, conn);

        Some(ChatMessage {
            message: format!("{} entered in this chat room", username),
            author: "system".to_string(),
            created_at: Utc::now().naive_utc(),
        })
    }

    pub async fn remove(&self, id: usize) -> Option<ChatMessage> {
        let mut conns = self.connections.lock().await;
        if let Some(conn) = conns.get_mut(&id) {
            let username = conn.username.clone();
            conns.remove(&id);

            Some(ChatMessage {
                message: format!("{} left this chat room", username),
                author: "system".to_string(),
                created_at: Utc::now().naive_utc(),
            })
        } else {
            None
        }
    }

    pub async fn send_username(&self, id: usize) {
        let mut conns = self.connections.lock().await;
        if let Some(conn) = conns.get_mut(&id) {
            let websocket_message = WebSocketMessage {
                message_type: WebSocketMessageType::UsernameChange,
                message: None,
                users: None,
                username: Some(conn.username.clone()),
            };

            let _ = conn
                .sink
                .send(Message::Text(json!(websocket_message).to_string()))
                .await;
        }
    }

    pub async fn broadcast_message(&self, chat_message: ChatMessage) {
        let websocket_message = WebSocketMessage {
            message_type: WebSocketMessageType::NewMessage,
            message: Some(chat_message),
            users: None,
            username: None,
        };

        let mut conns = self.connections.lock().await;
        for (_id, conn) in conns.iter_mut() {
            let _ = conn
                .sink
                .send(Message::Text(json!(websocket_message).to_string()))
                .await;
        }
    }

    pub async fn change_username(&self, id: usize, username: String) -> Option<ChatMessage> {
        let mut conns = self.connections.lock().await;
        if let Some(conn) = conns.get_mut(&id) {
            let old_username = conn.username.clone();
            let new_username = username.clone();
            conn.username = username;

            Some(ChatMessage {
                message: format!("{} changed username to {}", old_username, new_username),
                author: "system".to_string(),
                created_at: Utc::now().naive_utc(),
            })
        } else {
            None
        }
    }

    pub async fn broadcast_user_list(&self) {
        let mut conns = self.connections.lock().await;
        let mut users = vec![];
        for (_id, conn) in conns.iter() {
            users.push(conn.username.clone());
        }

        let websocket_message = WebSocketMessage {
            message_type: WebSocketMessageType::UsersList,
            message: None,
            users: Some(users),
            username: None,
        };

        for (_id, conn) in conns.iter_mut() {
            let _ = conn
                .sink
                .send(Message::Text(json!(websocket_message).to_string()))
                .await;
        }
    }
}

async fn handle_ws_message(message_contents: Message, state: &State<ChatRoom>, id: usize) {
    match message_contents {
        Message::Text(json) => {
            if let Ok(ws_message) = serde_json::from_str::<WebSocketMessage>(&json) {
                match ws_message.message_type {
                    WebSocketMessageType::NewMessage => {
                        if let Some(system_message) = ws_message.message {
                            state.broadcast_message(system_message).await;
                        }
                    }
                    WebSocketMessageType::UsersList => {}
                    WebSocketMessageType::UsernameChange => {
                        if let Some(username) = ws_message.username {
                            if let Some(change_message) = state.change_username(id, username).await
                            {
                                state.send_username(id).await;
                                state.broadcast_user_list().await;
                                state.broadcast_message(change_message).await;
                            }
                        }
                    }
                }
            }
        }
        Message::Binary(_) => {}
        Message::Frame(_) => {}
        Message::Ping(_) => {}
        Message::Pong(_) => {}
        Message::Close(_) => {}
    }
}

#[rocket::get("/")]
fn chat<'r>(ws: WebSocket, state: &'r State<ChatRoom>) -> Channel<'r> {
    ws.channel(move |stream| {
        Box::pin(async move {
            let id = USER_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
            let (ws_sink, mut ws_stream) = stream.split();
            if let Some(system_message) = state.add(id, ws_sink).await {
                state.broadcast_message(system_message).await;
                state.broadcast_user_list().await;
                state.send_username(id).await;
            }

            while let Some(message) = ws_stream.next().await {
                if let Ok(message_contents) = message {
                    handle_ws_message(message_contents, state, id).await;
                }
            }

            if let Some(system_message) = state.remove(id).await {
                state.broadcast_user_list().await;
                state.broadcast_message(system_message).await;
            }

            Ok(())
        })
    })
}

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .mount("/", rocket::routes![chat])
        .manage(ChatRoom::default())
        .launch()
        .await;
}
