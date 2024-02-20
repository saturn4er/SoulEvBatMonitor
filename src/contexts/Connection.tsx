import {createContext, useContext, useState} from 'react';
import {CarInfo} from "models/CarInfo.ts";
import {tauri} from "@tauri-apps/api";
import {ConnectionMethod} from "models/ConnectionMethod.ts";

type ConnectionParams = {
    connectionMethod: ConnectionMethod,
    connectionParam: string
}
type ConnectionContextValue = {
    connecting: boolean,
    connectedDevice: String | null,
    connect: (connectionMethod: ConnectionMethod, params: string) => Promise<string>;
    disconnect: () => Promise<void>;
    reconnect: () => Promise<string>;
    getCarInfo: () => Promise<CarInfo>;
}

export const ConnectionContext = createContext<ConnectionContextValue>({
    connecting: false,
    connectedDevice: null,
    connect: (_cm: ConnectionMethod, _params: any) => Promise.reject("context not initialized"),
    disconnect: () => Promise.reject("context not initialized"),
    reconnect: () => Promise.reject("context not initialized"),
    getCarInfo: () => Promise.reject("context not initialized"),
});

export const useConnection = () => {
    return useContext(ConnectionContext);
}

export function ConnectionContextProvider({children}: { children: React.ReactNode }) {
    let [connecting, setConnecting] = useState<boolean>(false)
    let [connectedDevice, setConnectedDevice] = useState<string | null>(null)
    let [lastConnectionParams, setLastConnectionParams] = useState<ConnectionParams | null>(null)
    const value = {
        connecting,
        connectedDevice: connectedDevice,
        connect: async (cm: ConnectionMethod, _params: string) => {
            setConnecting(true);
            setConnectedDevice(null);
            setLastConnectionParams({
                connectionMethod: cm,
                connectionParam: _params
            })
            try {
                const connectedDevice: string = await tauri.invoke<string>("connect", {
                    connectionMethod: cm,
                    connectionParam: _params
                });
                console.log("we connected to ", connectedDevice);
                setConnecting(false)
                setConnectedDevice(connectedDevice);

                return Promise.resolve(connectedDevice);
            } catch (e) {
                setConnecting(false)
                setConnectedDevice(null);
                return Promise.reject(e);
            }
        },
        disconnect: async () => {
            try {
                await tauri.invoke("disconnect", {});
                setConnectedDevice(null);
            } catch (e) {
                return Promise.reject(e);
            }
        },
        reconnect: async () => {
            if (lastConnectionParams) {
                return await value.connect(lastConnectionParams.connectionMethod, lastConnectionParams.connectionParam);
            }
            return Promise.reject("No previous connection");
        },
        getCarInfo: async () => {
            return await tauri.invoke<CarInfo>("get_car_info", {});
        },
    }

    return <ConnectionContext.Provider
        value={value}>{children}
    </ConnectionContext.Provider>
}
