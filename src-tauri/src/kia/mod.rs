mod command;

use std::collections::HashMap;
use error_stack::{Report, ResultExt};
use serde::{Serialize, Serializer};
use serde::ser::{SerializeSeq};
use crate::elm327::Command;

type Result<T> = error_stack::Result<T, Error>;

use super::{elm327::{
    Elm327
}, elm327};

#[derive(Debug,thiserror::Error)]
pub enum Error {
    #[error("Not connected")]
    NotConnected,
    #[error("Internal error")]
    Other,
}
impl Error {
    fn from_elm327(e: Report<elm327::Error>) -> Report<Error> {
        match e.current_context() {
            elm327::Error::Communication => e.change_context(Error::Other),
            elm327::Error::Other => e.change_context(Error::Other),
            elm327::Error::NotConnected => e.change_context(Error::NotConnected),
        }
    }
}

pub(crate) struct Kia {
    device: Elm327,
}

struct CellVoltages([f32; 96]);
impl Serialize for CellVoltages {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut state = serializer.serialize_seq(Some(96))?;
        for i in 0..96 {
            state.serialize_element(&self.0[i])?;
        }
        state.end()
    }
}

#[derive(Serialize)]
pub struct BatteryInfo {
    charge_level: f64,
    charging: bool,
    chademo_plugged: bool,
    j1772_plugged: bool,
    battery_current: f64,
    battery_dc_voltage: f64,
    max_cell_voltage: f64,
    min_cell_voltage: f64,
    motor_speed: i32,
    cell_voltages: CellVoltages,
    module_temperatures: [i32; 7],
}

struct CarInfoTime(std::time::SystemTime);
impl Serialize for CarInfoTime {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_i64(self.0.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64)
    }
}

#[derive(Serialize)]
pub struct CarInfo {
    time: CarInfoTime,
    battery_info: BatteryInfo,
}

impl Kia {
    pub fn new(device: Elm327) -> Self {
        Self { device }
    }

    pub fn init(&mut self) -> Result<()> {
        let commands = [
            "AT D",
            "AT Z",
            "AT E0",
            "AT L0",
            "AT H0",
            "AT H1",
            "AT AT1",
            "AT AR",
            "AT AL",
            "AT S1",
            "AT FE",
            "AT SPA6",
            "AT CAF1",
            "01 00",
            "01 20",
            "09 00",
            "01 01",
            "AT SH 7DF",
            "AT CRA 7EC",
            "21 00",
        ];

        for cmd in commands {
            self.device.serial_cmd(cmd).map_err(Error::from_elm327)?;
        }

        return Ok(());
    }

    pub fn get_cell_voltages(&mut self) -> Result<[f32; 96]> {
        let mut result: [f32; 96] = [0.0; 96];

        for i in 0..3 {
            let response = self.device
                .execute_command(command::CellVoltagesCommand(format!("{}", i+2)))
                .map_err(Error::from_elm327)?;
            result[i * 32..(i + 1) * 32].copy_from_slice(&response[..32]);
        }

        return Ok(result);
    }

    fn get_battery_info(&mut self) -> Result<BatteryInfo> {
        let battery_info = self.device
            .execute_command(command::BatteryInfoCommand())
            .map_err(Error::from_elm327)?;

        return Ok(BatteryInfo{
            charge_level: battery_info.charge_level,
            charging: battery_info.charging,
            chademo_plugged: battery_info.chademo_plugged,
            j1772_plugged: battery_info.j1772_plugged,
            battery_current: battery_info.battery_current,
            battery_dc_voltage: battery_info.battery_dc_voltage,
            max_cell_voltage: battery_info.max_cell_voltage,
            min_cell_voltage: battery_info.min_cell_voltage,
            motor_speed: battery_info.motor_speed,
            cell_voltages: CellVoltages(self.get_cell_voltages()?),
            module_temperatures: battery_info.module_temperatures,
        });
    }

    pub fn get_car_info(&mut self) -> Result<CarInfo> {
        Ok(CarInfo {
            time: CarInfoTime(std::time::SystemTime::now()),
            battery_info: self.get_battery_info()?,
        })
    }
}

