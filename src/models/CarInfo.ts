export type BatteryInfo = {
    charge_level: number;
    charging: boolean;
    chademo_plugged: boolean;
    j1772_plugged: boolean;
    battery_current: number;
    battery_dc_voltage: number;
    max_cell_voltage: number;
    min_cell_voltage: number;
    motor_speed: number;
    cell_voltages: number[];
    module_temperatures: number[]
}

export type CarInfo = {
    time: number;
    battery_info: BatteryInfo;
}