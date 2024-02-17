use std::collections::HashMap;
use std::fmt::format;
use log::debug;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;
use rand::random;
use super::{
    elm327,
    elm327::{
        Elm327
    },
};


#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An I/O error in a low-level [std::io] stream operation
    #[error("IO error: `{0:?}`")]
    InvalidResponse(i32, String),

    #[error("Elm327 error: `{0:?}`")]
    Elm327Error(#[from] elm327::Error),
}
impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut struct_serializer = serializer.serialize_struct("Error", 2)?;
        match self {
            Error::InvalidResponse(code, message) => {
                struct_serializer.serialize_field("error", code)?;
                struct_serializer.serialize_field("code",  &message)?;
            },
            Error::Elm327Error(e) => {
                struct_serializer.serialize_field("error", &format!("elm327 error: {:?}", e))?;
                struct_serializer.serialize_field("code",  &300)?;
            }
        }
        struct_serializer.end()
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::InvalidResponse(100, format!("invalid data recieved: {:?}", e))
    }
}

pub(crate) struct Kia {
    device: Elm327,
}

struct CellVoltages([f32; 96]);

impl Default for CellVoltages {
    fn default() -> Self {
        CellVoltages([0.0; 96])
    }
}

impl Serialize for CellVoltages {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.collect_seq(self.0.iter())
    }
}

#[derive(Default)]
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
    msb: i32,
}

pub struct CarInfo {
    time: std::time::SystemTime,
    vin: String,
    battery_info: BatteryInfo,
    cell_voltages: CellVoltages,
}

impl Default for CarInfo {
    fn default() -> Self {
        Self {
            time: std::time::SystemTime::now(),
            vin: "".to_string(),
            battery_info: BatteryInfo::default(),
            cell_voltages: CellVoltages::default(),
        }
    }
}

impl Serialize for CarInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("CarInfo", 12)?;
        serializer.serialize_field("time", &self.time)?;
        serializer.serialize_field("charge_level", &self.battery_info.charge_level)?;
        serializer.serialize_field("charging", &self.battery_info.charging)?;
        serializer.serialize_field("chademo_plugged", &self.battery_info.chademo_plugged)?;
        serializer.serialize_field("j1772_plugged", &self.battery_info.j1772_plugged)?;
        serializer.serialize_field("battery_current", &self.battery_info.battery_current)?;
        serializer.serialize_field("battery_dc_voltage", &self.battery_info.battery_dc_voltage)?;
        serializer.serialize_field("max_cell_voltage", &self.battery_info.max_cell_voltage)?;
        serializer.serialize_field("min_cell_voltage", &self.battery_info.min_cell_voltage)?;
        serializer.serialize_field("motor_speed", &self.battery_info.motor_speed)?;
        serializer.serialize_field("vin", &self.vin)?;
        serializer.serialize_field("cell_voltages", &self.cell_voltages)?;
        serializer.end()
    }
}

impl Kia {
    pub fn new(device: Elm327) -> Self {
        Self { device }
    }

    fn init(&mut self) -> Result<(), Error> {
        self.device.serial_cmd("AT D")?;    // set all to default
        self.device.serial_cmd("AT Z")?;    // reset
        self.device.serial_cmd("AT E0")?;   // echo off
        self.device.serial_cmd("AT L0")?;   // linefeeds off
        self.device.serial_cmd("AT H0")?;   // headers off
        self.device.serial_cmd("AT H1")?;   // headers on
        self.device.serial_cmd("AT AT1")?;  // adaptive timing on
        self.device.serial_cmd("AT AR")?;   // automatic receive
        self.device.serial_cmd("AT AL")?;   // allow long messages
        self.device.serial_cmd("AT S1")?;   // spaces on
        self.device.serial_cmd("AT FE")?;   // fast init
        self.device.serial_cmd("AT SPA6")?; // protocol auto
        self.device.serial_cmd("AT CAF1")?; // automatic formatting on
        self.device.serial_cmd("01 00")?;   // get supported service 01 PIDs
        self.device.serial_cmd("01 20")?;   // dump supported service 01 PIDs
        self.device.serial_cmd("09 00")?;   // get supported service 09 PIDs
        self.device.serial_cmd("01 01")?;   // service 01 monitor status
        self.device.serial_cmd("AT SH 7DF")?;
        self.device.serial_cmd("AT CRA 7EC")?; //
        self.device.serial_cmd("21 00")?;

        return Ok(());
    }

    pub fn get_vin(&mut self) -> Result<String, Error> {
        let response = self.device.serial_cmd("09 02")?;
        let vin = response.split_whitespace().collect();

        return Ok(vin);
    }

    pub fn get_cell_voltages(&mut self) -> Result<[f32; 96], Error> {
        let mut result: [f32; 96] = [0.0; 96];

        for i in 0..3 {
            let response = self.device.serial_cmd(format!("21 0{}", i + 2).as_ref())?;
            let mut cells = self.parse_cells_pid_output(response.as_str())?;
            result[i * 32..(i + 1) * 32].copy_from_slice(&cells[..32]);
        }

        return Ok(result);
    }
    /*
    write: '21 01\r\n'
    read: '7EC 10 3D 61 01 FF FF FF FF
    7EC 21 56 23 28 23 28 03 00
    7EC 22 2A 0E 10 0B 0A 0B 0A
    7EC 23 0A 0A 0A 00 0C BD 24
    7EC 24 B9 60 00 00 8F 00 07
    7EC 25 1A 49 00 06 FC C3 00
    7EC 26 02 8F 6A 00 02 6C C4
    7EC 27 01 56 63 E9 45 01 6F
    7EC 28 00 00 00 00 03 E8 00
    '
     */
    fn get_battery_info(&mut self) -> Result<BatteryInfo, Error> {
        let mut result = BatteryInfo::default();
        let response = self.device.serial_cmd("21 01")?;
        let mut lines = response.split("\n").filter(|l| l.trim().len() > 0);
        let mut block_values: HashMap<String, Vec<i32>> = HashMap::new();
        for line in lines {
            let mut parts = line.trim().split(" ").skip(1);
            let block = parts.next().ok_or(Error::InvalidResponse(200,"missing block number".to_string()))?;
            let int_result_parts = parts
                .map(|s| {
                    i32::from_str_radix(s, 16)
                        .map_err(|e| Error::InvalidResponse(300, format!("invalid data recieved: {} {:?}", s, e)))
                }).collect::<Result<Vec<i32>, Error>>()?;
            if int_result_parts.len() < 7 {
                return Err(Error::InvalidResponse(400, "missing values for block ".to_string()));
            }

            block_values.insert(block.to_string(), int_result_parts);
        }
        let block_21 = block_values.get("21").ok_or(Error::InvalidResponse(500,"missing block 21".to_string()))?;
        let block_22 = block_values.get("22").ok_or(Error::InvalidResponse(600,"missing block 22".to_string()))?;
        let block_23 = block_values.get("23").ok_or(Error::InvalidResponse(700, "missing block 23".to_string()))?;

        let charging_flags = block_21[5];
        result.charge_level = (block_21[0] as f64) * 0.5;
        result.msb = block_21[6];
        result.charging = (charging_flags & (1 << 7)) != 0;
        result.chademo_plugged = (charging_flags & (1 << 6)) != 0;
        result.j1772_plugged = (charging_flags & (1 << 5)) != 0;
        let (battery_dc_voltage, _) = block_22[1].overflowing_shl( 8);
        result.battery_dc_voltage = (battery_dc_voltage + block_22[2]) as f64 * 0.1;
        result.min_cell_voltage = block_23[0] as f64 * 0.02;
        result.max_cell_voltage = block_23[5] as f64 * 0.02;

        if result.msb > 0 {
            result.battery_current = (result.msb.overflowing_shl(8).0 + block_22[0]) as f64 * 0.1;
            if (result.msb & 0x80) != 0 {
                result.battery_current -= 6553.6;
            }
        }

        Ok(result)
    }

    fn parse_cells_pid_output(&self, response: &str) -> Result<[f32; 35], Error> {
        let mut result = [0.0; 35];
        let mut lines = response.split("\n");
        for line in lines {
            if line.len() == 0 {
                continue;
            }

            let mut parts = line.split(" ").skip(1);
            let block = parts.next().ok_or(Error::InvalidResponse(800, "missing block number".to_string()))?;
            let offset = match block {
                "21" => 0,
                "22" => 7,
                "23" => 14,
                "24" => 21,
                "25" => 28,
                _ => continue,
            };

            for i in 0..7 {
                let value = parts.next().ok_or(Error::InvalidResponse(900, format!("missing value for block {} cell {}", block, i)))?;
                result[offset + i] = (i32::from_str_radix(value, 16)? as f32) * 0.02
            }
        }

        Ok(result)
    }

    pub fn get_car_info(&mut self) -> Result<CarInfo, Error> {
        let cell_voltages = self.get_cell_voltages()?;
        Ok(CarInfo {
            time: std::time::SystemTime::now(),
            battery_info: self.get_battery_info()?,
            vin: "".to_string(),
            cell_voltages: CellVoltages(cell_voltages),
        })
    }
}

