import CellInfo from "./CellInfo.tsx";

type CellsBlockProps = {
    fromCell: number;
    toCell: number;
    cellVoltages: number[];
    goodVoltage: number;
    badVoltage: number;
    onClick?: (cellNumber: number) => void;
}

export default function CellsBlock({
                                       fromCell,
                                       toCell,
                                       cellVoltages,
                                       goodVoltage,
                                       badVoltage,
                                       onClick
                                   }: CellsBlockProps) {
    const rowsCount = (toCell + 1 - fromCell) / 2
    return <div className={"flex flex-col gap-[3px]"}>
        {Array.from({length: rowsCount}, (_, i) => i).map((_, i) => {
            return <div key={i} className={"flex flex-row gap-[10px]"}>
                <CellInfo
                    cellNumber={toCell - i} voltage={cellVoltages[cellVoltages.length - 1 - i]}
                    goodVoltage={goodVoltage}
                    badVoltage={badVoltage}
                    onClick={() => onClick && onClick(toCell - i)}
                />
                <CellInfo cellNumber={fromCell + i} voltage={cellVoltages[i]}
                          goodVoltage={goodVoltage}
                          badVoltage={badVoltage}
                          onClick={() => onClick && onClick(fromCell + i)}
                />
            </div>
        })}
    </div>

}