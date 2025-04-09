#![allow(non_snake_case)]
use crate::{audio::func, types::SharedAsyncRwLock};
use std::ops::Not;
use tracing::{debug, error, info};

pub struct Audio {
    pub is_audio_stoped: SharedAsyncRwLock<bool>,
    audioInThread: Option<std::thread::JoinHandle<()>>,
    audioOutThread: Option<std::thread::JoinHandle<()>>,
}

impl Audio {
    pub fn new() -> Self {
        Audio {
            is_audio_stoped: SharedAsyncRwLock::new(true.into()),
            audioInThread: None,
            audioOutThread: None,
        }
    }

    pub async fn start(&mut self, audio_cache: SharedAsyncRwLock<crate::audio::cache::AudioCache>) {
        if self.is_audio_stoped.read().await.not() {
            return;
        }
        *self.is_audio_stoped.write().await = false;

        let audioStoped_ = self.is_audio_stoped.clone();

        let audio_cache_ = audio_cache.clone();

        let in_thread = std::thread::Builder::new()
            .name("音频输入线程".into())
            .spawn(move || {
                tauri::async_runtime::block_on(func::input(audioStoped_, audio_cache_));
                // func::input(audioStoped_);
            })
            .inspect_err(|e| error!("音频输入线程启动失败: {}", e))
            .unwrap();

        let audioStoped_ = self.is_audio_stoped.clone();
        let out_thread = std::thread::Builder::new()
            .name("音频输出线程".into())
            .spawn(move || {
                tauri::async_runtime::block_on(func::output(audioStoped_, audio_cache));
                // func::output(audioStoped_);
            })
            .inspect_err(|e| error!("音频输出线程启动失败: {}", e))
            .unwrap();

        self.audioInThread = Some(in_thread);
        self.audioOutThread = Some(out_thread);
    }

    pub async fn close(&mut self) {
        debug!("音频停止中...");
        if *self.is_audio_stoped.read().await {
            info!("音频已停止，无需再次停止");
            return;
        }
        self.is_audio_stoped.write().await.clone_from(&true);

        debug!("音频停机信号已发送...");

        if let Some(thread) = self.audioInThread.take() {
            thread.join().unwrap_or_else(|_e| {
                error!("输入线程停止失败");
            });
        }

        debug!("输入线程停止");

        if let Some(thread) = self.audioOutThread.take() {
            thread.join().unwrap_or_else(|_e| {
                error!("输出线程停止失败");
            });
        }
        debug!("输出线程停止");
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        tokio::task::block_in_place(|| {
            tauri::async_runtime::block_on(self.close());
            debug!("音频实例释放资源")
        })
    }
}
