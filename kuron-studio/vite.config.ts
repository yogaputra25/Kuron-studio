import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";

// ponytail: SPA, bukan SvelteKit — M0 hanya butuh satu build target (webview Tauri).
// Upgrade ke SvelteKit (adapter-static, ssr=false) saat butuh file-based routing.
export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { outDir: "dist" },
  test: { environment: "node", include: ["src/**/*.{test,spec}.ts"] },
});
