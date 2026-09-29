# Tasks: reset-translation

> P0 = Must, P1 = Should. Estimasi hari. Checkbox `- [ ]`.

## 1. Baseline `ai*` backend (Rust) — P0 — 0.25d

- [x] 1.1 `translation.rs`: tambah `aiOriginal/aiReading/aiTranslated: String`
  (`#[serde(default)]`) di `BubbleTranslation`; isi = hasil AI di
  `map_mosaic` + `map_full_image` (`isUserEdited=false`) — P0
- [x] 1.2 `translation.rs` `preserve_user_edits_except_vec`: bubble edited
  bawa `ai*` lama dari old; unedited ikut AI baru (D2); `retry` target
  (`except_index`) selalu AI baru + flag false — P0
- [x] 1.3 `commands/translate.rs` `save_translation`: merge `ai*` dari stored
  by index, abaikan kiriman frontend + komentar anti-tamper — P0
- [x] 1.4 `commands/batch.rs` `retry_bubble`: isi `ai*` target dari hasil AI
  baru (= baseline baru, D2-retry) — P0
- [x] 1.5 Test Rust: (a) AI segar → `ai*` = current; (b) manual+save →
  current manual + `ai*` utuh; (c) preserve edited → `ai*` lama;
  (d) JSON lama tanpa `ai*` ke-parse default `""`;
  `cargo test && cargo clippy --all-targets -- -D warnings` hijau — P0

**Done bila:** edit manual → save → reopen: current manual, `ai*` = AI lama.

## 2. Baseline + tombol ↺ frontend — P0 — 0.25d

- [x] 2.1 `types.ts`: tambah 3 field `ai*` di `BubbleTranslation`; `mk()`
  di test ikut (default `""`) — P0
- [x] 2.2 `translation.ts` `mergeUserEdits`: bubble edited ikut restore
  `ai*` dari old (satu baris dalam spread existing) + test — P0
- [x] 2.3 `EditorPanel.svelte`: `resetBubble(i)` (current = `ai*`,
  `isUserEdited=false` → `queueSaveTranslationEdits`); tombol ↺ per row,
  tampil hanya bila `isUserEdited && aiTranslated` non-kosong (D4) — P0
- [x] 2.4 Test vitest pola-sumber: (a) ↺ restore `ai*` + flag false,
  (b) tanpa baseline tombol tak render, (c) reset lewat debounce bukan
  save langsung; `pnpm test && pnpm check` hijau — P0

**Done bila:** klik ↺ di bubble edited → teks = AI terakhir, badge hilang,
tanpa panggilan AI (network idle).

## 3. Verifikasi + gates — P1 — 0.1d

- [ ] 3.1 Manual: translate 1 halaman → edit 1 bubble → ↺ → teks AI kembali,
  badge hilang → re-translate → bubble itu update normal — P1
- [x] 3.2 Gates penuh hijau ulang pasca-cancel (`cargo test`, `clippy`, `pnpm test`, `pnpm check`) — P1
  (cargo 72 lib + integrasi ok, clippy -D warnings 0, vitest 59/59,
  svelte-check 0 errors + 1 warning pre-existing BatchPanel `pageCount`)
- [ ] 3.3 Arsip via `/opsx:archive` — P1

## 4. Cancel translate per-halaman (opsi B) — P1 — 0.5d

- [x] 4.1 `lib.rs` + `translate.rs`: `AppState.cancel: Mutex<HashSet<String>>`
  + stash prev-status; `claim_page` stash; command baru `cancel_translate
  { page_id }` + registrasi; `api.ts`: `cancelTranslate` — P1
- [x] 4.2 `backoff_translate(state, page_id, …)`: cek flag tiap iterasi +
  `select!` saat sleep + cek ulang pasca-backoff; caller (`translate_prep`,
  `run_work`, `retry_bubble`) skip `fail_page`/persist di path cancel +
  restore status — P1
- [x] 4.3 `TranslatePanel.svelte`: tombol Batal saat `busy` (flag `cancelled`,
  `busy=false`, info); guard resolve/`catch` telat — P1
- [x] 4.4 Test: Rust (claim→cancel→status pulih; backoff batal saat flag) +
  vitest pola-sumber (tombol Batal, guard telat); gates §3.2 hijau — P1

**Done bila:** klik Translate → Batal di tengah backoff → UI lepas seketika,
tanpa persist tanpa banner error, status halaman kembali seperti sebelum translate.
