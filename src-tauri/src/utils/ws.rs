use crate::types::SharedAsyncRwLock;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::ops::Not;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::warn;
use tracing::{debug, error, info};

pub struct WebsocketProtocol {
    websocket_url: String,
    is_connected: SharedAsyncRwLock<bool>,
    session_id: SharedAsyncRwLock<Option<String>>,

    hello_received: Arc<Notify>,
    timeout_received: Arc<Notify>,

    input_handle: Option<tauri::async_runtime::JoinHandle<()>>,
    output_handle: Option<tauri::async_runtime::JoinHandle<()>>,

    // 消息通道
    msg_sender: Option<mpsc::UnboundedSender<Message>>,
    frame_recver: SharedAsyncRwLock<Option<mpsc::UnboundedReceiver<crate::utils::frame::Frame>>>,
    audio_recver: SharedAsyncRwLock<Option<mpsc::UnboundedReceiver<Vec<u8>>>>,
}

impl WebsocketProtocol {
    pub fn new(websocket_url: String) -> Self {
        Self {
            websocket_url,
            is_connected: SharedAsyncRwLock::new(false.into()),
            session_id: SharedAsyncRwLock::new(None.into()),

            hello_received: Arc::new(Notify::new()),
            timeout_received: Arc::new(Notify::new()),

            input_handle: None,
            output_handle: None,

            msg_sender: None,
            frame_recver: SharedAsyncRwLock::new(None.into()),
            audio_recver: SharedAsyncRwLock::new(None.into()),
        }
    }

    pub async fn connect(
        &mut self,
        // audio_cache: SharedAsyncRwLock<AudioCache>,
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

                let connected = self.is_connected.clone();
                let id = self.session_id.clone();
                let hello_received = self.hello_received.clone();
                let timeout_received = self.timeout_received.clone();

                let (frame_sender, frame_recv) =
                    mpsc::unbounded_channel::<crate::utils::frame::Frame>();
                let (audio_sender, audio_recv) = mpsc::unbounded_channel::<Vec<u8>>();

                self.frame_recver.write().await.replace(frame_recv);
                self.audio_recver.write().await.replace(audio_recv);

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
                                    audio_sender.send(bytes.to_vec()).unwrap_or_else(|e| {
                                        error!("发送音频数据失败: {}", e);
                                    });
                                    // audio_cache
                                    //     .read()
                                    //     .await
                                    //     .write_output_data(bytes.to_vec())
                                    //     .await;
                                }
                                Message::Close(frame) => {
                                    debug!("WebSocket 连接关闭: {:?}", frame);
                                    // *connected.write().await = false;
                                    // controller.write().await.stop().await;
                                    // self.close()./;
                                    // TODO: 处理关闭连接的逻辑
                                    timeout_received.notify_waiters();
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

                let hello_msg = json!(
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
                );
                self.send_text(hello_msg.to_string()).await?;
                debug!("发送hello消息: {}", &hello_msg);

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
        if let Some(mut recver) = self.frame_recver.write().await.take() {
            recver.close();
            drop(recver);
        }
        if let Some(mut recver) = self.audio_recver.write().await.take() {
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
    pub fn get_notify(&self) -> Arc<Notify> {
        self.timeout_received.clone()
    }

    pub async fn send_audio(&self, data: Vec<u8>) -> Result<(), String> {
        if let Some(sender) = &self.msg_sender {
            sender
                .send(Message::Binary(data.into()))
                .map_err(|_| "发送音频数据失败".to_string())
        } else {
            Err("WebSocket未连接".to_string())
        }
    }

    pub async fn send_text(&self, message: String) -> Result<(), String> {
        if let Some(sender) = &self.msg_sender {
            sender
                .send(Message::Text(message.into()))
                .map_err(|_| "发送文本消息失败".to_string())
        } else {
            Err("WebSocket未连接".to_string())
        }
    }

    pub async fn read_text_frame(&self) -> Option<crate::utils::frame::Frame> {
        if let Some(recver) = self.frame_recver.write().await.as_mut() {
            match recver.try_recv() {
                Ok(frame) => Some(frame),
                Err(_e) => {
                    // debug!("读取控制帧失败: {:?}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    pub async fn read_audio_data(&self) -> Option<Vec<u8>> {
        if let Some(recver) = self.audio_recver.write().await.as_mut() {
            match recver.try_recv() {
                Ok(data) => Some(data),
                Err(_e) => {
                    // debug!("读取音频数据失败: {:?}", e);
                    None
                }
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

    // #[deprecated]
    // pub fn audio_handler(
    //     mut read: futures_util::stream::SplitStream<
    //         tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>,
    //     >,
    //     connected: SharedAsyncRwLock<bool>,
    //     hello_received: Arc<Notify>,
    //     id: SharedAsyncRwLock<String>,
    //     audio_cache: SharedAsyncRwLock<AudioCache>,
    // ) -> impl Future + Send + 'static {
    //     async move {
    //         while let Some(Ok(msg)) = read.next().await {
    //             match msg {
    //                 Message::Text(text) => {
    //                     if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
    //                         if data["type"] == "hello" && connected.read().await.not() {
    //                             hello_received.notify_one();
    //                             *connected.write().await = true;
    //                             *id.write().await =
    //                                 data["session_id"].as_str().unwrap_or("").to_string();
    //                             debug!("session_id = {}", id.read().await);
    //                         }

    //                         let frame = crate::utils::frame::Frame::from(data.clone());

    //                         debug!("控制帧:\n{:#?}", frame);
    //                     }
    //                 }
    //                 Message::Binary(bytes) => {
    //                     audio_cache
    //                         .read()
    //                         .await
    //                         .write_output_data(bytes.to_vec())
    //                         .await;
    //                 }
    //                 Message::Close(frame) => {
    //                     debug!("WebSocket 连接关闭: {:?}", frame);
    //                     // *connected.write().await = false;
    //                     // controller.write().await.stop().await;
    //                     // TODO: 处理关闭连接的逻辑
    //                 }
    //                 _ => {
    //                     debug!("Received message:\n{:#?}", msg);
    //                 }
    //             }
    //         }
    //     }
    // }
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
