import classNames from "classnames";
import {useSelectedCell} from "contexts/SelectedCell.ts";

type CellsInfoProps = {
    cellNumber: number;
    voltage: number;
    className?: string;
    goodVoltage: number;
    badVoltage: number;
    onClick?: () => void;
}

export default function CellInfo({cellNumber, voltage, className, goodVoltage, badVoltage, onClick}: CellsInfoProps) {
    const [seletedCell, setSelectedCell] = useSelectedCell();
    let color = "bg-gray-600";
    if (voltage > goodVoltage) {
        color = "bg-green-800";
    }
    if (voltage < badVoltage) {
        color = "bg-red-800";
    }

    const _onClick = () => {
        setSelectedCell(cellNumber-1);
        onClick && onClick();
    }

    return <div onClick={_onClick} className={
        classNames([
            "cursor-pointer",
            "flex",
            "flex-row",
            "gap-1",
            "h-[25px]",
            className,
        ])}>
        <span className={"w-[26px]"}>
        {cellNumber}:
        </span>
        <span className={classNames(["rounded-md ", color,
            seletedCell === (cellNumber-1) ? "border-[2px] border-blue-500" : ""])}>
            {voltage != 0 && voltage.toFixed(2)}
        </span>
    </div>
}