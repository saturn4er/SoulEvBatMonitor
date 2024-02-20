import {useEffect, useState} from "react";
import {ConnectionMethod} from "models/ConnectionMethod.ts";
import {useConnection} from "contexts/Connection.tsx";
import {useCarInfoHistory} from "contexts/CarInfoHistory.tsx";
import {useTranslation} from "react-i18next";
import {tauri} from "@tauri-apps/api";

type ConnectWiFiSettingsProps = {
    address: string
    onAddressChange: (address: string) => void
}

function ConnectWiFiSettings({address, onAddressChange}: ConnectWiFiSettingsProps) {
    const {t} = useTranslation();
    return <div className={"flex flex-row items-center gap-2"}>
        {t("address")}
        <input type="text" value={address} onChange={(e) => {
            onAddressChange(e.target.value)
        }} className="input input-sm input-bordered w-full max-w-xs"/>
    </div>
}


type ConnectSerialSettingsProps = {
    onPortChange: (address: string) => void
}

function ConnectSerialSettings({onPortChange}: ConnectSerialSettingsProps) {
    const {t} = useTranslation();
    const [serialDevices, setSerialDevices] = useState<string[]>([]);

    tauri.invoke<string[]>("list_serial_devices", {}).then((devices) => {
        setSerialDevices(devices);
    })

    return <div className={"flex flex-row items-center gap-2"}>
        {t("device")}
        <select className="select select-sm select-bordered w-full max-w-xs" onChange={(e) => {
            onPortChange(e.target.value);
        }}>
            {serialDevices.map((device) => {
                return <option value={device}>{device}</option>
            })}
        </select>
    </div>
}


export default function Connect() {
    const {connect, disconnect, connecting, connectedDevice} = useConnection();
    const {fetch, setFetch} = useCarInfoHistory();
    const [connectionType, setConnectionMethod] = useState<ConnectionMethod>(ConnectionMethod.WIFI);
    const [connectionParam, setConnectionParam] = useState<string>("127.0.0.1:50059");//"192.168.0.10:35000");
    const {t} = useTranslation();

    const connectSettings: Record<ConnectionMethod, JSX.Element> = {
        [ConnectionMethod.WIFI]: <ConnectWiFiSettings address={connectionParam} onAddressChange={(value: string) => {
            setConnectionParam(value);
        }}/>,
        [ConnectionMethod.SERIAL]: <ConnectSerialSettings onPortChange={
            (value: string) => {
                setConnectionParam(value);
            }
        }/>
    }

    const handleConnect = () => {
        connect(connectionType, connectionParam).then((connectedDevice) => {
            console.log("we connected to " + connectedDevice);
        }).catch((e) => {
            console.log("we failed to connect: ", e);
        });
    }
    const handleDisconnect = () => {
        disconnect().then(() => {
            console.log("we disconnected");
        }).catch((e) => {
            console.log("we failed to disconnect: ", e);
        });
    }
    const updateConnectionMethod = (e: React.ChangeEvent<HTMLSelectElement>) => {
        switch (e.target.value) {
            case ConnectionMethod.WIFI.toString():
                setConnectionMethod(ConnectionMethod.WIFI);
                setConnectionParam("192.168.0.10:35000")
                break;
            case ConnectionMethod.SERIAL.toString():
                setConnectionMethod(ConnectionMethod.SERIAL);
                setConnectionParam("")
                break;

        }
    }

    return (
        <div className="card w-full card-compact bg-base-100 shadow-xl">
            <div className="card-body flex flex-row chil justify-start items-center">
                <select className="select select-sm select-bordered w-full max-w-xs" onChange={updateConnectionMethod}>
                    <option value={ConnectionMethod.WIFI}>WiFi</option>
                    <option disabled={true} value={ConnectionMethod.SERIAL}>Serial</option>
                </select>
                <div className={"grow flex items-center "}>
                    {connectSettings[connectionType]}
                </div>
                <div>
                    {connectedDevice && <span>{t('connected_to')} {connectedDevice}</span>}
                </div>
                <div className="card-actions justify-self-end">
                    {
                        connectedDevice ? <>
                                <button className="btn btn-sm btn-primary" disabled={connecting}
                                        onClick={handleDisconnect}>{connecting?t("connecting"):t("disconnect")}</button>
                            </>
                            :
                            <button className="btn btn-sm btn-primary" disabled={connecting}
                                    onClick={handleConnect}>{connecting?t("connecting"):t("connect")}</button>
                    }

                </div>
            </div>
        </div>
    );
}