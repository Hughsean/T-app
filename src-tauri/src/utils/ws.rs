use crate::audio::audio_pipeline::AudioPipeline;
use crate::constraint::WS_URL;
use futures_util::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use serde_json::json;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;
use tokio::sync::RwLock;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

lazy_static! {
    static ref WEBSOCKET_PROTOCOL: Arc<RwLock<WebsocketProtocol>> =
        Arc::new(RwLock::new(WebsocketProtocol::new(WS_URL.to_string())));
}

// #[derive(Clone)]
pub struct WebsocketProtocol {
    websocket_url: String,
    // access_token: String,
    // client_id: String,
    // device_id: String,
    connected: Arc<Mutex<bool>>,
    hello_received: Arc<Notify>,
    sender: Option<mpsc::UnboundedSender<Message>>,
    recver: Arc<Mutex<Option<mpsc::UnboundedReceiver<Vec<u8>>>>>,
}

impl WebsocketProtocol {
    fn new(
        websocket_url: String,
        // access_token: String,
        // client_id: String,
        // device_id: String,
    ) -> Self {
        Self {
            websocket_url,
            // access_token,
            // client_id,
            // device_id,
            connected: Arc::new(Mutex::new(false)),
            hello_received: Arc::new(Notify::new()),
            sender: None,
            recver: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_instance() -> Arc<RwLock<WebsocketProtocol>> {
        WEBSOCKET_PROTOCOL.clone()
    }

    pub async fn connect(&mut self) -> Result<(), String> {
        if self.connected.lock().unwrap().clone() {
            return Ok(());
        }

        let url = self.websocket_url.clone();
        // let headers = [
        //     ("Authorization", format!("Bearer {}", self.access_token)),
        //     ("Protocol-Version", "1".to_string()),
        //     ("Device-Id", self.device_id.clone()),
        //     ("Client-Id", self.client_id.clone()),
        // ];
        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                println!("WebSocket connected: {}", url);
                let (mut write, mut read) = ws_stream.split();
                // Spawn a task to handle incoming messages
                let hello_received = self.hello_received.clone();
                let connected = self.connected.clone();

                let (_tx, rx) = mpsc::unbounded_channel();
                self.recver = Arc::new(Mutex::new(Some(rx)));
                
                tokio::spawn(async move {
                    while let Some(Ok(msg)) = read.next().await {
                        match msg {
                            Message::Text(text) => {
                                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if data["type"] == "hello" {
                                        hello_received.notify_one();
                                        let mut conn = connected.lock().unwrap();
                                        *conn = true;
                                    } else {
                                        println!("{}", text);
                                    }
                                }
                            }
                            Message::Binary(bytes) => {
                                AudioPipeline::get_instance()
                                    .read()
                                    .unwrap()
                                    .write_output_data(bytes.to_vec());
                            }
                            _ => {
                                println!("Received message: {:?}", msg);
                            }
                        }
                    }
                });

                // Spawn a task to handle outgoing messages
                let (tx, mut rx) = mpsc::unbounded_channel();
                self.sender = Some(tx);
                tokio::spawn(async move {
                    while let Some(msg) = rx.recv().await {
                        if write.send(msg).await.is_err() {
                            break;
                        }
                    }
                });

                // Send hello message
                let hello_message = json!({
                    "type": "hello",
                    "version": 1,
                    "transport": "websocket",
                    "audio_params": {
                        "format": "opus",
                        "sample_rate": 16000,
                        "channels": 1,
                        "frame_duration": 60
                    }
                });

                self.send_text(hello_message.to_string()).await?;

                // Wait for hello response
                tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    self.hello_received.notified(),
                )
                .await
                .map_err(|_| "等待服务器hello响应超时".to_string())?;

                Ok(())
            }
            Err(e) => Err(format!("WebSocket连接失败: {}", e)),
        }
    }

    pub async fn send_audio(&self, data: Vec<u8>) -> Result<(), String> {
        if let Some(sender) = &self.sender {
            sender
                .send(Message::Binary(data.into()))
                .map_err(|_| "发送音频数据失败".to_string())
        } else {
            Err("WebSocket未连接".to_string())
        }
    }

    pub async fn send_text(&self, message: String) -> Result<(), String> {
        if let Some(sender) = &self.sender {
            sender
                .send(Message::Text(message.into()))
                .map_err(|_| "发送文本消息失败".to_string())
        } else {
            Err("WebSocket未连接".to_string())
        }
    }

    // pub fn receive_audio(&self) -> Result<Vec<Vec<u8>>, String> {
    //     let mut buffer = Vec::new();
    //     if let Some(recver) = self.recver.lock().unwrap().as_mut() {
    //         let _n = recver.blocking_recv_many(&mut buffer, 10);
    //         return Ok(buffer);
    //     } else {
    //         Err("WebSocket未连接".to_string())
    //     }
    // }

    pub fn close(&self) -> Result<(), String> {
        if let Some(sender) = &self.sender {
            sender
                .send(Message::Close(None))
                .map_err(|_| "关闭WebSocket连接失败".to_string())
        } else {
            Err("WebSocket未连接".to_string())
        }
    }
}
