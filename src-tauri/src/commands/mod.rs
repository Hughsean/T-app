use tauri::Manager;

pub mod audio;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
pub async fn greet(name: String) -> String {
    name
}

#[tauri::command]
pub async fn open_settings_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    let settings_window = app_handle
        .get_webview_window("settings")
        .ok_or("未查询到设置窗口")?;

    settings_window
        .show()
        .map_err(|e| format!("设置窗口显示失败: {}", e))?;

    if settings_window
        .is_visible()
        .map_err(|e| format!("设置窗口查询失败: {}", e))?
    {
        settings_window
            .set_focus()
            .map_err(|e| format!("设置窗口显示失败: {}", e))?;
    }

    Ok(())
}
