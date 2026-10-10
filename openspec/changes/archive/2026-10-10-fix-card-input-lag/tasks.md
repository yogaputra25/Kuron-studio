## 1. Probe ukur dulu (jangan tebak) — P0 — 0.1d

- [x] 1.1 Analisis statis jalur ketik (deviasi: tanpa app running): draftInput tulis $state per huruf + respons save ganti objek utuh → dominan B+C; IME tanpa guard terkonfirmasi — P0
- [x] 1.2 Hipotesis dominan B render-per-huruf + C respons-menabrak (diatasi fix sama: nol write reactive per huruf); tercatat di design — P0

**Done bila:** angka 1.1 menunjuk satu hipotesis dominan, tercatat di design sebagai temuan ukur.

## 2. Uncontrolled textarea + cache non-reactive — P0 — 0.3d

- [x] 2.1 `EditorPanel.svelte`: `draftInput` tulis cache plain (bukan `$state`); `value=` baca snapshot yang hanya berubah saat ganti/commit/sinkronisasi — P0
- [x] 2.2 `commitDraft` baca cache + `editTr` per field berubah (guard existing); `selectBubble`/`saveTranslationEdits`/`onTranslated` cek cache (ganti `if (draft)`) — P0
- [x] 2.3 Guard IME: `composing` flag via `oncompositionstart/end`; timer tak commit tengah komposisi; commit jalan pasca-`compositionend` — P0
- [x] 2.4 Matriks sinkronisasi D3: ganti bubble / hasil baru / respons save / ↺ tak tulis DOM fokus; snapshot dibersihkan searah tiap commit — P0
- [x] 2.5 Test pola-sumber: (a) `oninput` tak tulis `$state` per huruf, (b) commit via `editTr` existing, (c) guard `isComposing`, (d) kontrak `editTr`/saveGen/badge/↺ tak berubah — P0

**Done bila:** skenario spec `responsive-card-input` hijau pola-sumber + kontrak `edit-stuck` diperbarui.

## 3. Verifikasi + gates — P1 — 0.1d

- [x] 3.1 Manual E2E terukur (`pnpm tauri dev`): ketik-cepat 1 kalimat → lengkap-berurutan; ketik-terus lewati save → kursor stabil; IME → utuh; Translate-baru/↺/pindah saat draft kotor → tak hilang — P1 (E2E user: hijau, via typing-comfort)
- [x] 3.2 Gates hijau (`pnpm test`, `pnpm check`; Rust tak tersentuh) — P1
- [x] 3.3 Arsip via `/opsx:archive` — P1
