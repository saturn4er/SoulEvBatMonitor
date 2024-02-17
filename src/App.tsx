import {appWindow} from '@tauri-apps/api/window';
import {useEffect} from "react";
import BatteryCells from "components/BatteryCells.tsx";
import Connect from "components/Connect.tsx";
import Chart from "components/Chart.tsx";
import {useCarInfoHistory} from "contexts/CarInfoHistory.tsx";
import {save, open} from "@tauri-apps/api/dialog";
import {readTextFile, writeFile} from "@tauri-apps/api/fs";
import {useSelectedHistoryElement} from "contexts/SelectedHistoryElement.ts";

function App() {
    //async set resizeable
    useEffect(() => {
        appWindow.setResizable(true);
    }, []);
    const {carInfoHistory, setCarInfoHistory} = useCarInfoHistory();
    const [selectedHistoryElement] = useSelectedHistoryElement();
    let cellVoltages = Array.from({length: 96}, () => 0);
    if(selectedHistoryElement) {
        cellVoltages = carInfoHistory[selectedHistoryElement].cellsVoltages;
    }else if (carInfoHistory.length > 0) {
        cellVoltages = carInfoHistory[carInfoHistory.length-1].cellsVoltages;
    }
    const saveHistory = async () => {
        const filePath = await save({
            filters: [{
                name: 'SoulHistory',
                extensions: ['json']
            }]
        });
        if (filePath) {
            console.log(await writeFile(filePath, JSON.stringify(carInfoHistory), {}));
        }
    }
    const loadHistory = async () => {
        const selected = await open({
            multiple: false,
            filters: [{
                name: 'SoulHistory',
                extensions: ['json']
            }]
        });
        if (typeof selected == "string") {
            readTextFile(selected).then((content) => {
                let data = JSON.parse(content).map((carInfo: any) => {
                    carInfo.time = new Date(carInfo.time);
                    return carInfo;
                });
                setCarInfoHistory(data);
            })
        }
    }
    const clearHistory = () => {
        setCarInfoHistory([]);
    }

    return (
        <>
            <Connect/>
            <div>
                <div className="card-actions justify-self-end">
                    <button className="btn btn-primary" onClick={saveHistory}>Save
                    </button>
                    <button className="btn btn-primary" onClick={loadHistory}>Load
                    </button>
                    <button className="btn btn-primary" onClick={clearHistory}>Clear
                    </button>
                </div>
            </div>
            <div className={"flex flex-col lg:flex-row  p-10"}>
            <div className={"grow"}>
                    <Chart/>
                </div>

                <BatteryCells cellVoltages={cellVoltages}></BatteryCells>
            </div>
        </>
    );
}

export default App;
