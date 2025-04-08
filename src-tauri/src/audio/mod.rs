pub mod audio;
pub mod cache;
pub mod controller;
mod func;

use crate::{
    types::SharedAsyncRwLock,
    utils::{config::Config, ws::WebsocketProtocol},
};
use audio::Audio;
use cache::AudioCache;
use controller::Controller;
//
//
//
pub type AudioState = SharedAsyncRwLock<AudioState_>;

pub struct AudioState_ {
    pub audio: SharedAsyncRwLock<Audio>,
    pub audio_cache: SharedAsyncRwLock<AudioCache>,
    pub controller: SharedAsyncRwLock<Controller>,
    pub ws: SharedAsyncRwLock<WebsocketProtocol>,
}

impl AudioState_ {
    pub async fn new() -> Self {
        let ws = SharedAsyncRwLock::new(WebsocketProtocol::new(
            Config::get_instance().websocket.url.clone(),
        ));
        let audio_cache = AudioCache::new(ws.clone()).await;
        Self {
            audio: SharedAsyncRwLock::new(Audio::new()),
            audio_cache: audio_cache.clone(),
            controller: Controller::new(audio_cache).await,
            ws,
        }
    }
    pub async fn ws_connect(&self) -> Result<String, String> {
        let mut ws = self.ws.write().await;
        ws.connect(self.audio_cache.clone(), self.controller.clone())
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn audio_start(&self) {
        let mut audio = self.audio.write().await;
        audio.start(self.audio_cache.clone()).await;
    }

    pub async fn audio_stop(&self) {
        let mut audio = self.audio.write().await;
        audio.stop().await;
    }
}

// impl Default for AudioState_ {
//     fn default() -> Self {
//         Self {
//             audio: None,
//             audio_cache: None,
//             controller: None,
//             ws: None,
//         }
//     }
// }
// lazy_static! {
//     static ref AUDIO_STATE: AudioState = SharedAsyncRwLock::new();
// }

// pub fn close() {
//     let mut audio_state = AUDIO_STATE.lock().unwrap();
//     if let Some(instance) = audio_state.take() {}
// }
