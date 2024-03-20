use chrono::Utc;
use common::{ChatMessage, WebSocketMessage, WebSocketMessageType};
use serde_json::json;
use yew::prelude::*;
use yew_hooks::use_websocket;

use crate::message_list::MessageList;
use crate::send_dialog::SendDialog;
use crate::user_list::UserList;

mod message_list;
mod send_dialog;
mod user_list;

#[function_component]
fn App() -> Html {
    let messages_handler = use_state(Vec::default);
    let messages = (*messages_handler).clone();
    let users_handler = use_state(Vec::default);
    let users = (*users_handler).clone();
    let username_handler = use_state(String::default);
    let username = (*username_handler).clone();

    let ws = use_websocket("ws://127.0.0.1:8000".to_string());

    let mut cloned_messages = messages.clone();
    use_effect_with(ws.message.clone(), move |ws_message| {
        if let Some(ws_msg) = &**ws_message {
            let websocket_message: WebSocketMessage = serde_json::from_str(&ws_msg).unwrap();
            match websocket_message.message_type {
                WebSocketMessageType::NewMessage => {
                    let msg = websocket_message.message.expect("missing message payload");
                    cloned_messages.push(msg);
                    messages_handler.set(cloned_messages);
                }
                WebSocketMessageType::UsersList => {
                    let users = websocket_message.users.expect("missing users payload");
                    users_handler.set(users);
                }
                WebSocketMessageType::UsernameChange => {
                    let username = websocket_message
                        .username
                        .expect("missing username payload");
                    username_handler.set(username);
                }
            }
        }
    });

    let cloned_username = username.clone();
    let cloned_ws = ws.clone();
    let send_message_callback = Callback::from(move |message: String| {
        let chat_message = ChatMessage {
            author: cloned_username.clone(),
            message: message,
            created_at: Utc::now().naive_utc(),
        };
        let ws_message = WebSocketMessage {
            message_type: WebSocketMessageType::NewMessage,
            message: Some(chat_message),
            username: None,
            users: None,
        };

        cloned_ws.send(json!(ws_message).to_string());
    });

    let cloned_ws = ws.clone();
    let change_username_callback = Callback::from(move |username: String| {
        let ws_message = WebSocketMessage {
            message_type: WebSocketMessageType::UsernameChange,
            message: None,
            username: Some(username),
            users: None,
        };

        cloned_ws.send(json!(ws_message).to_string());
    });

    html! {
        <div class="container-fluid">
            <div class="row">
                <div class="col-3">
                    <UserList users={users} />
                </div>
                <div class="col-9">
                    <MessageList messages={messages}  />
                </div>
            </div>
            <div class="row mt-3">
                if username.len() > 0 {
                    <SendDialog
                        change_username_callback={change_username_callback}
                        send_message_callback={send_message_callback}
                        username={username}
                    />
                }
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
