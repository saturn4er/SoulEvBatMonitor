import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";
import {CarInfoHistoryContextProvider} from "./contexts/CarInfoHistory.tsx";
import {SelectedCellProvider} from "./contexts/SelectedCell.ts";
import {ConnectionContextProvider} from "contexts/Connection.tsx";
import {SelectedHistoryElementProvider} from "contexts/SelectedHistoryElement.ts";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
        <SelectedCellProvider>
            <SelectedHistoryElementProvider>
                <ConnectionContextProvider>
                    <CarInfoHistoryContextProvider>
                        <App/>

                    </CarInfoHistoryContextProvider>
                </ConnectionContextProvider>
            </SelectedHistoryElementProvider>
        </SelectedCellProvider>
    </React.StrictMode>,
);
