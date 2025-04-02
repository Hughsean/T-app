use anyhow::anyhow;
use commands::{audio::audio_start, greet};
use tauri::Manager;
pub mod audio;
pub mod commands;
pub mod constraint;
pub mod state;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app
                .get_webview_window("main")
                .ok_or(anyhow!("未查询到主窗口"))?;

            std::thread::spawn(move || {
                #[cfg(not(debug_assertions))]
                {
                    std::thread::sleep(std::time::Duration::from_millis(600));
                }
                window.show().unwrap();
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, audio_start])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
