#![allow(non_snake_case)]
use crate::{audio::func, types::SharedAsyncRwLock};
use std::ops::Not;
use tracing::{debug, error, info};

pub struct Audio {
    pub audioStoped: SharedAsyncRwLock<bool>,
    audioInThread: Option<std::thread::JoinHandle<()>>,
    audioOutThread: Option<std::thread::JoinHandle<()>>,
}

impl Audio {
    pub fn new() -> Self {
        Audio {
            audioStoped: SharedAsyncRwLock::new(true),
            audioInThread: None,
            audioOutThread: None,
        }
    }

    #[allow(dead_code)]
    async fn reset_(&mut self) {
        self.stop().await;
        *self = Self::new();
    }

    pub async fn start(&mut self, audio_cache: SharedAsyncRwLock<crate::audio::cache::AudioCache>) {
        if self.audioStoped.read().await.not() {
            return;
        }
        *self.audioStoped.write().await = false;

        let audioStoped_ = self.audioStoped.clone();

        let audio_cache_ = audio_cache.clone();

        let in_thread = std::thread::Builder::new()
            .name("音频输入线程".into())
            .spawn(move || {
                tauri::async_runtime::block_on(func::input(audioStoped_, audio_cache_));
                // func::input(audioStoped_);
            })
            .inspect_err(|e| error!("音频输入线程启动失败: {}", e))
            .unwrap();

        let audioStoped_ = self.audioStoped.clone();
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

    pub async fn stop(&mut self) {
        debug!("音频停止中...");
        if *self.audioStoped.read().await {
            info!("音频已停止，无需再次停止");
            return;
        }

        *self.audioStoped.write().await = true;

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
