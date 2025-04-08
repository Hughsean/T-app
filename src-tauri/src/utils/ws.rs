use crate::audio::cache::AudioCache;
use crate::audio::controller::Controller;
use crate::types::SharedAsyncRwLock;
use crate::types::SharedMutex;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::ops::Not;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::debug;
use tracing::info;

pub struct WebsocketProtocol {
    websocket_url: String,
    connected: SharedAsyncRwLock<bool>,
    hello_received: Arc<Notify>,
    sender: Option<mpsc::UnboundedSender<Message>>,
    recver: SharedMutex<Option<mpsc::UnboundedReceiver<Vec<u8>>>>,
    session_id: SharedAsyncRwLock<String>,
}

impl WebsocketProtocol {
    pub fn new(websocket_url: String) -> Self {
        Self {
            websocket_url,
            connected: SharedAsyncRwLock::new(false),
            hello_received: Arc::new(Notify::new()),
            sender: None,
            recver: SharedMutex::new(None),
            session_id: SharedAsyncRwLock::new(String::new()),
        }
    }

    pub async fn connect(
        &mut self,
        audio_cache: SharedAsyncRwLock<AudioCache>,
        controller: SharedAsyncRwLock<Controller>,
    ) -> Result<String, String> {
        if *self.connected.read().await {
            return Ok(self.session_id.read().await.clone());
        }

        let url = self.websocket_url.clone();
        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                info!("WebSocket 已连接: {}", url);
                let (mut write, mut read) = ws_stream.split();
                // Spawn a task to handle incoming messages
                let hello_received = self.hello_received.clone();
                let connected = self.connected.clone();

                let (_tx, rx) = mpsc::unbounded_channel();
                let id = self.session_id.clone();

                self.recver = SharedMutex::new(Some(rx));

                tokio::spawn(async move {
                    while let Some(Ok(msg)) = read.next().await {
                        match msg {
                            Message::Text(text) => {
                                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if data["type"] == "hello" && connected.read().await.not() {
                                        hello_received.notify_one();
                                        *connected.write().await = true;
                                        *id.write().await =
                                            data["session_id"].as_str().unwrap_or("").to_string();
                                        debug!("session_id = {}", id.read().await);
                                    }

                                    let frame = crate::utils::frame::Frame::from(data.clone());

                                    debug!("控制帧:\n{:#?}", frame);

                                    controller.write().await.push_frame(frame).await;
                                }
                            }
                            Message::Binary(bytes) => {
                                audio_cache
                                    .read()
                                    .await
                                    .write_output_data(bytes.to_vec())
                                    .await;
                            }
                            Message::Close(frame) => {
                                debug!("WebSocket 连接关闭: {:?}", frame);
                                // *connected.write().await = false;
                                // controller.write().await.stop().await;
                                // TODO: 处理关闭连接的逻辑
                            }
                            _ => {
                                debug!("Received message:\n{:#?}", msg);
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

                Ok(self.session_id.read().await.clone())
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

    pub async fn auto_ctrl(&self) -> Result<(), String> {
        if let Some(sender) = &self.sender {
            sender
                .send(Message::Text("{\"type\":\"auto_control\"}".into()))
                .map_err(|_| "发送自动控制消息失败".to_string())
        } else {
            Err("WebSocket未连接".to_string())
        }
    }

    pub async fn close(&self) -> Result<(), String> {
        if *self.connected.read().await {
            return Ok(());
        }
        *self.connected.write().await = false;

        if let Some(sender) = &self.sender {
            sender
                .send(Message::Close(None))
                .map_err(|_| "关闭WebSocket连接失败".to_string())
        } else {
            Err("WebSocket未连接".to_string())
        }
    }

    pub async fn get_session_id(&self) -> Option<String> {
        if *self.connected.read().await {
            Some(self.session_id.read().await.clone())
        } else {
            None
        }
    }
}
