import React, { createContext, useContext, useState, ReactNode } from 'react';

export function wrapUseStateWithContext<T>(defaultValue: T) {
    const StateContext = createContext<[T, React.Dispatch<React.SetStateAction<T>>]>([defaultValue, () => {}]);

    const useCustomState = () => useContext(StateContext);

    const StateProvider: React.FC<{children: ReactNode}> = ({ children }) => {
        const state = useState<T>(defaultValue);
        return <StateContext.Provider value={state}>{children}</StateContext.Provider>;
    };

    return [useCustomState, StateProvider, StateProvider] as const;
}