use crate::audio::cache::AudioCache;
use crate::types::SharedAsyncRwLock;
use futures_util::{SinkExt, StreamExt};
use std::ops::Not;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::warn;
use tracing::{debug, error, info};

const HELLO_JSON: &str = r#"
{
    "type":"hello",
    "version":1,
    "transport":"websocket",
    "audio_params":
    {
        "format":"opus",
        "sample_rate":16000,
        "channels":1,
        "frame_duration":60
    }
}
"#;

pub struct WebsocketProtocol {
    websocket_url: String,
    is_connected: SharedAsyncRwLock<bool>,
    hello_received: Arc<Notify>,
    msg_sender: Option<mpsc::UnboundedSender<Message>>,
    frame_recver: Option<mpsc::UnboundedReceiver<crate::utils::frame::Frame>>,
    session_id: SharedAsyncRwLock<Option<String>>,
    input_handle: Option<tauri::async_runtime::JoinHandle<()>>,
    output_handle: Option<tauri::async_runtime::JoinHandle<()>>,
}

impl WebsocketProtocol {
    pub fn new(websocket_url: String) -> Self {
        Self {
            websocket_url,
            is_connected: SharedAsyncRwLock::new(false.into()),
            hello_received: Arc::new(Notify::new()),
            msg_sender: None,
            frame_recver: None,
            session_id: SharedAsyncRwLock::new(None.into()),
            input_handle: None,
            output_handle: None,
        }
    }

    pub async fn connect(
        &mut self,
        audio_cache: SharedAsyncRwLock<AudioCache>,
    ) -> Result<String, String> {
        if self.is_connected().await {
            warn!("WebSocket 已连接，拒绝重复连接");
            return self
                .session_id
                .read()
                .await
                .as_ref()
                .map(|e| Ok(e.to_owned()))
                .unwrap_or(Err("未查询到 session_id".to_string()));
        }

        let url = self.websocket_url.clone();
        match connect_async(&url).await {
            Ok((ws_stream, _response)) => {
                info!("WebSocket 已连接: {}", url);
                let (mut write, mut read) = ws_stream.split();
                // Spawn a task to handle incoming messages
                let hello_received = self.hello_received.clone();
                let connected = self.is_connected.clone();
                let id = self.session_id.clone();

                let (frame_sender, frame_recv) =
                    mpsc::unbounded_channel::<crate::utils::frame::Frame>();

                self.frame_recver.replace(frame_recv);

                self.input_handle
                    .replace(tauri::async_runtime::spawn(async move {
                        while let Some(Ok(msg)) = read.next().await {
                            match msg {
                                Message::Text(text) => {
                                    if let Ok(data) =
                                        serde_json::from_str::<serde_json::Value>(&text)
                                    {
                                        if data["type"] == "hello" && connected.read().await.not() {
                                            hello_received.notify_one();
                                            *connected.write().await = true;
                                            *id.write().await =
                                                data["session_id"].as_str().map(|e| e.to_string());
                                            debug!(
                                                "session_id = {}",
                                                id.read().await.clone().unwrap_or("".to_string())
                                            );
                                        }

                                        let frame = crate::utils::frame::Frame::from(data.clone());
                                        debug!("控制帧:\n{:#?}", frame);

                                        frame_sender
                                            .send(frame)
                                            .map_err(|e| e.to_string())
                                            .unwrap_or_else(|e| {
                                                error!("发送控制帧失败: {}", e);
                                            });
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
                                    // self.close()./;
                                    // TODO: 处理关闭连接的逻辑
                                }
                                _ => {
                                    debug!("Received message:\n{:#?}", msg);
                                }
                            }
                        }
                        debug!("WebSocket 输入处理线程结束");
                    }));
                debug!("ws 输入处理线程启动成功");

                let (tx, mut rx) = mpsc::unbounded_channel();
                self.msg_sender = Some(tx);

                self.output_handle
                    .replace(tauri::async_runtime::spawn(async move {
                        while let Some(msg) = rx.recv().await {
                            if write.send(msg).await.is_err() {
                                break;
                            }
                        }
                        debug!("WebSocket 输出处理线程结束");
                    }));
                debug!("ws 输出处理线程启动成功");

                self.send_text(HELLO_JSON.into()).await?;
                debug!("发送hello消息: {}", HELLO_JSON);

                tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    self.hello_received.notified(),
                )
                .await
                .map_err(|_| "等待服务器hello响应超时".to_string())?;

                Ok(self
                    .session_id
                    .read()
                    .await
                    .clone()
                    .unwrap_or("None".to_string()))
            }
            Err(e) => Err(format!("WebSocket连接失败: {}", e)),
        }
    }

    pub async fn close(&mut self) -> Result<(), String> {
        if self.is_connected().await.not() {
            warn!("WebSocket 未连接，拒绝重复关闭");
            return Ok(());
        }

        self.is_connected.write().await.clone_from(&false);

        if let Some(sender) = self.msg_sender.take() {
            sender
                .send(Message::Close(None))
                .map_err(|_| "关闭WebSocket连接失败".to_string())?;
            drop(sender);
        }
        if let Some(mut recver) = self.frame_recver.take() {
            recver.close();
            drop(recver);
        }

        if let Some(t) = self.input_handle.take() {
            t.await
                .map_err(|_| "WebSocket输入处理线程关闭失败".to_string())?;
        };
        if let Some(t) = self.output_handle.take() {
            t.await
                .map_err(|_| "WebSocket输出处理线程关闭失败".to_string())?;
        }

        Ok(())
    }
}

impl WebsocketProtocol {
    pub async fn send_audio(&self, data: Vec<u8>) -> Result<(), String> {
        if self.is_connected().await.not() {
            return Err("WebSocket未连接".to_string());
        }

        if let Some(sender) = &self.msg_sender {
            sender
                .send(Message::Binary(data.into()))
                .map_err(|_| "发送音频数据失败".to_string())
        } else {
            Err("WebSocket未连接".to_string())
        }
    }

    pub async fn send_text(&self, message: String) -> Result<(), String> {
        if self.is_connected().await.not() {
            return Err("WebSocket未连接".to_string());
        }

        if let Some(sender) = &self.msg_sender {
            sender
                .send(Message::Text(message.into()))
                .map_err(|_| "发送文本消息失败".to_string())
        } else {
            Err("WebSocket未连接".to_string())
        }
    }

    pub async fn read_text_frame(&mut self) -> Option<crate::utils::frame::Frame> {
        if let Some(recver) = self.frame_recver.as_mut() {
            match recver.recv().await {
                Some(frame) => Some(frame),
                None => None,
            }
        } else {
            None
        }
    }

    pub async fn get_session_id(&self) -> Option<String> {
        self.session_id.read().await.clone()
    }

    pub async fn is_connected(&self) -> bool {
        *self.is_connected.read().await
    }

    pub fn audio_handler(
        mut read: futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>,
        >,
        connected: SharedAsyncRwLock<bool>,
        hello_received: Arc<Notify>,
        id: SharedAsyncRwLock<String>,
        audio_cache: SharedAsyncRwLock<AudioCache>,
    ) -> impl Future + Send + 'static {
        async move {
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
        }
    }
}

impl Drop for WebsocketProtocol {
    fn drop(&mut self) {
        tokio::task::block_in_place(|| {
            tauri::async_runtime::block_on(self.close()).unwrap_or_else(|e| {
                error!("WebSocket 实例关闭失败: {}", e);
            });
            debug!("WebSocket 实例释放资源");
        })
    }
}
