import {defineConfig} from "vite";
import react from "@vitejs/plugin-react";
import {resolve} from "path";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
    plugins: [react()],

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
        host: true,
        port: 3048,
        strictPort: true,
        watch: {
            // 3. tell vite to ignore watching `src-tauri`
            ignored: ["**/src-tauri/**"],
        },
    },
    resolve: {
        alias: [
            {find: "components", replacement: resolve("./src/components")},
            {find: "contexts", replacement: resolve("./src/contexts")},
            {find: "utils", replacement: resolve("./src/utils")},
            {find: "models", replacement: resolve("./src/models")},
            {find: "hooks", replacement: resolve("./src/hooks")},
        ],
    }
}));
