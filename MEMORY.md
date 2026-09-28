# MEMORY.md — Kuron Studio

Catatan lintas sesi. Yang sudah ada di kode atau di `AGENTS.md` tidak diulang di
sini; dokumen ini khusus hal yang **tidak bisa dibaca dari repo**.

---

## Identitas

| Kunci | Nilai |
|---|---|
| Nama | Kuron Studio |
| Tagline | Manga Translation Workbench |
| Bundle ID | `id.kuron.studio` |
| Stack | Tauri v2 (Rust) + Vite + Svelte 5 + Tailwind 4 + Konva |
| Platform | Desktop: Windows, macOS, Linux. Mobile ditunda. |
| Induk | Kuron App (Flutter, repo `nhasixapp`) |
| Bahasa kerja | Komentar dan commit Bahasa Indonesia, istilah teknis English |
| Bahasa default UI | Indonesia, dengan terjemahan English dan Mandarin |

---

## Keputusan produk

Semua ketetapan ini sudah final. Kalau ada yang battled, jawab dengan
pertanyaan, jangan dengan implementasi.

| Tanggal | Keputusan | Alasan |
|---|---|---|
| 2026-09-08 | Nama `Kuron Studio` | Manfaatkan brand Kuron, mudah rebrand ke Kotoba atau PanelForge |
| 2026-09-08 | Scope publik | Untuk provider dan siapa saja yang mau memakai |
| 2026-09-08 | Svelte, bukan React | Preferensi eksplisit pemilik proyek. Bukan karena React buruk secara teknis. |
| 2026-09-08 | Rust sebagai sumber kebenaran | Reuse `kuron_native/rust`, performa batch, akses file native |
| 2026-09-08 | BYOK plus OS keychain | Tidak menyimpan kunci dalam bentuk plaintext |
| 2026-09-08 | Reuse prompt Kuron 1:1 | Konsistensi hasil terjemahan, tidak membuat ulang |
| 2026-09-08 | 3 gambar per permintaan translate | Menghemat token, semaphore 3, antrean tetap bisa 50 halaman |
| 2026-09-08 | Glossary di SQLite lokal | `glossary.db`, bukan SharedPreferences JSON |
| 2026-09-08 | Ekspor: JSON dulu, PNG dan CBZ menyusul, PSD ditunda | JSON wajib untuk integrasi Kuron App |
| 2026-09-08 | ONNX runtime `ort` | Di bawah 2 detik per halaman, bundel sekitar 12MB sudah cukup |
| 2026-09-08 | Prompt dan model boleh dibagikan keluar | Lisensi tidak masalah untuk provider eksternal |

---

## Peta reuse dari Kuron App

 Dipetakan ulang: logika di Flutter yang harus punya pasangan Rust. Kalau salah
satu berubah di sisi Flutter, sisi Rust ikut berubah.

| Kuron App (Flutter/Dart) | Kuron Studio (Rust) | Yang harus sama |
|---|---|---|
| `MosaicBuilder.buildMosaic` | `image_ops::build_mosaic` | Crop padding 20%, skala 2x, label merah, JPEG 85/75, plafon 2MB/1MB |
| `FallbackImageHandler.compressPage` | `image_ops::compress_page` | Sisi terpanjang 1280px, JPEG 85 |
| `image_ops_chunk_webtoon` | `image_ops::chunk_webtoon` | Pemotongan strip panjang |
| `BubbleBox` | `detector.rs` | x, y, w, h, confidence, shape, kind, tail |
| `openai_compatible_provider.dart` | `provider/openai.rs` | Prompt mosaic dan full, aturan SFX, append glossary |
| `gemini_translation_provider.dart` | `provider/gemini.rs` | REST Google |
| `cohere_translation_provider.dart` | `provider/cohere.rs` | Endpoint `/v2/chat` |
| `AiProviderType` dan `AiProviderConfig` | `provider/config.rs` | 9 tipe, base URL bawaan, URL model |
| `TranslationStyle` | `prompt.rs` | 7 gaya: natural, genz, action, romantis, formal, kasar, literal |
| `PageTranslation` dan `BubbleTranslation` | `translation.rs` | rect, original, reading, translated, shape |
| `GlossaryEntry` | `glossary.rs` | sourceText, translatedText, reading |
| `TranslationCacheRepository` | `cache.rs` | Kunci = hash(gambar + bubble + bahasa + gaya) |

Format glossary dan cache sengaja kompatibel dengan JSON Kuron App supaya
bisa dimigrasi.

---

## Jebakan yang sudah pernah menguras waktu

Hal-hal yang memakan biaya sekali saja. Tetap berlaku kecuali ada test yang
menjaganya.

- **`pnpm dev` menyesatkan.** Hanya antarmuka, tanpa backend Rust. Semua
  tombol yang memanggil `invoke` akan gagal. Selalu `pnpm tauri dev`.
- **`keyring` macOS gagal tanpa gelembung.** Tanpa feature `apple-native`,
  crate memakai credential store `mock`: `set_password` mengembalikan Ok tanpa
  error, lalu API key hilang tiap restart aplikasi. Gejalanya tidak menunjuk
  ke penyebabnya. Feature itu dikunci di `Cargo.toml`, test
  `keychain_backend_is_real` menjaganya.
- **Bundel dev yang basi.** Gejalanya: perubahan `CanvasEditor.svelte` atau
  `bubble.ts` tidak muncul padahal sudah disimpan. `pnpm dev:fresh` menghapus
  cache Vite. `--nuke` juga mereset profil WebView2, tapi data proyek tetap
  aman.
- **Pratinjau gambar besar bisa menggantung.** Gambar penuh di-decode di
  tempat. Pratinjau memakai `get_image_preview` dengan batas sisi, dan
  thumbnail 512px untuk grid.
- **Kontrak Rust dan TypeScript terpisah.** Nama command dan nama argumen
  hidup di dua file berbeda. Test `invoke-contract.test.ts` dan
  `image-ext-contract.test.ts` menjaga agar tidak menyimpang. Mengganti nama
  di satu sisi saja sudah cukup untuk membuat semuanya gagal.

---

## Status per 2026-09-29

- M0 sampai M4 selesai.
- M5 sebagian. Yang masih terbuka ada di
  `openspec/changes/kuron-studio-mvp/tasks.md` (M5-5: plugin prompt per genre,
  dan batch size yang bisa dipilih).
- Empat perubahan aktif masih menunggu verifikasi manual, semuanya terkait
  alur import dan editor kanvas:
  `fix-import-pages-project-id`, `fix-preview-hang`,
  `fix-editor-blank-canvas`, `manual-text-before-after`. Daftar tasknya ada di
  masing-masing `tasks.md`.

---

## Cara kerja yang terbukti

- **Gate dulu, baru bilang selesai.** Gate ada di `AGENTS.md` bagian 2. Klaim
  selesai tanpa menjalankan gate tidak sah.
- **Kontrak lebih penting dari implementasi.** Test yang menjaga nama command,
  kunci i18n, dan token warna mencegah lebih banyak regresi daripada test
  logika.
- **Pemangkasan sadar ditulis.** Komentar `ponytail:` menandai tempat yang
  sengaja dibiarkan sederhana beserta plafonnya, supaya tidak diam-diam
  menyesatkan nanti.
- **Spesifikasi mendahului kode.** Perubahan besar mulai dari `/opsx:propose`.
