import {useState} from "react";
import {ConnectionMethod} from "models/ConnectionMethod.ts";
import {useConnection} from "contexts/Connection.tsx";
import {useCarInfoHistory} from "contexts/CarInfoHistory.tsx";

function ConnectWiFiSettings() {
    return <div>
        Address
    </div>
}

function ConnectTestSettings() {
    return <></>
}


export default function Connect() {
    const {connect} = useConnection();
    const {fetch, setFetch} = useCarInfoHistory();
    const [connectionType, setConnectionMethod] = useState<ConnectionMethod>(ConnectionMethod.WIFI);

    const connectSettings : Record<ConnectionMethod, JSX.Element> = {
        [ConnectionMethod.WIFI]: <ConnectWiFiSettings/>,
        [ConnectionMethod.HISTORY]: <ConnectTestSettings/>,
    }

    const handleClick = () => {
        connect(ConnectionMethod.WIFI, "127.0.0.1:50059").then((connectedDevice) => {
            console.log("we connected to " + connectedDevice);
        }).catch((e) => {
            console.log("we failed to connect: " + e);
        });
    }
    const updateConnectionMethod = (e: React.ChangeEvent<HTMLSelectElement>) => {
        switch (e.target.value) {
            case "wifi":
                setConnectionMethod(ConnectionMethod.WIFI);
                break;
            case "history":
                setConnectionMethod(ConnectionMethod.HISTORY);
                break;
        }
    }

    return (
        <div className="card w-full bg-base-100 shadow-xl">
            <div className="card-body flex flex-row chil justify-start">
                <h2 className="card-title">Connect</h2>
                <select className="select select-bordered w-full max-w-xs" onChange={updateConnectionMethod}>
                    <option value={ConnectionMethod.WIFI}>WiFi</option>
                    <option value={ConnectionMethod.HISTORY}>Test data</option>
                </select>
                <div className={"grow"}>
                {connectSettings[connectionType]}
                </div>
                <div className="card-actions justify-self-end">
                    <button className="btn btn-primary" onClick={handleClick}>Connect</button>
                    <button className="btn btn-primary" onClick={()=>{setFetch(!fetch)}}>{fetch?"Stop":"Start"}</button>
                </div>
            </div>
        </div>
    );
}