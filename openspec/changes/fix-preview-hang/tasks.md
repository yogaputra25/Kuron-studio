# Tasks: fix-preview-hang

> P0 = Must, P1 = Should, P2 = Nice. Estimasi hari. Checkbox `- [ ]`.

## 1. Backend non-blocking — P0 — 0.25d

- [x] 1.1 `commands/image.rs`: jadikan `get_image_preview` `async` + bungkus read/decode/resize/encode dalam `tauri::async_runtime::spawn_blocking` (pola `detect.rs:99`); error String 1:1 — P0
- [x] 1.2 Jadikan test `missing_file_errors` async; `cargo test image` + `cargo clippy --all-targets -- -D warnings` hijau — P0

**Done bila:** preview besar tidak blokir runtime; wire key `{path, max_side}` tak berubah.

## 2. Progressive open — P0 — 0.25d

- [x] 2.1 `App.svelte` `openEditor`: set `selectedId` langsung (mount + thumb), fetch full 1600 paralel tanpa await-before-mount; resolve → `fullUrls` update → swap otomatis — P0 (lane-2)
- [x] 2.2 Gagal/timeout fetch → `openError` (reuse lane-2) tapi thumb tetap tampil (jangan sembunyikan canvas bila thumb ada) — P0 (lane-2)

**Done bila:** buka halaman besar → editor langsung tampil + thumb <1s, full menyusul.

## 3. Load defensif CanvasEditor — P0 — 0.25d

- [x] 3.1 `loadBackground`: timeout 15s (konstanta bernama, guard token per-request) → `imgError` + tombol "Coba lagi" tanpa remount — P0 (lane-2)
- [x] 3.2 Prop opsional `fallbackUrl` (thumb): full gagal/timeout → coba thumb otomatis sebelum menyerah — P0 (lane-2)
- [x] 3.3 Test vitest pola timeout/fallback (murah saja); `pnpm test && pnpm check` hijau — P0 (lane-2)

**Done bila:** tidak ada keadaan "Memuat gambar…" >15s; tiap gagal ada pesan + retry.

## 4. Verifikasi manual + gates + arsip — P1 — 0.1d

- [ ] 4.1 Manual: file `SI7pZ…png` → gambar muncul (atau thumb + pesan + retry bila backend lambat); halaman kecil tetap instan — P1
- [x] 4.2 Gates penuh hijau (`cargo test`, `clippy`, `pnpm test`, `pnpm check`) — P1 (cargo 68 passed/clippy clean; pnpm 32 tests pass/check 0 error, via lane-3)
- [ ] 4.3 Arsip `fix-preview-hang` (+ `fix-editor-blank-canvas` §1/§4.2 bila sembuh) via `/opsx:archive` — P1

## 5. Ganti structuredClone → $state.snapshot (DataCloneError) — P0 — 0.25d

Bug: `structuredClone()` tidak bisa clone Svelte 5 `$state` proxy →
`DataCloneError` di `CanvasEditor.svelte:47` tepat saat bubble non-kosong
(detect → 4 bubble → effect sync THROW → overlay tetap []). Sidebar sehat
karena state `EditorPanel` tidak lewat effect itu. Stack trace user konklusif.

- [x] 5.1 `CanvasEditor.svelte`: ganti 6× `structuredClone` → `$state.snapshot` (effect sync + 5× `onChange`) — P0
- [x] 5.2 `EditorPanel.svelte`: ganti 2× `structuredClone` → `$state.snapshot` (`syncFromPage` + restore translation) — P0
- [x] 5.3 Tambah kasus vitest "snapshot array proxy non-kosong tidak throw" (pola-sumber, gaya `canvas-editor.test.ts`) — P0
- [x] 5.4 `pnpm test && pnpm check` hijau — P0

**Done bila:** buka halaman → detect → 4 kotak biru tampil tanpa console error; gates hijau.

## 6. Auto-save setelah hapus bubble — P1 — 0.25d

Bug: tombol hapus sidebar hanya `splice` lokal + `dirty = true` — tidak panggil
`onSaved`/`save()`. Prop `page` (milik App) tetap berisi N bubble, sehingga
`syncFromPage()` me-reset `bubbles` ke N saat effect jalan ulang (ketik,
ganti tool, dsb) → bubble "muncul lagi". Canvas juga tidak tahu ada yang
dihapus sampai save manual. Guard JSON tidak boleh di-disable (melindungi
reset saat ngetik) — yang diperbaiki adalah jalur hapus, bukan guard.

- [x] 6.1 `EditorPanel.svelte` tombol hapus sidebar: setelah `splice`, panggil `save()` (persist + `onSaved` → prop ikut N-1 → guard diam) — P1
- [x] 6.2 `deleteSelected()` (keyboard, via `CanvasEditor` → `onChange`): pastikan `onChange` sudah memicu persist juga — bila belum, panggil `save()` di handler `onChange` khusus delete, atau jadikan delete keyboard auto-save seperti 6.1 — P1
- [x] 6.3 Selama `save()` in-flight (saving=true): disable tombol hapus + Delete key (cegah race splice ganda); error save tampil di banner yang sudah ada — P1
- [x] 6.4 Test vitest pola-sumber (hapus → save dipanggil; guard tidak me-reset pasca-save) + `pnpm test && pnpm check` hijau — P1

**Done bila:** klik hapus → list berkurang permanen (tutup → buka lagi tetap N-1); canvas ikut update; tidak ada "muncul lagi".
