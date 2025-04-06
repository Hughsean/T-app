use anyhow::anyhow;
use commands::{audio::audio_start, greet, open_settings_window};
use std::sync::{Arc, RwLock};
use tauri::Manager;
use tracing::{debug, error};
pub mod audio;
pub mod commands;
pub mod state;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(setup)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            audio_start,
            open_settings_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup(app: &mut tauri::App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let exit_flag = Arc::new(RwLock::new(false));

    let main_window = app
        .get_webview_window("main")
        .ok_or(anyhow!("未查询到主窗口"))?;

    let settings = app
        .get_webview_window("settings")
        .ok_or(anyhow!("未查询到设置窗口"))?;

    let settings_ = settings.clone();
    let exit = exit_flag.clone();

    settings.on_window_event(move |e| match e {
        tauri::WindowEvent::CloseRequested { api, .. } => {
            // 关闭窗口时，隐藏窗口而不是直接关闭
            if !*exit.read().unwrap() {
                api.prevent_close();
                settings_.hide().unwrap();
            }
        }
        _ => {
            debug!("设置窗口事件: {:?}", e);
        }
    });

    let settings_ = settings.clone();
    // let main_window_ = main_window.clone();
    main_window.on_window_event(move |e| match e {
        tauri::WindowEvent::CloseRequested { .. } => {
            // 关闭窗口时，隐藏窗口而不是直接关闭
            exit_flag.write().unwrap().clone_from(&true);
            settings_.close().unwrap();
        }

        _ => {
            debug!("主窗口事件: {:?}", e);
        }
    });

    std::thread::Builder::new()
        .name("主窗口显示线程".into())
        .spawn(move || {
            #[cfg(not(debug_assertions))]
            {
                std::thread::sleep(std::time::Duration::from_millis(600));
            }
            main_window.show().unwrap();
        })
        .inspect_err(|e| error!("主窗口显示失败: {}", e))
        .unwrap();
    Ok(())
}
