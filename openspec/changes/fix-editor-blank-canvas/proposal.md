## Why

Editor terbuka dengan body hitam polos — gambar tidak tampil, kotak bubble tidak
tampil — padahal sidebar membuktikan data sehat: status `detected`, `Bubble (4)`
dengan daftar `rect 170×220` dkk. User mengira detect gagal, padahal yang gagal
murni rendering `CanvasEditor.svelte`. Ini P0 UX blocker: editor adalah layar
kerja utama, dan kondisi ini tidak bisa di-diagnose user (tidak ada pesan error
yang terlihat — error `openEditor` dirender di grid tapi tertutup overlay
`fixed inset-0 z-50`).

Dua akar yang teridentifikasi (urut kemungkinan):

1. **Race `imageUrl` vs `onMount` (kode lama, kemungkinan besar).**
   `App.openEditor` (`App.svelte:103-112`) set `selectedId` DULU, fetch
   `get_image_preview(path, 1600)` async SESUDAHNYA. Saat `EditorPanel` +
   `CanvasEditor` mount, prop `fullImageUrl` masih `""`. Kode lama membuat
   `new Konva.Stage` DI DALAM `img.onload` — dengan `src=""`, `onload` tidak
   pernah fire → stage null selamanya → `redraw()` early-return → hitam total.
   Catatan: fix parsial sudah ada di working tree (stage dibuat di `onMount` +
   `$effect` swap background), tapi screenshot user (tanpa kotak biru overlay,
   tanpa hint "Memuat gambar…") konsisten dengan bundle lama yang masih jalan
   — belum terkonfirmasi user sudah restart `dev:fresh` setelah fix.
2. **Error preview tersembunyi.** `openEditor` catch → `error` diset di
   `App.svelte:212`, tapi banner itu ada di grid di BAWAH overlay editor
   (`z-50`), jadi user tidak pernah melihatnya. Editor yang kehabisan URL stuck
   "Memuat gambar…" selamanya tanpa jalan keluar selain tutup manual.

Kait ke `docs/ide.md` (editor canvas sebagai core loop) dan `docs/target.md`
(M0: import → grid → editor tampil).

## Scope

- `kuron-studio/src/lib/CanvasEditor.svelte` — verifikasi fix race sudah di
  working tree bekerja (stage di `onMount`, `$effect` swap background saat
  `imageUrl` tiba, guard request basi, hint loading/error); tambah bila kurang.
- `kuron-studio/src/App.svelte` — error `openEditor` harus terlihat DARI DALAM
  editor (teruskan ke `EditorPanel` atau render banner di atas overlay), plus
  tombol "tutup/kembali" saat gambar gagal dimuat agar tidak stuck.
- Regression test frontend untuk kontrak ini (vitest, tanpa Konva asli bila
  perlu — assert alur prop/URL, bukan pixel).

## Non-goals

- Tidak ubah pipeline detect/ONNX (sudah sehat — 4 bubble terbukti).
- Tidak ubah `get_image_preview` backend (thumbnail grid tampil = backend sehat).
- Tidak tambah command Tauri baru, tidak ubah struktur `Page`/`Project`.
- Tidak refactor `redraw()` overlay (append-only, sesuai komentar kode).

## Risks

- **Stale bundle menutupi verifikasi** — screenshot bisa saja kode lama.
  Mitigasi: langkah verifikasi pertama SELALU restart `pnpm dev:fresh`
  sebelum menyimpulkan fix gagal.
- **Konva crash saat init** (`holder` undefined) memberi gejala sama (hitam
  total). Mitigasi: minta console DevTools webview bila restart tidak mempan.
- **Scope creep ke UX editor umum** (zoom, pan, dsb). Mitigasi: change ini
  hanya blank-canvas + error visibility, sisanya change terpisah.
