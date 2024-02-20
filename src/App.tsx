import BatteryCells from "components/BatteryCells.tsx";
import Connect from "components/Connect.tsx";
import Chart from "components/Chart.tsx";
import {useCarInfoHistory} from "contexts/CarInfoHistory.tsx";
import {save, open} from "@tauri-apps/api/dialog";
import {readTextFile, writeFile} from "@tauri-apps/api/fs";
import {useSelectedHistoryElement} from "contexts/SelectedHistoryElement.ts";
import {useTranslation} from "react-i18next";

function App() {
    //async set resizeable
    const {t} = useTranslation();
    const {fetch, setFetch} = useCarInfoHistory();

    const {carInfoHistory, setCarInfoHistory} = useCarInfoHistory();
    const [selectedHistoryElement] = useSelectedHistoryElement();
    let cellVoltages = Array.from({length: 96}, () => 0);
    if (selectedHistoryElement) {
        cellVoltages = carInfoHistory[selectedHistoryElement].battery_info.cell_voltages;
    } else if (carInfoHistory.length > 0) {
        cellVoltages = carInfoHistory[carInfoHistory.length - 1].battery_info.cell_voltages;
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
            <div className={"flex flex-row px-2 py-2"}>
                <div className={"flex-grow min-w-0"}>
                    <Chart/>
                </div>
                <div className={"flex flex-col gap-[10px]"}>
                    <div className="flex flex-col gap-[3px]">
                        <button className="btn btn-sm btn-primary" onClick={() => {
                            setFetch(!fetch)
                        }}>{fetch ? t("stop") : t("start")}</button>
                        <button className="btn btn-sm btn-primary" onClick={loadHistory}>{t("load")}</button>
                        <button className="btn btn-sm btn-primary" disabled={carInfoHistory.length > 0}
                                onClick={saveHistory}>{t("save")}
                        </button>
                        <button className="btn btn-sm btn-primary" disabled={carInfoHistory.length > 0}
                                onClick={clearHistory}>{t("clear")}</button>
                    </div>
                    <div>
                        <BatteryCells cellVoltages={cellVoltages}></BatteryCells>
                    </div>
                </div>
            </div>
        </>
    );
}

export default App;
