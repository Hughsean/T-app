use commands::{audio::start, greet};
pub mod audio;
pub mod commands;
pub mod constraint;
pub mod state;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // .manage(state::new_state())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, start])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
