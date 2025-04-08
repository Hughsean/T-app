#[tauri::command]
pub async fn audio_start() {
    // FIXME
    // Audio::get_instance().write().await.start().await;
}

#[tauri::command]
pub async fn audio_stop() {
    // FIXME
    // Audio::get_instance().write().await.stop().await;
}

#[tauri::command]
pub async fn abort() -> Result<(), String> {
    unimplemented!()
}
