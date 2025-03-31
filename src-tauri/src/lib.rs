// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod state;
pub mod audio;
pub mod constraint;
pub mod ws;

#[tauri::command]
async fn greet(name: String) -> String {
    // pin!(f);

    // let _ = tauri::async_runtime::spawn(audio::fun()).await.unwrap();
    name
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // .manage(state::State::new())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
