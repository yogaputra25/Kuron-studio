## Why

Editor stuck "Memuat gambar…" selamanya: stage + hint baru (change
`fix-editor-blank-canvas`) sudah jalan — stale bundle gugur — tapi
`img.onload`/`onerror` tidak pernah fire. Artinya `img.src` diset ke sesuatu
yang browser gantung: request tak pernah selesai, bukan URL salah (itu langsung
`onerror`).

Rantai yang dicurigai (bukti kode, bukan tebakan):

- `App.openEditor` (`App.svelte:104-115`) `await api.getImagePreview(pg.path, 1600)`
  — command Rust `get_image_preview` (`commands/image.rs:7`) jalan SYNCHRONOUS
  di handler: `fs::read` + `image::load` + `resize` + JPEG encode, semua di
  thread async Tauri tanpa `spawn_blocking`. Bandingkan: `detect.rs:99`,
  `batch.rs:243`, `translate.rs:256` semuanya bungkus kerja berat dengan
  `tauri::async_runtime::spawn_blocking`. Untuk file manga besar (4–10MB PNG),
  full decode + resize 1600 + encode JPEG bisa makan detik–puluhan detik sambil
  MEMBLOKIR runtime — `await` di frontend tak pernah resolve, `fullImageUrl`
  tetap `""` atau thumb lama, `img.src = ""` tak fire apa-apa → hint selamanya.
- Lapisan kedua: kalaupun resolve, JPEG 1600px → data-URL base64 2–4 juta
  karakter diset ke `new Image().src` di WebView2 — lambat/gantung tanpa
  timeout, tanpa retry, tanpa fallback ke thumbnail 512 yang terbukti tampil
  di grid.

Ini P0: editor = layar kerja utama; user tidak bisa bedakan "loading" vs
"macet" karena tidak ada timeout, tidak ada progres, tidak ada tombol coba lagi.

Kait ke `docs/ide.md` (editor canvas core loop) dan `docs/target.md`
(M0: import → grid → editor tampil). Lanjutan `fix-editor-blank-canvas`
(§1–§3 selesai, §4.1 hijau) — change ini menyerang lapisan transport gambar,
bukan rendering.

## Scope

- `kuron-studio/src-tauri/src/commands/image.rs` — bungkus
  `get_image_preview` dengan `spawn_blocking` (ikuti pola `detect.rs:99`);
  jadikan `async fn`. Clamp `max_side` tetap (64–2048).
- `kuron-studio/src/lib/CanvasEditor.svelte` — load background defensif:
  timeout (mis. 15s) → `imgError` + tombol "Coba lagi"; fallback otomatis ke
  thumbnail bila URL full gagal/timeout; retry manual tanpa remount.
- `kuron-studio/src/App.svelte` — `openEditor`: tampilkan editor LANGSUNG
  dengan thumbnail dulu (jangan `await` full 1600 sebelum mount); upgrade ke
  full saat resolve (progressive). Timeout + error diteruskan via `openError`
  yang sudah ada (lane-2 change sebelumnya).
- Test: Rust unit test tetap (missing file errors); tambah test vitest pola
  timeout/fallback bila murah; catat manual checklist (file besar → gambar
  muncul <Xs atau pesan + retry).

## Non-goals

- Tidak ubah pipeline detect/ONNX (sehat — 4 bubble terbukti).
- Tidak ubah kontrak invoke L1/L2 (`get_image_preview {path, max_side}` tetap;
  `async` tidak mengubah wire key).
- Tidak tambah command baru, tidak ubah struktur `Page`/`Project`.
- Tidak refactor `redraw()` overlay; tidak sentuh translate/batch/glossary.

## Risks

- **`async` + `spawn_blocking` mengubah perilaku error**: `String` error tetap
  sama (`read failed`/`decode failed`), tapi timing berubah (sekarang truly
  async). Mitigasi: existing test `missing_file_errors` harus jadi async;
  guard invoke-contract tidak terpengaruh (key sama).
- **Timeout terlalu agresif** memutus load file besar yang sebenarnya sukses
  1 detik kemudian. Mitigasi: timeout longgar (15s) + retry manual, bukan
  auto-abort agresif; fallback thumb tetap tampil sehingga editor usable.
- **Data-URL raksasa tetap berat** walau backend cepat. Mitigasi P1 lanjutan
  (di luar change ini): serve via `asset:` protocol / temp file, bukan base64.
  Dicatat sebagai `ponytail:` di kode bila diambil.
