# Tasks: edit-in-bubble

> P0 = Must, P1 = Should. Estimasi hari. Checkbox `- [ ]`.

## 1. Editor in-bubble di After (`CanvasEditor.svelte`) — P0 — 0.3d

- [x] 1.1 Props kabel edit: `onEditTranslation(i, value)?` (+ pakai
  `onSelectForward` existing untuk seleksi); default mati agar Before aman — P0
- [x] 1.2 Double-klik bubble (`dblclick` pada group, hanya saat
  `showTranslation`) → buka `<textarea>` absolute di stage container,
  posisi `(b.x*s, b.y*s)`, `width=b.w*s`, value awal = `tr.translated` — P0
- [x] 1.3 State ketikan LOKAL (tanpa tulis `translations` per huruf);
  commit → `onEditTranslation(i, value)`; Esc → tutup + buang;
  klik bubble lain → commit otomatis lalu pindah — P0
- [x] 1.4 WYSIWYG: `fontSize` = rumus `fs` overlay existing
  (`clamp(10..18, …)`); tema ikut `needsWhitePatch`; tinggi boleh tumbuh
  ke bawah (`max(b.h*s, …)`) — P0

**Done bila:** double-klik bubble di After → ketik → blur → teks overlay
berubah + bubble jadi `edited`, tanpa fokus amblas saat mengetik.

## 2. Sidebar 1 kartu + wiring (`EditorPanel.svelte`) — P0 — 0.2d

- [x] 2.1 `<ol>` N kartu → `{#if selectedBubble}` tunggal
  (`find(manualSel) ??` placeholder bila null); isi = 3 textarea + ↺ +
  badge + glossary existing (pindah rumah) — P0
- [x] 2.2 After pasang `onEditTranslation` → `editTr(i, "translated", v)`
  (jalur save existing; `original`/`reading` tetap via kartu) — P0
- [x] 2.3 Header `translated` + `Save edits` + hint overlay tetap — P0
- [x] 2.4 Test vitest pola-sumber: (a) dblclick→textarea + prefill,
  (b) commit→`editTr`/`onEditTranslation`, (c) Esc tanpa tulis,
  (d) 1 kartu + `Save edits` tetap; `pnpm test && pnpm check` hijau — P0
  (70/70, check 0 errors + 1 warning pre-existing BatchPanel `pageCount`)

**Done bila:** sidebar ramping (1 kartu ikut seleksi), edit di bubble
masuk jalur save yang sama (badge `edited`, ↺ tetap berfungsi).

## 3. Verifikasi + gates — P1 — 0.1d

- [ ] 3.1 Manual E2E di app (`pnpm tauri dev`): double-klik → ketik →
  blur/Ctrl+Enter → overlay update + `edited` → Esc di edit berikut →
  tak berubah → pindah bubble → commit otomatis → `Save edits` flush — P1
- [x] 3.2 Gates hijau (`cargo test`, `clippy -- -D warnings`,
  `pnpm test`, `pnpm check`) — P1
  (cargo 72 lib + integrasi ok, clippy 0 warnings, vitest 70/70,
  svelte-check 0 errors + 1 warning pre-existing BatchPanel `pageCount`)
- [ ] 3.3 Arsip via `/opsx:archive` — P1
