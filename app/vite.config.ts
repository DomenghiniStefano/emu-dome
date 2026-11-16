import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Configuration made for Tauri: fixed port, no browser opening.
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  envPrefix: ["VITE_", "TAURI_"],
  build: { target: "es2022", outDir: "dist" },
});
