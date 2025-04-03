use anyhow::anyhow;
use commands::{audio::audio_start, greet};
use tauri::Manager;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utils::config::Config;
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

/// 初始化日志
pub fn init_logger() {
    let log_file: std::fs::File = Config::get_instance().logger.clone().into();
    let level: tracing::level_filters::LevelFilter = Config::get_instance().logger.clone().into();

    // 配置输出到文件的 fmt 层
    let file_fmt = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_timer(tracing_subscriber::fmt::time::LocalTime::new(
            time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
        ))
        .with_ansi(false)
        .with_writer(std::sync::Mutex::new(log_file));

    // 配置输出到控制台的 fmt 层
    let console_fmt = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_timer(tracing_subscriber::fmt::time::LocalTime::new(
            time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
        ))
        .with_ansi(true);

    tracing_subscriber::registry()
        .with(level)
        .with(file_fmt)
        .with(console_fmt)
        .init();
    info!(">>>>>日志初始化完成<<<<<");

    info!("工作目录: {}", std::env::current_dir().unwrap().display());
}
