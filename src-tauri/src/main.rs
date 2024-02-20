// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync;
mod elm327;
mod kia;
mod error;
mod command;
use command::*;

struct AppState {
    kia: Option<kia::Kia>,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();


    Ok(tauri::Builder::default()
        .manage(sync::Mutex::new(AppState {
            kia: None,
        }))
        .invoke_handler(tauri::generate_handler![
            connect,
            disconnect,
            get_car_info,
            list_serial_devices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application"))
}
