// dev:fresh — dev-loop tanpa cache (Node stdlib only, no new deps).
//
// Default: hapus Vite optimizer cache (node_modules/.vite), cek port 1420
// bebas, lalu spawn `pnpm tauri dev`. Setelah webview naik: Ctrl+Shift+R.
// --nuke: + hapus profile WebView2 dev (print path dulu, default OFF).
//
// Closed-list: script ini TIDAK PERNAH menyentuh %APPDATA%/id.kuron.studio
// (projects.json, kuron-studio.db = DATA user) dan TIDAK PERNAH wipe target/.
// Lihat openspec/changes/dev-fresh-script/specs/dev-fresh/spec.md.
import { spawn } from "node:child_process";
import { existsSync, rmSync } from "node:fs";
import net from "node:net";
import { homedir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const PORT = 1420;
const NUKE = process.argv.includes("--nuke");

function webviewProfileDir() {
  // Terverifikasi di Windows: %LOCALAPPDATA%\id.kuron.studio\EBWebView.
  // macOS/Linux: belum terverifikasi di mesin — folder tak ada => skip.
  if (process.platform === "win32" && process.env.LOCALAPPDATA) {
    return join(process.env.LOCALAPPDATA, "id.kuron.studio", "EBWebView");
  }
  if (process.platform === "darwin") {
    return join(homedir(), "Library", "Application Support", "id.kuron.studio");
  }
  return join(homedir(), ".local", "share", "id.kuron.studio");
}

function tryListen(port, host) {
  return new Promise((resolveFree) => {
    const s = net
      .createServer()
      .once("error", () => resolveFree(false))
      .once("listening", () => s.close(() => resolveFree(true)))
      .listen(port, host);
  });
}

async function portFree(port) {
  // Probe both stacks: a Vite holder on this machine binds [::1] (IPv6),
  // which a 127.0.0.1-only probe misses (Windows default dual-stack).
  return (
    (await tryListen(port, "127.0.0.1")) && (await tryListen(port, "::1"))
  );
}

const viteCache = join(ROOT, "node_modules", ".vite");
if (!(await portFree(PORT))) {
  console.error(
    `dev:fresh abort: port ${PORT} sudah dipakai (Vite zombie?).\n` +
      `  Matikan proses pemegang port dulu, lalu ulangi. Tidak ada yang dihapus.`,
  );
  process.exit(1);
}
if (existsSync(viteCache)) {
  rmSync(viteCache, { recursive: true, force: true });
  console.log(`dev:fresh: dihapus ${viteCache}`);
} else {
  console.log("dev:fresh: node_modules/.vite tidak ada — skip.");
}
if (NUKE) {
  const profile = webviewProfileDir();
  console.log(`dev:fresh --nuke: target profile ${profile}`);
  if (existsSync(profile)) {
    rmSync(profile, { recursive: true, force: true });
    console.log("dev:fresh --nuke: profile WebView2 dihapus.");
  } else {
    console.log("dev:fresh --nuke: folder tidak ada — skip.");
  }
}
console.log("dev:fresh: menjalankan `pnpm tauri dev` …");
console.log("dev:fresh: setelah webview naik, tekan Ctrl+Shift+R (hard reload).");
const child = spawn("pnpm", ["tauri", "dev"], {
  cwd: ROOT,
  stdio: "inherit",
  shell: process.platform === "win32",
});
child.on("exit", (code) => process.exit(code ?? 0));
