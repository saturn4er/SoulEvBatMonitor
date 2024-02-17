export type CarInfo = {
    time: Date;
    chargeLevel: number;
    charging: boolean;
    chademoPlugged: boolean;
    j1772Plugged: boolean;
    batteryCurrent: number;
    batteryDCVoltage: number;
    maxCellVoltage: number;
    minCellVoltage: number;
    msb: number;
    motorSpeed: number;
    cellsVoltages: number[];
}