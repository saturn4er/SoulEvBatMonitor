import {Line} from 'react-chartjs-2';
import {useCarInfoHistory} from "contexts/CarInfoHistory.tsx";
import 'chart.js/auto';
import {useSelectedCell} from "contexts/SelectedCell.ts";
import {ActiveElement} from "chart.js";
import type {ChartDataset} from "chart.js"
import {useSelectedHistoryElement} from "contexts/SelectedHistoryElement.ts";


function formatDate(xTime: Date) {
    const minutes = String(xTime.getMinutes()).padStart(2, '0');
    const seconds = String(xTime.getSeconds()).padStart(2, '0');
    return `${minutes}:${seconds}`
}

export default function Chart() {
    const {carInfoHistory} = useCarInfoHistory();
    const [_selectedHistoryElement, setSelectedHistoryElement] = useSelectedHistoryElement()
    const [selectedCell, setSelectedCell] = useSelectedCell();

    let cellsDataset: ChartDataset<"line">[];
    try {
        cellsDataset = Array.from({length: 96}, (_, index) => {
            let data = carInfoHistory.map((carInfo) => {
                return +carInfo.battery_info.cell_voltages[index].toFixed(2);
            });
            // myChart.data.datasets[i].borderColor = 'rgba(255, 0, 0, 1)'
            // myChart.data.datasets[i].order = 100;
            let color = 'rgba(122,122,122,0.2)';
            if (selectedCell) {
                color = selectedCell == index ? 'rgba(255, 0, 0, 1)' : 'rgba(36,36,36,0.2)';
            }
            return {
                label: 'Cell ' + (index + 1),
                data: data,
                borderColor: color,
                order: selectedCell == index ? 100000 : 0,
            }
        })
    } catch (error) {
        cellsDataset = []
    }

    const powerDataset: ChartDataset<"line">[] = [
        {
            label: 'Power',
            data: carInfoHistory.map((carInfo) => carInfo.battery_info.battery_current),
            borderColor: 'rgba(0, 0, 255, 1)',
        }
    ]
    const datasets: ChartDataset<"line">[] = [
        ...cellsDataset,
        ...powerDataset,
    ]

    return <Line
        redraw={false}
        data={{
            labels: carInfoHistory.map((carInfo) => formatDate(new Date(carInfo.time))),
            datasets: datasets,
        }}
        options={{
            plugins: {
                colors: {
                    enabled: true
                },
                legend: {
                    display: false
                },
            },
            onClick(_e, elements: ActiveElement[], _c) {
                if (elements.length > 0) {
                    setSelectedCell(elements[0].datasetIndex);
                    setSelectedHistoryElement(elements[0].index);
                } else {
                    setSelectedCell(null);
                    setSelectedHistoryElement(null);
                }
            }
        }}
    />
}