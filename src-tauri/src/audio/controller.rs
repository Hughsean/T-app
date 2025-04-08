use crate::{
    audio::cache::AudioCache,
    types::SharedAsyncRwLock,
    utils::frame::{Frame, tts::TtsState},
};
use std::collections::VecDeque;
use tracing::{debug, error};

pub struct Controller {
    pub frame_buffer: VecDeque<Frame>,
    worker_thread: Option<tauri::async_runtime::JoinHandle<()>>,
    stop_flag: SharedAsyncRwLock<bool>,
}

impl Controller {
    pub async fn new(audio_cache: SharedAsyncRwLock<AudioCache>) -> SharedAsyncRwLock<Self> {
        let stop_flag = SharedAsyncRwLock::new(false);
        let stop_flag_ = stop_flag.clone();

        let shared_controller = SharedAsyncRwLock::new(Self {
            frame_buffer: VecDeque::new(),
            worker_thread: None,
            stop_flag,
        });

        let shared_controller_ = shared_controller.clone();

        shared_controller
            .write()
            .await
            .worker_thread
            .replace(tauri::async_runtime::spawn(async move {
                loop {
                    if *stop_flag_.read().await {
                        break;
                    }

                    if let Some(frame) = shared_controller_.write().await.frame_buffer.pop_front() {
                        match frame {
                            Frame::TtsFrame(frame) => {
                                match frame.state {
                                    TtsState::Start => {
                                        // TODO
                                    }
                                    TtsState::Stop => {
                                        // TODO
                                        // AudioCache::get_instance()
                                        //     .write()
                                        //     .await
                                        //     .session_stop()
                                        //     .await;
                                    }
                                    TtsState::SentenceStart => {
                                        // let ac= AudioCache::get_instance().write().await;;
                                        audio_cache.write().await.session_stop().await;
                                        audio_cache.write().await.session_start();
                                        debug!("句子开始");
                                    }
                                    TtsState::SentenceEnd => debug!("句子结束"),
                                }
                                // println!("视频帧: {:?}", frame);
                            }
                            Frame::ListenFrame(_frame) => {}
                            Frame::Error => {}
                        }
                    };

                    // 这里是工作线程的逻辑
                    tokio::time::sleep(tokio::time::Duration::from_millis(70)).await;
                }
            }));
        shared_controller
    }

    pub async fn push_frame(&mut self, frame: Frame) {
        self.frame_buffer.push_back(frame);
    }

    pub async fn stop(&mut self) {
        *self.stop_flag.write().await = true;
        if let Some(wt) = self.worker_thread.take() {
            if let Err(e) = wt.await {
                error!("控制器工作线程停止失败: {}", e);
            } else {
                debug!("控制器工作线程已停止");
            }
        } else {
            debug!("控制器工作线程已停止");
        }
        // FIXME
        // Audio::get_instance().write().await.stop().await;
    }
}
