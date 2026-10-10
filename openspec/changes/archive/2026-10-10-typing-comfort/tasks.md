## 1. Commit-on-boundary + timer-jaring — P0 — 0.3d

- [x] 1.1 `EditorPanel.svelte`: hapus `draftTimer` 500ms dari `draftInput` (tulis cache saja); `compEnd` commit langsung (bukan timer); konstanta `DRAFT_SAFETY_MS = 10_000` + timer-jaring per baris kotor — P0
- [x] 1.2 Save flush eksplisit: handler Save `commitDraft()` + `saveTranslationEdits()` langsung; indikator dirty (`dirtyDraft` boolean, 1 render per mulai-ketik) ikut konvensi `Simpan*` — P0
- [x] 1.3 Flush-on-destroy: `onDestroy`/tutup-panel commit bila cache kotor sebelum clear timer (best effort, tanpa await) — P0
- [x] 1.4 Test pola-sumber: (a) tanpa timer-500ms di `draftInput`, (b) `compEnd` commit langsung, (c) konstanta jaring + fire sekali, (d) indikator dirty, (e) kontrak `editTr`/saveGen/badge/↺/IME tak berubah — P0
- [x] 1.5 `EditorPanel.svelte` sync-guard: cabang `translation` di `syncFromPage` skip bila `draftBox.cur` kotor ATAU save terbang (`saveTimer`/`savingTr`); cabang `bubbles` tak diubah — P0
- [x] 1.6 Test pola-sumber sync-guard: (a) skip saat kotor, (b) skip saat terbang, (c) sync normal saat tenang (mount/echo/batch), (d) cabang `bubbles` tak tersentuh — P0

## 2. Verifikasi + gates — P1 — 0.1d

- [x] 2.1 Manual E2E terukur (`pnpm tauri dev`): ketik 1 kalimat penuh tanpa tunggu → lengkap + kursor stabil; diam >10 dtk → commit+save fire sekali; Save/manual-dirty → indikator padam; tutup-tanpa-blur → tak hilang; IME + kalimat->10dtk → ekspektasi jujur — P1 (E2E user: hijau)
- [x] 2.2 Gates hijau (`pnpm test`, `pnpm check`; Rust tak tersentuh) — P1
- [x] 2.3 Arsip via `/opsx:archive` — P1
