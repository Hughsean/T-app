use crate::{
    audio::cache::AudioCache,
    types::SharedAsyncRwLock,
    utils::frame::{Frame, tts::TtsState},
};
use std::ops::Not;
use tracing::{debug, error, warn};

pub struct Controller {
    worker_thread: Option<tauri::async_runtime::JoinHandle<()>>,
    is_stopped: SharedAsyncRwLock<bool>,
}

impl Controller {
    pub(super) fn new() -> Self {
        Self {
            worker_thread: None,
            is_stopped: SharedAsyncRwLock::new(true.into()),
        }
    }

    pub(super) async fn start(
        controller: SharedAsyncRwLock<Self>,
        audio_cache: SharedAsyncRwLock<AudioCache>,
        ws: SharedAsyncRwLock<crate::utils::ws::WebsocketProtocol>,
    ) {
        if controller.read().await.is_stopped.read().await.not() {
            warn!("已拒绝重复启动控制器工作线程");
            return;
        }
        controller
            .write()
            .await
            .is_stopped
            .write()
            .await
            .clone_from(&false);

        let is_stopped = controller.read().await.is_stopped.clone();

        controller
            .write()
            .await
            .worker_thread
            .replace(tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
                    if *is_stopped.read().await {
                        break;
                    }
                    if let Some(frame) = ws.read().await.read_text_frame().await {
                        match frame {
                            Frame::TtsFrame(frame) => {
                                match frame.state {
                                    TtsState::Start => {
                                        if let Some(text) = frame.text {
                                            // TODO: 处理文本，发送到前端
                                            debug!("对话文本: {}", text);
                                        } else {
                                            error!("对话文本为空");
                                        }
                                    }
                                    TtsState::Stop => {
                                        // XXX 后续考虑增加功能
                                    }
                                    TtsState::SentenceStart => {
                                        audio_cache.write().await.session_stop().await;
                                        audio_cache.write().await.session_start();
                                        debug!("句子开始");
                                    }
                                    TtsState::SentenceEnd => debug!("句子结束"),
                                }
                            }
                            Frame::ListenFrame(_frame) => {}
                            Frame::Error => {}
                        }
                    };
                }
            }));
    }

    pub(crate) async fn close(&mut self) {
        if *self.is_stopped.read().await {
            warn!("已拒绝重复停止控制器工作线程");
            return;
        }
        self.is_stopped.write().await.clone_from(&true);

        if let Some(wt) = self.worker_thread.take() {
            if let Err(e) = wt.await {
                error!("控制器工作线程停止失败: {}", e);
            }
            debug!("控制器工作线程已停止");
        }
    }
}

impl Drop for Controller {
    fn drop(&mut self) {
        tokio::task::block_in_place(|| {
            tauri::async_runtime::block_on(self.close());
            debug!("控制器实例释放资源")
        })
    }
}
