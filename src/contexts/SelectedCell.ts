import {wrapUseStateWithContext} from "../utils/wrapUseStateWithContext.tsx";

const [
    useSelectedCell,
    setSelectedCell,
    SelectedCellProvider
] = wrapUseStateWithContext<number | null>(null)

export {useSelectedCell, setSelectedCell, SelectedCellProvider}