use commands::audio::start;
pub mod audio;
pub mod commands;
pub mod constraint;
pub mod state;
pub mod utils;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn greet(name: String) -> String {
    name
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(state::new_state())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, start])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
