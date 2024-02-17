import {createContext, useContext, useState} from 'react';
import {CarInfo} from "models/CarInfo.ts";
import {tauri} from "@tauri-apps/api";
import {ConnectionMethod} from "models/ConnectionMethod.ts";

type ConnectionContextValue = {
    connectedDevice: String | null,
    connect: (connectionMethod: ConnectionMethod, params: any) => Promise<string>;
    getCarInfo: () => Promise<CarInfo>;
}

export const ConnectionContext = createContext<ConnectionContextValue>({
    connectedDevice: null,
    connect: (_cm: ConnectionMethod, _params: any) => Promise.reject("context not initialized"),
    getCarInfo: () => Promise.reject("context not initialized"),
});

export const useConnection = () => {
    return useContext(ConnectionContext);
}

export function ConnectionContextProvider({children}: { children: React.ReactNode }) {
    let [connectedDevice, setConnectedDevice] = useState<string | null>(null)
    const value = {
        connectedDevice: connectedDevice,
        connect: async (cm: ConnectionMethod, _params: string) => {
            try {
                const connectedDevice: string = await tauri.invoke<string>("connect", {
                    connectionMethod: cm,
                    connectionParam: _params
                });
                console.log("we connected to " + connectedDevice);
                setConnectedDevice(connectedDevice);

                return Promise.resolve(connectedDevice);
            } catch (e) {
                return Promise.reject(e);
            }
        },
        getCarInfo: async () => {
            const rustCarInfo = await tauri.invoke<any>("get_car_info", {});
            return Promise.resolve({
                time: new Date(rustCarInfo.time.secs_since_epoch * 1000),
                chargeLevel: rustCarInfo.charge_level,
                charging: rustCarInfo.charging,
                chademoPlugged: rustCarInfo.chademo_plugged,
                j1772Plugged: rustCarInfo.j_1772_plugged,
                batteryCurrent: rustCarInfo.battery_current,
                batteryDCVoltage: rustCarInfo.battery_dcvoltage,
                maxCellVoltage: rustCarInfo.max_cell_voltage,
                minCellVoltage: rustCarInfo.min_cell_voltage,
                msb: rustCarInfo.msb,
                motorSpeed: rustCarInfo.motor_speed,
                cellsVoltages: rustCarInfo.cell_voltages,
            });

        },
    }

    return <ConnectionContext.Provider
        value={value}>{children}
    </ConnectionContext.Provider>
}
