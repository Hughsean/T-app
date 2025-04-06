// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_app_lib::utils::log::init_logger;

fn main() {
    init_logger();
    tauri_app_lib::run()
}
