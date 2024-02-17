import CellsBlock from "./CellsBlock.tsx";

type ConnectProps = {
    cellVoltages: number[];
    onCellClick?: (cellNumber: number) => void;
}
export default function BatteryCells({cellVoltages}: ConnectProps) {
    const avgVoltage = cellVoltages.reduce((a, b) => a + b, 0) / cellVoltages.length
    const avgDiffFromAvg = cellVoltages.map(value => Math.abs(value - avgVoltage)).reduce((a, b) => a + b, 0) / cellVoltages.length
    const badVoltage = avgVoltage - avgDiffFromAvg;
    const goodVoltage = avgVoltage + avgDiffFromAvg;

    let cellBlocks = [
        [[35, 48], [49, 62]],
        [[25, 34], [63, 72]],
        [[15, 24], [73, 82]],
        [[1, 14], [83, 96]],
    ]
    return <div>
        <div className={"flex flex-col  gap-5"}>
            <p>
                Average voltage: {avgVoltage.toFixed(2)}
                <br/>
                <span className={"text-green-600"}>&gt; {goodVoltage.toFixed(2)}</span>
                <br/>
                <span className={"text-red-600"}>&lt; {badVoltage.toFixed(2)}</span>
            </p>
            {cellBlocks.map((block, i) => {
                return <div key={i} className={"flex flex-row gap-[20px]"}>
                    <CellsBlock fromCell={block[0][0]} toCell={block[0][1]}
                                cellVoltages={cellVoltages.slice(block[0][0] - 1, block[0][1])}
                                badVoltage={badVoltage} goodVoltage={goodVoltage}
                                onClick={cellNumber => {console.log(cellNumber)}}
                    />
                    <CellsBlock fromCell={block[1][0]} toCell={block[1][1]}
                                cellVoltages={cellVoltages.slice(block[1][0] - 1, block[1][1])}
                                badVoltage={badVoltage} goodVoltage={goodVoltage}
                                onClick={cellNumber => {console.log(cellNumber)}}
                    />
                </div>
            })}
        </div>
    </div>
}