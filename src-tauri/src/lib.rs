use anyhow::anyhow;
use commands::{audio::audio_start, greet};
use tauri::Manager;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utils::config::CONFIG;
pub mod audio;
pub mod commands;
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
pub fn init_logger() {
    let (filter, reload_handle) = tracing_subscriber::reload::Layer::new(LevelFilter::DEBUG);

    let fmt = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_level(true)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry().with(filter).with(fmt).init();

    let level = CONFIG.logger.clone().into();
    reload_handle
        .modify(|f| *f = level)
        .expect("Failed to reload tracing subscriber");

    info!("CWD: {}", std::env::current_dir().unwrap().display());
}
