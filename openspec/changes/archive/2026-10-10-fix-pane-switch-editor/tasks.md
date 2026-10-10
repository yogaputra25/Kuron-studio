## 1. Commit-on-switch (`CanvasEditor` + `EditorPanel`) — P0 — 0.2d

- [x] 1.1 `CanvasEditor.svelte`: expose `commitEditor` sebagai `export function` (pola sama seperti `deleteSelected`) — P0
- [x] 1.2 `EditorPanel.svelte`: tambah `afterRef` + `bind:this` pada instance After; handler `setMode(m)` panggil `afterRef?.commitEditor()` synchronous sebelum `mode = m`; switcher pakai `setMode` — P0
- [x] 1.3 Test pola-sumber: (a) `commitEditor` di-export, (b) `afterRef` ter-bind di instance After, (c) handler tab panggil commit sebelum assign `mode` — P0
- [x] 1.4 Canvas mentok pane: `redraw()` di-export + sinkronkan `bgImage.size()` (anti split-brain); `setMode` redraw kedua pane via `requestAnimationFrame` pasca-layout; `editorRef` type + `redraw` — P0

**Done bila:** skenario spec `pane-switch-commit` hijau pola-sumber.

## 2. Verifikasi + gates — P1 — 0.1d

- [x] 2.1 Manual E2E di app (`pnpm tauri dev`): Split → double-klik bubble → ketik → klik After → ketikan selamat + tak ada textarea basi + stage mentok pane (tak ada ruang kosong/hitam di kanan); ulang After → Split → Before; ganti tab tanpa editor = tanpa save — P1 (E2E user: hijau)
- [x] 2.2 Gates hijau (`pnpm test`, `pnpm check`; `cargo test`/`clippy` tak tersentuh bila tanpa perubahan Rust) — P1
- [x] 2.3 Arsip via `/opsx:archive` — P1
