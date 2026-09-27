# Tasks: dev-fresh-script

> P0 = Must, P1 = Should, P2 = Nice. Estimasi hari. Checkbox `- [ ]`.

## 1. Script inti — P0 — 0.5d

- [x] 1.1 Tulis `kuron-studio/scripts/dev-fresh.mjs` (Node stdlib only: cek port 1420 via `node:net`, hapus `node_modules/.vite` via `node:fs`, spawn `pnpm tauri dev` inherit stdio, print reminder `Ctrl+Shift+R`) — P0
- [x] 1.2 Tambah flag `--nuke`: print path lalu hapus profile WebView2 per-OS (`win32` → `%LOCALAPPDATA%\id.kuron.studio\EBWebView`, `darwin` → `~/Library/Application Support/id.kuron.studio/`, `linux` → `~/.local/share/id.kuron.studio/`); folder tidak ada → skip + pesan — P0
- [x] 1.3 Tambah script `dev:fresh` di `kuron-studio/package.json` (`node scripts/dev-fresh.mjs`); pastikan `-- --nuke` diteruskan — P0

**Done bila:** `pnpm dev:fresh --help`-ekuivalen jelas; tanpa flag tidak ada penghapusan selain `.vite`.

## 2. Guard data + verifikasi — P0 — 0.5d

- [x] 2.1 Buktikan closed-list: `node --check scripts/dev-fresh.mjs` + grep bahwa satu-satunya `rm` target adalah `.vite` dan profile dir; tidak ada referensi ke `%APPDATA%`/`projects.json`/`target/`/`dist/` — P0
- [x] 2.2 Uji abort: jalankan `pnpm dev` di satu terminal (pegang :1420), lalu `pnpm dev:fresh` di terminal lain → harus abort dengan pesan jelas sebelum menghapus apa pun; matikan pemegang port — P0
- [x] 2.3 Uji `--nuke` aman-data: backup `%APPDATA%\id.kuron.studio\kuron-studio\projects.json` checksum, jalankan `dev:fresh -- --nuke` (hentikan setelah webview naik via timeout/Ctrl+C), bandingkan checksum → identik — P0

**Done bila:** abort terbukti, checksum data identik, profile terhapus hanya dengan flag.

## 3. Docs + unblock verifikasi import — P1 — 0.25d

- [x] 3.1 Tambah ±3 baris di `kuron-studio/README.md` (kapan pakai `dev:fresh` vs `tauri dev`, apa yang dihapus `--nuke`, peringatan data tidak tersentuh) — P1
- [ ] 3.2 Jalankan `pnpm dev:fresh`, ulangi import (folder + zip + drag-drop) untuk menutup `fix-import-pages-project-id` 1.2/4.3; catat hasil di tasks change tersebut — P1

**Done bila:** README menjelaskan script; hasil verifikasi import tercatat (sembuh = stale cache terkonfirmasi, tetap error = lanjut debug non-cache).
