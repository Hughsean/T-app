// use tauri::State;

// use crate::state::State;

use crate::audio::audio::Audio;

#[tauri::command]
pub async fn audio_start() {
    Audio::get_instance().write().await.start().await;
}

#[tauri::command]
pub async fn audio_stop() {
    Audio::get_instance().write().await.stop().await;
}
