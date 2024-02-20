import {wrapUseStateWithContext} from "../utils/wrapUseStateWithContext.tsx";

const [
    useAutomaticReconnect,
    setAutomaticReconnect,
    AutomaticReconnectProvider
] = wrapUseStateWithContext<boolean>(true)

export {useAutomaticReconnect, setAutomaticReconnect, AutomaticReconnectProvider}