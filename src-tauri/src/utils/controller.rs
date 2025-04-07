use super::frame::Frame;
use crate::{audio::cache::AudioCache, types::SharedAsyncRwLock};
use lazy_static::lazy_static;
use std::{collections::VecDeque, sync::Once};
use tracing::{debug, error, info};

lazy_static! {
    static ref CONTROLLER: SharedAsyncRwLock<Controller> =
        SharedAsyncRwLock::new(Controller::new());
}
const INIT: Once = Once::new();

pub struct Controller {
    pub frame_buffer: VecDeque<Frame>,
    worker_thread: Option<tauri::async_runtime::JoinHandle<()>>,
    stop_flag: SharedAsyncRwLock<bool>,
}

impl Controller {
    fn new() -> Self {
        let mut wt = None;
        let stop_flag = SharedAsyncRwLock::new(false);

        let wt_ = &mut wt;
        let stop_flag_ = stop_flag.clone();

        INIT.call_once(|| {
            let _ = wt_.replace(tauri::async_runtime::spawn(async move {
                loop {
                    if *stop_flag_.read().await {
                        break;
                    }
                    // TODO
                    if let Some(frame) = Self::get_instance().write().await.frame_buffer.pop_front()
                    {
                        match frame {
                            Frame::ListenFrame(_frame) => {
                                // println!("音频帧: {:?}", frame);
                            }
                            Frame::TtsFrame(frame) => {
                                match frame.state {
                                    super::frame::tts::TtsState::Start => {
                                        // TODO
                                    }
                                    super::frame::tts::TtsState::Stop => {
                                        // TODO
                                        // AudioCache::get_instance()
                                        //     .write()
                                        //     .await
                                        //     .session_stop()
                                        //     .await;
                                    }
                                    super::frame::tts::TtsState::SentenceStart => {
                                        // let ac= AudioCache::get_instance().write().await;;
                                        AudioCache::get_instance()
                                            .write()
                                            .await
                                            .session_stop()
                                            .await;
                                        AudioCache::get_instance().write().await.session_start();
                                        debug!("句子开始");
                                    }
                                    super::frame::tts::TtsState::SentenceEnd => debug!("句子结束"),
                                }
                                // println!("视频帧: {:?}", frame);
                            }
                            Frame::Error => {
                                info!("读取错误帧");
                            }
                        }
                    };

                    // 这里是工作线程的逻辑
                    // println!("控制器工作线程正在运行...");
                    tokio::time::sleep(tokio::time::Duration::from_millis(70)).await;
                }
            }));
        });
        Self {
            frame_buffer: VecDeque::new(),
            worker_thread: wt,
            stop_flag,
        }
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
    }

    pub fn get_instance() -> SharedAsyncRwLock<Controller> {
        CONTROLLER.clone()
    }
}
