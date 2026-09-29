# Tasks: fix-edit-stuck

> P0 = Must, P1 = Should. Estimasi hari. Checkbox `- [ ]`.

## 1. Draft lokal kartu (`EditorPanel.svelte`) — P0 — 0.3d

- [x] 1.1 State `draft` (`{ row, original, reading, translated } | null`,
  key = `selectedRow`); `oninput` kartu tulis draft saja, textarea baca
  `draft?.field ?? b.field` — P0
- [x] 1.2 Commit blur per field + debounce ~500ms ketik-tanpa-blur →
  `editTr(row, field, v)` existing (badge `edited`, ↺, glossary utuh) — P0
- [x] 1.3 Pindah seleksi saat draft kotor → commit otomatis dulu
  (konsisten in-bubble); `manualSel === null` → draft dibuang — P0
- [x] 1.4 `Save edits` commit draft kotor dulu bila ada (flush manual tetap) — P0

**Done bila:** ketik + backspace di tengah ketiga field stabil (kursor tak
loncat), autosave/badge/↺opsa tetap berfungsi seperti sebelumnya.

## 2. In-bubble + test pola-sumber — P0 — 0.1d

- [x] 2.1 `CanvasEditor.svelte`: `value={editingValue}` →
  `bind:value={editingValue}` (satu baris) — P0
- [x] 2.2 Test vitest pola-sumber (`edit-stuck.test.ts`): (a) oninput kartu
  tulis draft, tak sentuh `translation.bubbles`; (b) blur/debounce →
  `editTr`; (c) pindah seleksi commit; (d) `bind:value` in-bubble;
  `pnpm test && pnpm check` hijau — P0 (2026-09-29: 81/81 vitest,
  svelte-check 0 errors + 1 warning pre-existing BatchPanel)

**Done bila:** pola draft-terpisah terbukti statis untuk kartu + in-bubble.

## 4. Last-write-wins: generasi save + ↺ sinkron (§4, temuan susulan) — P0 — 0.1d

- [x] 4.1 `saveTranslationEdits`: `saveGen++` per kirim; respons basi
  (`my !== saveGen`) diabaikan sebelum `translation = out` + `onSaved` — P0
- [x] 4.2 `resetBubble`: tulis `ai*` + flag false lalu `saveTranslationEdits()`
  langsung (bukan debounce) — respons save lama otomatis basi via §4.1 — P0
- [x] 4.3 Test vitest pola-sumber di `edit-stuck.test.ts`: (a) guard generasi
  sebelum tulis state; (b) ↺ panggil save langsung; (c) error path tak
  menyentuh state — P0 (2026-09-29: `reset-translation.test.ts` ikut
  diupdate ke pola save-langsung, semua hijau)

**Done bila:** backspace-cepat menabrak save tak membangkitkan huruf;
klik ↺ saat save terbang → AI bertahan + badge mati.

## 3. Verifikasi + gates — P1 — 0.1d

- [ ] 3.1 Manual E2E di app (`pnpm tauri dev`): ketik + backspace di TENGAH
  kalimat (kartu DAN bubble After) → kursor stabil; ketik-cepat menabrak
  save (500ms) → tak ada huruf hilang; klik ↺ pasca-ketik → AI kembali;
  `Save edits` flush; (§4) backspace-cepat 3× berturut → tak ada huruf
  bangkit ~1 dtk kemudian; klik ↺ saat spinner save → AI sempat tampil
  dan BERTAHAN + badge mati — P1
- [x] 3.2 Gates hijau (`cargo test`, `clippy -- -D warnings`,
  `pnpm test`, `pnpm check`) — P1 (tanpa perubahan backend; Rust hanya
  pengaman regresi)
  (2026-09-29: cargo 72 lib + 5 integrasi ok, clippy 0 warnings,
  vitest 81/81, svelte-check 0 errors + 1 warning pre-existing BatchPanel)
- [ ] 3.3 Arsip via `/opsx:archive` — P1
