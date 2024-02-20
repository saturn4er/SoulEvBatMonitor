use std::sync;
use log::debug;
use tauri::State;
use crate::{AppState, elm327, kia};
use crate::elm327::Elm327;
use crate::error::CommandError;

#[tauri::command]
pub async fn connect(connection_method: &str, connection_param: &str, app_state: State<'_, sync::Mutex<AppState>>) -> Result<String, CommandError> {
    let transport: Box<dyn elm327::transport::Transport> = match connection_method {
        "wifi" => Box::new(elm327::transport::WiFi::new(connection_param)?),
        "serial" => Box::new(elm327::transport::Serial::new(connection_param)?),
        _ => return Err(CommandError {
            code: "invalid_connection_method".to_string(),
            message: "Invalid connection method".to_string(),
            parameters: Some(vec!(connection_method.to_string())),
        })
    };

    let elm327 = Elm327::new(transport).map_err(|e| {
        match e.current_context() {
            elm327::Error::Communication | elm327::Error::Other => CommandError::new_internal(),
            elm327::Error::NotConnected => CommandError::new_not_connected(),
        }
    })?;
    let connected_device_name = elm327.get_connected_device_name();
    let mut kia = kia::Kia::new(elm327);
    kia.init()?;
    app_state.lock().unwrap().kia.replace(kia);
    Ok(connected_device_name)
}


#[tauri::command]
pub fn disconnect(app_state: State<'_, sync::Mutex<AppState>>) {
    app_state.lock().unwrap().kia = None;
}



#[tauri::command]
pub async fn get_car_info(app_state: State<'_, sync::Mutex<AppState>>) -> Result<kia::CarInfo, CommandError> {
    if let Some(kia) = app_state.lock().unwrap().kia.as_mut() {
        return Ok(kia.get_car_info()?);
    }

    Err(CommandError::new_not_connected())
}

#[tauri::command]
pub async fn list_serial_devices() -> Result<Vec<String>, CommandError> {
    let devices = serialport::available_ports()
        .map_err(|e| {
            debug!("Error listing serial devices: {}", e);
            CommandError::new_internal()
        })?
        .iter()
        .map(|port| port.port_name.clone())
        .collect();
    Ok(devices)
}
