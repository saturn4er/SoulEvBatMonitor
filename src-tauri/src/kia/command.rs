use std::collections::HashMap;
use error_stack::{Report, ResultExt};
use crate::elm327;
use crate::elm327::Command;
use crate::kia::{CellVoltages, Error};

// Commands information from https://github.com/langemand/SoulEVSpy/blob/master/app/src/main/java/com/evranger/soulevspy/util/BatteryManagementSystemParser.java

pub struct CellVoltagesCommand(pub String);

impl Command for CellVoltagesCommand {
    type Response = [f32; 35];
    fn serial_command(&self) -> String {
        return format!("21 0{}", self.0).to_string();
    }
    fn parse_result(&self, response: String) -> elm327::error::Result<Self::Response> {
        let frames = parse_multi_frame_response::<7>(response)?;
        let mut result = [0.0; 35];

        let frame_offsets = [("21", 0), ("22", 7), ("23", 14), ("24", 21), ("25", 28)];

        for (frame, offset) in frame_offsets {
            let values = frames.get(frame).ok_or(
                Report::new(elm327::Error::Other)
                    .attach_printable(format!("missing block {}", frame))
            )?;
            result[offset..offset + 7].clone_from_slice(&values.map(|v| (v as f32) * 0.02))
        }

        return Ok(result);
    }
}

#[derive(Default)]
pub struct BatteryInfo {
    pub charge_level: f64,
    pub charging: bool,
    pub chademo_plugged: bool,
    pub j1772_plugged: bool,
    pub battery_current: f64,
    pub battery_dc_voltage: f64,
    pub max_cell_voltage: f64,
    pub min_cell_voltage: f64,
    pub motor_speed: i32,
    pub module_temperatures: [i32; 7],
}

pub struct BatteryInfoCommand();

impl Command for BatteryInfoCommand {
    type Response = BatteryInfo;

    fn serial_command(&self) -> String {
        return "21 01".to_string();
    }
    fn parse_result(&self, response: String) -> elm327::error::Result<Self::Response> {
        let frames = parse_multi_frame_response::<7>(response)?;
        let frame_21 = frames.get("21").ok_or(Report::new(elm327::Error::Other).attach_printable("missing frame 21".to_string()))?;
        let frame_22 = frames.get("22").ok_or(Report::new(elm327::Error::Other).attach_printable("missing frame 22".to_string()))?;
        let frame_23 = frames.get("23").ok_or(Report::new(elm327::Error::Other).attach_printable("missing frame 23".to_string()))?;
        let frame_24 = frames.get("24").ok_or(Report::new(elm327::Error::Other).attach_printable("missing frame 24".to_string()))?;

        let mut result = BatteryInfo::default();

        let charging_flags = frame_21[5];
        result.charge_level = (frame_21[0] as f64) * 0.5;
        result.charging = (charging_flags & (1 << 7)) != 0;
        result.chademo_plugged = (charging_flags & (1 << 6)) != 0;
        result.j1772_plugged = (charging_flags & (1 << 5)) != 0;
        let (battery_dc_voltage, _) = frame_22[1].overflowing_shl(8);
        result.battery_dc_voltage = (battery_dc_voltage + frame_22[2]) as f64 * 0.1;
        result.min_cell_voltage = frame_23[0] as f64 * 0.02;
        result.max_cell_voltage = frame_24[0] as f64 * 0.02;
        result.module_temperatures = [
            frame_22[3],
            frame_22[4],
            frame_22[5],
            frame_22[6],
            frame_23[0],
            frame_23[1],
            frame_23[2],
        ];

        let msb = frame_21[6];
        if msb > 0 {
            result.battery_current = (msb.overflowing_shl(8).0 + frame_22[0]) as f64 * 0.1;
            if (msb & 0x80) != 0 {
                result.battery_current -= 6553.6;
            }
        }

        Ok(result)
    }
}


// returns a map of block number to array of 7 integers
// example:
//  response:
//    7E8 10 21 00 00 00 00 F2 00\n
//    7E8 21 00 00 00 00 00 00 00\n
//  output:
//    {
//      "10": [33, 0, 0, 0, 0, 242, 0],
//      "21": [0, 0, 0, 0, 0, 0, 0],
//    }

fn parse_multi_frame_response<const N: usize>(response: String) -> elm327::Result<HashMap<String, [i32; N]>> {
    response
        .split("\n")
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let mut parts = l.trim().split(" ").skip(1);
            let block = parts
                .next()
                .ok_or(Report::new(elm327::Error::Other)
                    .attach_printable(format!("Line does not contain block number: {}", l))
                )?;
            let int_result_parts = parts
                .map(|s| {
                    i32::from_str_radix(s, 16)
                        .attach_printable(format!("can't parse int32 from {s}"))
                        .change_context(elm327::Error::Other)
                })
                .collect::<elm327::Result<Vec<i32>>>()?;

            if int_result_parts.len() < N {
                return Err(Report::new(elm327::Error::Other)
                    .attach_printable(
                        format!("response {} has {} elements, when {} elements required", block, int_result_parts.len(), N)
                    ));
            }
            let mut result = [0; N];
            result.clone_from_slice(&int_result_parts.as_slice());

            Ok((block.to_string(), result))
        })
        .collect::<elm327::Result<HashMap<String, [i32; N]>>>()
}