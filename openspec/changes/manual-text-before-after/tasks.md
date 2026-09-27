# Tasks: manual-text-before-after

> P0 = Must, P1 = Should, P2 = Nice. Estimasi hari. Checkbox `- [ ]`.

## 1. Manual text entry (frontend) — P0 — 0.5d

- [ ] 1.1 `EditorPanel.svelte`: ubah `<p>O:>` (dan reading) jadi `<textarea>` per bubble terpilih; `editTr` dukung `field: original|reading|translated`, set `isUserEdited=true` ketiganya; simpan via `saveTranslationEdits` existing — P0
- [ ] 1.2 Debounce save textarea ~500ms (pola §6 fix-preview-hang: save → prop ikut → guard JSON diam); error save tampil di banner existing, teks lokal tetap — P0
- [ ] 1.3 Tombol "Isi manual" bila `PageTranslation` null: bangun `PageTranslation` kosong lokal (bubbles dari `BubbleBox`, string `""`) → `api.saveTranslation`; validasi frontend index unik ikut urutan bubble — P0
- [ ] 1.4 Test vitest pola-sumber: (a) ketik cepat → satu save, (b) isi-manual null → translation kosong tersimpan + survive, (c) guard tidak reset pasca-save; `pnpm test && pnpm check` hijau — P0

**Done bila:** klik bubble → ketik original/reading/translated tanpa AI → tutup → buka lagi teks tetap; skenario spec `manual-bubble-text` hijau.

## 2. Preserve manual edit (backend) — P0 — 0.25d

- [ ] 2.1 `translation.rs:157` `preserve_user_edits`: lindungi `original`/`reading` juga bila `isUserEdited` (clone 3 field, bukan cuma `translated`) — P0
- [ ] 2.2 `cargo test preserve` (atau substring setara) + `cargo clippy --all-targets -- -D warnings` hijau — P0

**Done bila:** edit manual original → translate ulang → field manual utuh, bubble lain tetap update; tanpa command baru, kontrak L1/L2 utuh.

## 3. Split before/after (frontend) — P0 — 0.5d

- [ ] 3.1 `EditorPanel.svelte`: state `mode: 'before'|'split'|'after'` + switcher `Before|Split|After`; mode tunggal render satu pane (before: editable, `showTranslation=false`; after: read-only, `showTranslation=true`) — P0
- [ ] 3.2 `CanvasEditor.svelte`: prop `editable: boolean` + `onSelectForward?: (i:number)=>void`; pane after `editable=false`, klik bubble teruskan seleksi ke before (bubbles tak berubah) — P0
- [ ] 3.3 Split = dua pane `flex-1` masing-masing scroll container sendiri, **satu data-URL dishare** (referensi sama, `get_image_preview` sekali) — P0
- [ ] 3.4 Scroll-sync: `onscroll` A → guard flag → `B.scrollTop/scrollLeft = A` dan sebaliknya (~10 baris, tanpa transform Konva); overlay survive scroll — P0
- [ ] 3.5 Test vitest pola-sumber: (a) tiap mode render pane benar, (b) scroll kiri 300px → kanan ikut tanpa loop, (c) klik after #3 → before selected 3 + bubbles unchanged; `pnpm test && pnpm check` hijau — P0

**Done bila:** mode Split tampil dua pane berdampingan, scroll bareng, after read-only; skenario spec `before-after-split` hijau.

## 4. Verifikasi + gates — P1 — 0.1d

- [ ] 4.1 Manual: halaman ber-bubble → Isi manual 1 bubble tanpa AI → Split → before kotak saja, after teks tampil → scroll kiri-kanan ikut → translate ulang → manual utuh — P1
- [ ] 4.2 Gates penuh hijau (`cargo test`, `clippy`, `pnpm test`, `pnpm check`) — P1
- [ ] 4.3 Arsip via `/opsx:archive` — P1
