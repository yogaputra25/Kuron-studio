# Referensi v1 — tur fitur Kuron Studio (refresh 2026-11-07)

Screenshot 1372×800 dari app Tauri dev yang sedang jalan, project `Contoh`
(29 halaman, semua TERDETEKSI engine ort, 1 TANPA BUBBLE). Diambil via
`orca computer` (computer-use, foreground) sesuai urutan klik.
Nama file dipertahankan dari tur lama agar kompatibel; isi mengikuti UI baru.

| # | File | Isi |
|---|------|-----|
| 01 | 01-grid-awal-thumbs.png | Grid awal, tema gelap, ID |
| 02 | 02-grid-kocak-26.png | Grid 29 halaman TERDETEKSI (filter All) |
| 03 | 03-f1-providers.png | Modal Providers + form tambah (Gemini, API key kosong) |
| 04 | 04-f2-gagal-masih-providers.png | Modal Providers menutupi panel Glossary |
| 05 | 05-verifikasi-tutup-panel.png | Providers ditutup via Tutup → Glossary tersisa |
| 06 | 06-grid-bersih.png | Grid bersih setelah semua panel ditutup |
| 07 | 07-f2-glossary.png | Panel Glossary (5 entri, form Add + CSV) |
| 08 | 08-f3-review.png | Panel Results & Export: Contoh (0%, ekspor JSON/PNG/CBZ/PSD/ZIP) |
| 09 | 09-f4-diagnostics.png | Panel Diagnostics (macOS aarch64, invoke JSON Lines) |
| 10 | 10-f5-editor.png | Editor halaman 01 (6 bubble, engine ort) |
| 11 | 11-f5b-detect-1-halaman.png | Editor 01: hasil detect 6 bubble |
| 12 | 12-grid-01-terdeteksi.png | Grid: 01 TERDETEKSI |
| 13 | 13-f6-detect-semua-progres.png | Detect semua jalan + progres (7/29, 24%) |
| 14 | 14-grid-semua-terdeteksi.png | Grid 29/29 TERDETEKSI |
| 15 | 15-f7-tema-terang.png | Tema terang, bahasa ID |
| 16 | 16-f8-bahasa-en.png | Bahasa EN (tema terang) |
| 17 | 17-f9-dialog-import.png | Dialog native di `/tmp/clean-import/`, zip BOUKENSHA Ch8 terlihat |
| 18 | 18-grid-52-halaman-zip.png | Grid tetap 29 halaman (import dibatalkan; nama file lama dipertahankan) |
| 19 | 19-f10-cek-update.png | Toast gagal cek update (inline, jelas) |

Temuan baru vs tur 2026-10-28:
Panel ditutup via tombol Tutup, bukan Escape (Escape hanya menutup modal
Providers; panel Glossary/Results/Diagnostics tetap terbuka).
Detect semua kini ADA indikator progres (dulu sunyi).
Toolbar kini punya tombol langsung Providers/Batch/Glossary/Results & Export
+ Check for update. Dialog import difoto di folder bersih agar tidak ikut
memotret file sensitif di Desktop.
