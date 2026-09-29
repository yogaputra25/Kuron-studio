## Why

Edit terjemahan sering "stuck": teks tak bisa ditambah, backspace seolah tak
jalan — terutama saat mengedit di tengah kalimat. Explore 2026-09-29 menemukan
tiga penyebab yang saling memperkuat di `EditorPanel.svelte`:

1. **Kursor loncat per huruf.** Textarea kartu pakai `value={b.translated}`
   satu arah + `oninput → editTr` yang mengganti **seluruh array**
   (`bubbles.map(...)`) tiap keystroke → Svelte render ulang → `value` DOM
   di-set ulang → kursor mental ke akhir teks. Ketik/backspace di tengah
   kalimat terasa "nggak masuk"; ketik di akhir kalimat kelihatan normal —
   makanya "sering", bukan "selalu".
2. **Respons save menimpa ketikan berjalan.** `saveTranslationEdits`
   (`debounce 500ms`) melakukan `translation = out` (replace seluruh objek
   dari server). Bila user mengetik lagi di jeda kirim→respons, huruf terbaru
   hilang ditimpa versi basi — teks *snap back*, terasa "tulisanku nggak masuk".
3. **Kartu `{#if selectedBubble}` dievaluasi ulang** tiap `translation`
   diganti, memperkuat efek (1).

Bukan penyebab (sudah dicek, aman): global `onKey` Delete/Backspace (return
dini untuk TEXTAREA), event Konva menembus textarea, `onblur → commit`,
CSS `user-select`.

Kait ke change `edit-in-bubble`: pola benarnya sudah dirumuskan di sana
(D2 — draft lokal, commit sekali) tapi baru diterapkan ke editor in-bubble.
Kartu sidebar masih pola lama. Change ini memperluas pola yang sama ke kartu.

Asumsi belum terverifikasi: laporan user tak menyebut stuck di kartu, bubble,
atau dua-duanya. Desain di bawah menutup keduanya (kartu = draft lokal;
in-bubble sudah lokal + perbaikan kecil `bind:value`); E2E §3 wajib
memverifikasi kedua lokasi.

## Temuan susulan (2026-09-29, ditempel sebagai §4)

Laporan user: (1) teks yang dihapus via backspace **muncul lagi** saat
edit-beruntun; tombol ↺ (2) **muncul tapi diklik tak mempan**. Keduanya satu
akar yang berbeda dari §1–§2: **last-response-wins**.

- `saveTranslationEdits` tanpa nomor generasi: `translation = out` untuk
  respons mana pun yang tiba — yang menang = respons tiba-terakhir, bukan
  tulisan terbaru. Dua timer berlapis (`draftTimer` 500ms + `saveTimer`
  500ms) membuat tiap jeda ketik ~1 dtk melahirkan satu save; backend agak
  lambat → dua save overlap → respons basi menimpa tulisan baru.
  Backspace `abc→ab`: respons save(`abc`) tiba belakangan → `c` bangkit lagi.
- `resetBubble` lewat debounce 500ms dan hanya membatalkan save yang *belum
  dikirim*; save yang *sudah terbang* pasti tiba dan pasti ditulis —
  hasil ↺ digilas sepersekian detik kemudian. Dari kursi user: AI sempat
  tampil sekilas (atau tidak), lalu teks manual lama kembali + badge
  `edited` menyala lagi.
- Backend tak bersalah di sini (`merge_ai_baseline` hanya sentuh `ai*`);
  draft §1–§2 mengurangi frekuensi tulis tapi tak menutup balapan ini.

Detail timing/urutan (tertunda ~1 dtk vs seketika; AI sempat tampil vs tidak
berubah) belum dikonfirmasi user — skenario E2E §3.1/§4 dirumuskan dari
analisis kode; konfirmasi saat E2E.

## What Changes

- 3 textarea kartu tunggal (`original`/`reading`/`translated`) → baca/tulis
  **state draft lokal** per field; `translation.bubbles` baru ditulis sekali
  saat commit (blur atau debounce) via `editTr` existing.
- Saat draft aktif, respons `save_translation` tidak boleh menghapus huruf
  yang diketik setelah snapshot dikirim (draft hidup di luar `translation`,
  jadi `translation = out` aman; commit draft berikutnya memicu save susulan).
- In-bubble: `value={editingValue}` → `bind:value` (kecil, sekalian; perilaku
  sama karena state sudah lokal).
- Jalur save (`editTr` → debounce 500ms → `save_translation`), baseline
  `ai*`, badge `edited`, ↺, glossary, autosave, cancel: dipakai ulang utuh.
- §4 menambah **penjaga urutan** di jalur yang sama (nomor generasi save +
  ↺ kirim-langsung), tanpa mengubah bentuk/ritme autosave dari sudut user.

## Scope

- `kuron-studio/src/lib/EditorPanel.svelte` — draft lokal 3 field kartu +
  commit blur/debounce + render kartu dari draft saat aktif.
- `kuron-studio/src/lib/CanvasEditor.svelte` — `bind:value` textarea
  in-bubble (satu baris).
- Test: vitest pola-sumber (draft lokal, commit→`editTr`, respons tak
  menimpa draft, `bind:value`, §4: generasi save + ↺ sinkron-menang) +
  Rust tidak perlu (tanpa perubahan backend).

## Non-goals

- Mengubah jalur save/backend (`save_translation`, baseline `ai*`, autosave
  500ms, cancel opsi B).
- Mengubah tampilan/posisi kartu maupun editor in-bubble.
- After-true / editor Before / hapus `Save edits`.

## Risks

- **Draft vs respons basi**: draft di luar `translation` membuat overwrite
  respons aman secara konstruksi; tapi commit draft yang datang *bersamaan*
  respons harus tetap antre save susulan — dijamin via `queueSave` di
  `editTr` (jalur existing). Mitigasi: E2E ketik-cepat §3.
- **Draft basi saat ganti bubble/halaman**: draft terikat `selectedRow`;
  pindah seleksi = commit/blokir draft lama (ikuti pola in-bubble:
  pindah = commit otomatis). Panel remount per `{#key page.id}` sehingga
  draft tak bocor antar halaman.
- **Tes pola-sumber tak membuktikan kursor/urusan nyata** — bukti interaksi =
  manual E2E §3 (§3.1 + §4 baru: backspace-cepat + ↺-saat-save-terbang).
