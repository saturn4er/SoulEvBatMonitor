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
    const [carInfo, setCarInfo] = useState<CarInfo[]>([]);
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
