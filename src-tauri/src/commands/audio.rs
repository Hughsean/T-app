use crate::state::AppState;
use tauri::State;
use tracing::debug;
#[tauri::command]
pub async fn audio_start(
    state: State<'_, AppState>,
    webview: tauri::WebviewWindow,
) -> Result<(), String> {
    debug!("{}", webview.label());

    state
        .audio_starte
        .write()
        .await
        .start(Some(webview))
        .await?;
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
