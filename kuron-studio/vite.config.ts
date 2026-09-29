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
  // Di bawah Vitest, resolve Svelte ke build *browser*. Tanpa ini kita dapat
  // index-server.js dan `mount(...)` melempar lifecycle_function_unavailable —
  // test runtime komponen (field-roundtrip) mustahil jalan.
  resolve: process.env.VITEST ? { conditions: ["browser"] } : undefined,
  test: { environment: "node", include: ["src/**/*.{test,spec}.ts"] },
});
