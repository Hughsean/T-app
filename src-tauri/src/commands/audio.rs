use crate::state::AppState;
use tauri::State;
#[tauri::command]
pub async fn audio_start(state: State<'_, AppState>) -> Result<(), String> {
    state.audio_starte.write().await.start().await?;
    Ok(())
}

#[tauri::command]
pub async fn audio_stop(state: State<'_, AppState>) -> Result<(), String> {
    state.audio_starte.read().await.stop().await?;
    Ok(())
}

#[tauri::command]
pub async fn abort() -> Result<(), String> {
    // TODO: 终止对话
    unimplemented!()
}
