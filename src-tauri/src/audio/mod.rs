pub mod audio;
pub mod cache;
pub mod controller;
mod func;

use std::ops::Not;

use crate::{
    types::SharedAsyncRwLock,
    utils::{config::Config, ws::WebsocketProtocol},
};
use audio::Audio;
use cache::AudioCache;
use controller::Controller;
use tracing::{debug, info, warn};
//
//
//
pub type AudioState = SharedAsyncRwLock<AudioState_>;

pub struct AudioState_ {
    audio: SharedAsyncRwLock<Audio>,
    audio_cache: SharedAsyncRwLock<AudioCache>,
    controller: SharedAsyncRwLock<Controller>,
    ws: SharedAsyncRwLock<WebsocketProtocol>,
    stopped: SharedAsyncRwLock<bool>,
}

impl AudioState_ {
    pub async fn new() -> Self {
        Self {
            audio: SharedAsyncRwLock::new(Audio::new().into()),
            audio_cache: SharedAsyncRwLock::new(AudioCache::new().into()),
            controller: SharedAsyncRwLock::new(Controller::new().into()),
            ws: SharedAsyncRwLock::new(
                WebsocketProtocol::new(Config::get_instance().websocket.url.clone()).into(),
            ),
            stopped: SharedAsyncRwLock::new(true.into()),
        }
    }

    pub async fn ws_connect(&self) -> Result<String, String> {
        self.ws
            .write()
            .await
            .connect()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn start(&mut self) -> Result<(), String> {
        if self.stopped.read().await.not() {
            debug!("对话已开始，拒绝再次启动");
            return Ok(());
        }

        if self.ws.read().await.is_connected().await.not() {
            // XXX id 留存，如果需要使用
            debug!("WebSocket 连接中...");
            let _id = self
                .ws_connect()
                .await
                .inspect_err(|e| debug!("WebSocket 连接失败: {}", e))?;
        }
        AudioCache::start(self.audio_cache.clone(), self.ws.clone()).await;
        Audio::start(self.audio.clone(), self.audio_cache.clone()).await;
        Controller::start(
            self.controller.clone(),
            self.audio_cache.clone(),
            self.ws.clone(),
        )
        .await;

        let notify = self.ws.read().await.get_notify();
        let controller = self.controller.clone();
        let audio = self.audio.clone();
        let audio_cache = self.audio_cache.clone();
        let ws = self.ws.clone();
        let stopped = self.stopped.clone();
        tauri::async_runtime::spawn(async move {
            notify.notified().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(900)).await;
            info!("WebSocket 断开连接，准备清理资源");
            stopped.write().await.clone_from(&true);
            controller.write().await.close().await;
            audio.write().await.close().await;
            audio_cache.write().await.reset().await;
            ws.write()
                .await
                .close()
                .await
                .inspect_err(|e| warn!("WebSocket 关闭失败: {}", e))
                .ok();
            info!("资源清理完成，已停止对话");
        });
        self.stopped.write().await.clone_from(&false);
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        if *self.stopped.read().await {
            warn!("对话已结束，无需再次停止");
            return Ok(());
        }
        self.ws.read().await.get_notify().notify_waiters();
        self.controller.write().await.close().await;
        self.audio.write().await.close().await;
        self.audio_cache.write().await.reset().await;
        self.ws.write().await.close().await?;
        self.stopped.write().await.clone_from(&true);
        Ok(())
    }
}
