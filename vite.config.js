import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Interface dans ui/, construite dans dist/ puis embarquée par Tauri (src-tauri/tauri.conf.json).
export default defineConfig({
  root: "ui",
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: { outDir: "../dist", emptyOutDir: true },
});
