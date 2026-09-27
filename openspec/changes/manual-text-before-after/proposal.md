## Why

Dua kebutuhan kerja nyata di editor: (1) user mau isi/koreksi teks bubble **manual per bubble terpilih tanpa panggil AI** — hari ini `original`/`reading` read-only (`<p>`) dan hanya terisi dari AI, sehingga bubble yang gagal dibaca AI tidak bisa dikerjakan; (2) user mau **lihat before/after berdampingan dan scroll bareng** untuk review akhir — hari ini cuma satu pane (gambar asli + overlay toggle), tidak bisa bandingkan posisi teks vs artwork sekaligus.

Kait ke `docs/ide.md` (editor canvas core loop) dan `docs/target.md` (M0: import → grid → editor tampil; workbench produksi untuk provider).

## What Changes

- Sidebar editor: field `original` (dan `reading`) bubble terpilih jadi textarea editable; tersimpan via jalur `saveTranslation` yang sudah ada.
- `preserve_user_edits` diperluas: `original`/`reading` yang `isUserEdited` tidak ditimpa translate ulang (hari ini cuma `translated` yang dilindungi).
- Tombol "isi manual" bila halaman/bubble belum punya `PageTranslation`: buat translation kosong lokal → `save_translation`.
- Body editor dapat switcher `Before | Split | After`; mode Split menampilkan dua pane berdampingan: kiri gambar asli + kotak (before), kanan gambar sama + overlay teks (after-lite).
- Scroll dua pane tersinkron (`scrollTop`/`scrollLeft` ditiru, guard flag anti-loop).
- Pane after read-only; klik bubble di after meneruskan seleksi ke before (satu sumber state).

## Scope

- `kuron-studio/src/lib/EditorPanel.svelte` — textarea manual, tombol isi-manual, switcher mode, scroll-sync dua pane.
- `kuron-studio/src/lib/CanvasEditor.svelte` — prop `editable`/`readOnly` + teruskan klik seleksi dari pane after; overlay teks via `showTranslation` yang sudah ada.
- `kuron-studio/src-tauri/src/translation.rs` — perluas `preserve_user_edits` ke `original`/`reading`.
- Test vitest pola-sumber (manual edit persist; mode switch render dua pane; scroll-sync).

## Non-goals

- Tidak ada command backend baru; after = overlay Konva (after-lite), bukan PNG typeset — `render_page_png` tidak diekspos ke preview.
- Tidak ubah pipeline detect/ONNX, translate/batch, glossary, export.
- Tidak ada zoom/pan Konva baru; scroll tetap native `overflow-auto`.
- Tidak tambah provider/style; tidak ubah kontrak invoke L1/L2.

## Risks

- **Guard sync JSON me-reset saat ngetik** (`EditorPanel.svelte:82`): textarea manual + `onSaved` bisa balapan → mitigasi: pola §6 (save → prop ikut → guard diam), debounce save textarea.
- **Dua Stage Konva dobel memori base64 1600px**: gambar yang sama dipakai dua pane → mitigasi: satu data-URL dishare (referensi sama), bukan fetch dua kali.
- **After-lite bukan WYSIWYG export** (tanpa white-patch/font wrap backend): user bisa protes "di export beda" → mitigasi: catat `ponytail:` upgrade ke after-true (PNG typeset via command baru).
- **`save_translation` tanpa validasi**: translation kosong manual bisa berisi index duplikat → mitigasi: validasi ringan di frontend (index unik, ikut urutan bubble).

## Capabilities

### New Capabilities

- `manual-bubble-text`: isi/koreksi `original`+`reading`+`translated` manual per bubble terpilih tanpa AI, termasuk buat translation kosong dan proteksi dari overwrite AI.
- `before-after-split`: tampil before/after berdampingan di body editor dengan scroll tersinkron dan switcher Before|Split|After; after read-only, klik teruskan seleksi.

### Modified Capabilities

- `kuron-studio`: requirement "Canvas editor SHALL support bubble editing" bertambah (edit teks manual), dan requirement "Overlay ... `isUserEdited` guard" diperluas (guard mencakup `original`/`reading`, bukan cuma `translated`).

## Impact

- Frontend: `EditorPanel.svelte`, `CanvasEditor.svelte` (+ test `canvas-editor.test.ts` / baru).
- Backend: `translation.rs` (`preserve_user_edits`) saja; tanpa migrasi DB, tanpa command baru.
- Kontrak invoke tak berubah; export PNG/CBZ tak berubah (tetap sumber kebenaran hasil akhir).
