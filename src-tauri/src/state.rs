// TODO
use crate::audio::{AudioState, AudioState_};

pub struct AppState {
    pub audio_starte: AudioState,
}

impl AppState {
    pub fn new() -> Self {
        tauri::async_runtime::block_on(Self::new_async())
    }

    async fn new_async() -> Self {
        Self {
            audio_starte: AudioState::new(AudioState_::new().await.into()),
        }
    }
}
