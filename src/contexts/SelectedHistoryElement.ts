import {wrapUseStateWithContext} from "../utils/wrapUseStateWithContext.tsx";

const [
    useSelectedHistoryElement,
    setSelectedHistoryElement,
    SelectedHistoryElementProvider
] = wrapUseStateWithContext<number | null>(null)

export {useSelectedHistoryElement, setSelectedHistoryElement, SelectedHistoryElementProvider}