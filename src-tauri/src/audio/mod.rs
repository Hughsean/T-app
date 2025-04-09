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
use tracing::debug;
//
//
//
pub type AudioState = SharedAsyncRwLock<AudioState_>;

pub struct AudioState_ {
    audio: SharedAsyncRwLock<Audio>,
    audio_cache: SharedAsyncRwLock<AudioCache>,
    controller: SharedAsyncRwLock<Controller>,
    ws: SharedAsyncRwLock<WebsocketProtocol>,
}

impl AudioState_ {
    pub async fn new() -> Self {
        Self {
            audio: SharedAsyncRwLock::new(Audio::new().into()),
            audio_cache: AudioCache::new().await,
            controller: Controller::new().await,
            ws: SharedAsyncRwLock::new(
                WebsocketProtocol::new(Config::get_instance().websocket.url.clone()).into(),
            ),
        }
    }
    
    pub async fn ws_connect(&self) -> Result<String, String> {
        self.ws
            .write()
            .await
            .connect(self.audio_cache.clone())
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn start(&self) -> Result<(), String> {
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

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        self.audio.write().await.close().await;
        self.controller.write().await.close().await;
        // self.audio_cache.write().await.clear();
        // FIXME: 这里需要清空缓存
        
        self.ws.write().await.close().await?;
        Ok(())
    }
}
