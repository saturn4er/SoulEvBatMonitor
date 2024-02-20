import {createContext, useContext, useEffect, useState} from 'react';
import {CarInfo} from "models/CarInfo.ts";
import {useConnection} from "contexts/Connection.tsx";

type CarInfoHisoryContextValue = {
    carInfoHistory: CarInfo[];
    setCarInfoHistory: (carInfo: CarInfo[]) => void;
    addCarInfo: (carInfo: CarInfo) => void;
    clearCarInfo: () => void;
    fetch: boolean,
    setFetch: (fetch: boolean) => void;
}

export const CarInfoHistoryContext = createContext<CarInfoHisoryContextValue>({
    carInfoHistory: [],
    setCarInfoHistory: () => {
    },
    addCarInfo: () => {
    },
    clearCarInfo: () => {
    },
    fetch: false,
    setFetch: () => {
    }
});

export const useCarInfoHistory = () => {
    return useContext(CarInfoHistoryContext);
}


export function CarInfoHistoryContextProvider({children}: { children: React.ReactNode }) {
    const defaultValues = Array.from({length: 0}, (_, index) => {
        let d = new Date();
        d.setDate((new Date()).getDate() + index * 10000)
        return {
            time: d,
            chargeLevel: 10,
            charging: true,
            chademoPlugged: true,
            j1772Plugged: true,
            batteryCurrent: 10,
            batteryDCVoltage: 10,
            maxCellVoltage: 10,
            minCellVoltage: 10,
            msb: 10,
            motorSpeed: 10,
            cellsVoltages: Array.from({length: 96}, () => Math.random() * 2.2 + 2)
        };
    });
    const [carInfo, setCarInfo] = useState<CarInfo[]>(defaultValues);
    const [fetch, setFetch] = useState<boolean>(false);
    const addCarInfo = (carInfo: CarInfo) => {
        setCarInfo((prevCarInfo) => {
            return [...prevCarInfo, carInfo];
        });
    }
    const clearCarInfo = () => {
        setCarInfo([]);
    }

    const {connectedDevice, reconnect, getCarInfo} = useConnection()
    useEffect(() => {
        if (connectedDevice) {
            const interval = setInterval(async () => {
                if (fetch) {
                    try {
                        let carInfo = await getCarInfo();
                        console.log(carInfo);
                        if(carInfo.battery_info.cell_voltages.some((v) => v == 0)){
                            return
                        }

                        addCarInfo(carInfo);
                    } catch (e) {
                        console.log(e)
                        clearInterval(interval);
                        try {
                            console.log("reconnecting")
                            await reconnect()
                        } catch (e) {
                            console.log("reconnect failed")
                        }
                    }
                }
            }, 1000)
            return () => clearInterval(interval)
        }
    }, [connectedDevice, fetch])


    return <CarInfoHistoryContext.Provider
        value={{
            carInfoHistory: carInfo,
            addCarInfo,
            clearCarInfo,
            setCarInfoHistory: setCarInfo,
            fetch,
            setFetch,
        }}>{children}</CarInfoHistoryContext.Provider>
}
