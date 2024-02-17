// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync;
use tauri::{Manager, State};
use crate::elm327::Elm327;

mod elm327;
mod kia;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An I/O error in a low-level [std::io] stream operation
    #[error("IO error: `{0:?}`")]
    InvalidResponse(String),

    #[error("Invalid parameter {0:?}: `{1:?}`")]
    InvalidParameter(String, String),

    #[error("Elm327 error: `{0:?}`")]
    Elm327Error(#[from] elm327::Error),

    #[error("Kia error: `{0:?}`")]
    KiaError(#[from] kia::Error),

    #[error("Error: `{0:?}`")]
    InvalidRequest(String),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}

struct AppState {
    kia: Option<kia::Kia>,
}

#[tauri::command]
async fn connect(connection_method: &str, connection_param: &str, mut app_state: State<'_, sync::Mutex<AppState>>) -> Result<String, Error> {
    let transport: Box<dyn elm327::transport::Transport> = match connection_method {
        "history" => Box::new(elm327::transport::history::History::default()),
        "wifi" => Box::new(elm327::transport::wifi::WiFi::new(connection_param)?),
        _ => return Err(Error::InvalidParameter("connectionMethod".to_string(), "supported values: history, wifi".to_string()))
    };

    let elm327 = Elm327::new(transport).map_err(Error::Elm327Error)?;
    let kia = kia::Kia::new(elm327);
    app_state.lock().unwrap().kia.replace(kia);
    Ok("ELM 327 Device".to_string())
}

#[tauri::command]
async fn get_car_info(mut app_state: State<'_, sync::Mutex<AppState>>) -> Result<kia::CarInfo, Error> {
    if let Some(kia) = app_state.lock().unwrap().kia.as_mut() {
        return kia.get_car_info().map_err(Error::KiaError);
    }

    Err(Error::InvalidRequest("not connected".to_string()))
}

#[tauri::command]
fn get_vin(mut app_state: State<sync::Mutex<AppState>>) -> Result<String, Error> {
    if let Some(kia) = app_state.lock().unwrap().kia.as_mut() {
        return Ok(kia.get_vin()?);
    }

    Err(Error::InvalidRequest("not connected".to_string()))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();


    Ok(tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)]
            app.get_window("main").unwrap().open_devtools(); // `main` is the first window from tauri.conf.json without an explicit label
            Ok(())
        })
        .manage(sync::Mutex::new(AppState {
            kia: None,
        }))
        .invoke_handler(tauri::generate_handler![connect, get_vin, get_car_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application"))
}
