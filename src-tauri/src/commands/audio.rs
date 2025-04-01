// use tauri::State;

use crate::state::State;

#[tauri::command]
pub async fn start(state: tauri::State<'_, State>) -> Result<(), String> {
    // state.read().await.audio.start();
    state.write().await.audio.start();
    
    Ok(())
}
