
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use gpui::{App, AppContext, AsyncApp, Context, Entity, EventEmitter, Global,};
use log::info;

use reqwest_client::runtime;

use serde_json::json;
use tokio::sync::mpsc;

use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite:: {Message, Utf8Bytes};
use crate::component::login::LoginResponseMsg;

#[derive(Debug, Clone)]
pub struct WsTextMessage(pub String);

#[derive(Clone)]
pub struct State {
    pub http_server: String,
    pub ws_server: String,
    pub user_state: LoginResponseMsg,
    pub tokio_handle:tokio::runtime::Handle,
    pub dial_window_is_open: bool,
}

#[derive(Clone)]
pub enum EventBus {
    WebSocketText(String),
    ChangeMessageGroupSelectIndex(usize),
}

impl EventEmitter<EventBus> for State {}
pub struct GlobalState(pub(crate) Entity<State>);
impl Global for GlobalState {}



pub fn new_state(cx:&mut App){
    let state_entity = cx.new(|cx_for_state| { State::new(cx_for_state) });
    cx.set_global(GlobalState(state_entity));
}


impl State {
    
    pub fn new(cx: &mut Context<Self>) -> Self {
        let tokio_runtime_handle = tokio::runtime::Handle::try_current().unwrap_or_else(|_| {
            log::debug!("no tokio runtime found");
            runtime().handle().clone()
        });

        State{
            http_server: String::from("http://127.0.0.1:34332"),
            ws_server: String::from("ws://127.0.0.1:34332"),
            user_state: Default::default(),
            tokio_handle: tokio_runtime_handle,
            dial_window_is_open:false,
        }
    }

    pub fn init_ws(&mut self, cx: &mut Context<Self>){
        info!("websocket init");
        let mut async_cx = cx.to_async().clone();
        let entity = cx.entity().clone();

        let url = self.ws_server.as_str().to_owned() +  "/register_ws";
        let user_id = self.user_state.clone().user_id;
        let user_token = self.user_state.clone().user_token;
        let tokio_handler = self.tokio_handle.clone();
        let (tx, mut rx) = mpsc::unbounded_channel::<WsTextMessage>();
        
        cx.spawn(|_, _: &mut AsyncApp| async move {
            while let Some(ws_msg) = rx.recv().await {
                _ = entity.update(&mut async_cx, |_, cx| {
                    cx.emit(EventBus::WebSocketText(ws_msg.0));
                });
            }
        }).detach();
        
        cx.spawn(|_, _: &mut AsyncApp| async move {

            tokio_handler.spawn(async move {

                loop {
                    println!("Connecting to WebSocket server...");
                    match connect_async(url.clone()).await {
                        Ok((ws_stream, _)) => {
                            println!("Connected!");
                            let (mut write, mut read) = ws_stream.split();

                            let payload = json!({
                                "id": user_id,
                                "token": user_token
                            });
                            if let Err(e) = write.send(Message::Text(Utf8Bytes::from(payload.to_string()))).await {
                                println!("Failed to send auth message: {}", e);
                                continue;
                            }


                            while let Some(msg) = read.next().await {
                                match msg {
                                    Ok(Message::Text(text)) => {
                                        if tx.send(WsTextMessage(text.to_string())).is_err() {
                                            println!("Channel closed, exiting WebSocket loop.");
                                            break;
                                        }
                                    }
                                    Ok(Message::Binary(data)) => {
                                        println!("Received binary data ({} bytes)", data.len());
                                    }
                                    Ok(Message::Close(frame)) => {
                                        println!("Server closed connection: {:?}", frame);
                                        break;
                                    }
                                    Ok(Message::Ping(data)) => {
                                        let _ = write.send(Message::Pong(data)).await;
                                    }
                                    Ok(Message::Pong(msg)) => {
                                        println!("{:?}", msg)
                                    }
                                    Err(e) => {
                                        eprintln!("Read error: {}", e);
                                        break;
                                    }
                                    _ => {}
                                }
                            }

                            println!("Connection lost. Reconnecting in 3 seconds...");
                        }
                        Err(e) => {
                            eprintln!("Failed to connect: {}. Retrying in 3s...", e);
                        }
                    }

                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            });

        }).detach();
    }


}