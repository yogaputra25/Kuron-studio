## Why

User mengedit isi bubble secara manual (original/reading/translated) — lewat
textarea sidebar EditorPanel — lalu mau mengembalikan SATU bubble ke hasil AI
terakhir tanpa panggil AI lagi (tanpa biaya/token baru, tanpa ubah bubble lain).
Hari ini tidak bisa: `save_translation` (`commands/translate.rs:403`) menimpa
`page_t.bubbles = bubbles` apa adanya, sehingga teks manual menghancurkan teks
AI secara permanen. Badge `edited` (`isUserEdited`) pun tidak bisa dikembalikan
ke normal karena teks AI-nya sudah hilang.

Tiga sumber "AI terakhir" yang sudah ada — semuanya terbukti BUKAN AI murni:

- `projects.json` — menyimpan bubbles hasil merge; manual sudah menimpa AI.
- translate cache (sqlite `translation_cache`) — payload = hasil `map_text`,
  yang SUDAH lewat `preserve_user_edits` sejak translate ke-2; hanya kebetulan
  murni bila cache-hit translate pertama. Rapuh, bukan kontrak.
- `onTranslated` di frontend — `t` dari backend juga sudah di-merge.

Kait ke `docs/ide.md` (editor canvas core loop) dan change
`manual-text-before-after` (D1–D3: jalur manual + preserve 3 field;
scope reset sengaja dikecualikan — change itu non-goal "tanpa command baru,
kontrak utuh").

## What Changes

- Tiap `BubbleTranslation` menyimpan salinan AI murni (`aiOriginal`,
  `aiReading`, `aiTranslated`, `#[serde(default)]` agar `projects.json` lama
  tetap ke-load): diisi jalur AI, disentuh manual TIDAK.
- `save_translation` tidak lagi menimpa buta: `ai*` diambil dari stored,
  hanya current + flag yang ikut kiriman frontend.
- Tombol ↺ per row bubble di sidebar: salin `ai*`→current,
  `isUserEdited=false`, simpan via `saveTranslation` existing.
- `retry_bubble` mengisi `ai*` target (hasil AI baru = baseline baru);
  `translate_page`/batch mengisi `ai*` semua bubble.
- Tombol Translate… menjadi Batal saat sibuk (opsi B): klik → flag lokal
  `cancelled` + command baru `cancel_translate { page_id }`; response telat
  diabaikan frontend, backend berhenti di titik cek berikut tanpa persist
  tanpa fail, status halaman dikembalikan ke sebelum translate.

## Scope

- `kuron-studio/src-tauri/src/translation.rs` — 3 field `ai*` di
  `BubbleTranslation`, isi di `map_mosaic`/`map_full_image`, ikut di
  `preserve_user_edits_except_vec` untuk bubble edited.
- `kuron-studio/src-tauri/src/commands/translate.rs` — `save_translation`:
  merge `ai*` dari stored, bukan dari kiriman frontend.
- `kuron-studio/src-tauri/src/commands/batch.rs` — `retry_bubble`: isi
  `ai*` bubble target dari hasil AI baru.
- `kuron-studio/src/lib/types.ts` — 3 field `ai*` di `BubbleTranslation`.
- `kuron-studio/src/lib/EditorPanel.svelte` — tombol ↺ per row
  (fungsi `resetBubble`), hanya tampil bila `isUserEdited`.
- `kuron-studio/src-tauri/src/lib.rs` + `commands/translate.rs` — set
  cancel di `AppState`, stash prev-status di `claim_page`, command baru
  `cancel_translate`, `backoff_translate` cek flag + `select!` saat sleep.
- `kuron-studio/src/lib/TranslatePanel.svelte` — tombol Batal + guard
  `cancelled` untuk resolve/reject telat; `api.ts`: `cancelTranslate`.
- Test: Rust unit (reset kembalikan AI; manual tak rusak `ai*`;
  `projects.json` lama tanpa `ai*` tetap ke-load) + vitest pola-sumber.

## Non-goals

- Reset per-halaman sekaligus (tombol global) — YAGNI, one-liner per bubble dulu.
- Re-translate saat reset (itu opsi B = `retry_bubble`, sudah ada).
- Clear ke string kosong (opsi C) — user memilih opsi A.
- Reset `original`/`reading` hasil OCR/detect manual sebelum AI pertama
  (belum ada AI = belum ada baseline; tombol tak tampil bila `ai*` kosong).
- Migrasi DB sqlite — `ai*` hidup di `projects.json` (serde default),
  cache payload ikut format baru secara alami (miss → tulis ulang).
- Cancel batch sekaligus (opsi C) — tetap non-goal. Efek samping sadar:
  worker batch untuk page_id yang dibatalkan ikut berhenti cepat (cek flag
  di backoff bersama), tapi antrean/progress batch jalan terus.

## Risks

- **`save_translation` merge `ai*` dari stored berarti frontend TAK BISA
  mengeset `ai*`** — by design (anti-tamper), tapi kontrak implisit ini
  harus dikomentari di kode agar tidak "diperbaiki" jadi pass-through.
- **Bubble lama tanpa `ai*`** (dibuat sebelum fitur ini): serde default → `""`;
  reset pada bubble itu = jadi kosong + `isUserEdited=false`. Mitigasi:
  tombol ↺ hanya tampil bila `ai*` non-kosong, atau banner info sekali.
  Keputusan final saat implementasi.
- **Cache payload lama** (ditulis sebelum `ai*` ada) terbaca tanpa `ai*` →
  `apply_cached` persist tanpa baseline. Mitigasi: payload ikut format baru
  secara alami (cache miss → `store_result` tulis ulang); jendela basi
  kecil dan self-healing (translate ulang mengisi `ai*`).
- **Response telat pasca-batal**: promise `translatePage` tetap resolve/reject
  → guard `cancelled` di `translate()` + `catch` wajib, kalau tidak banner
  error "dibatalkan" muncul padahal user sudah lanjut kerja.
- **Restore status**: `claim_page` menimpa status jadi Translating; cancel
  harus kembalikan ke status SEBELUM klaim (bisa detected/translated/failed),
  bukan default. Mitigasi: stash prev-status di claim, restore di path
  cancel; jangan lewat `fail_page`.
- **POST yang sedang jalan tak bisa di-abort** (tanpa abort handle reqwest):
  hasilnya dibuang via cek ulang pasca-backoff sebelum map/store — token
  satu call itu tetap kepakai. Sadar, bukan refund.
