# Changelog

Semua perubahan penting Kuron Studio dicatat di sini, dari sudut pandang pengguna.
Format mengikuti [Keep a Changelog](https://keepachangelog.com/id/1.1.0/).
Belum ada penomoran rilis/tag — semua di bawah adalah `[Unreleased]` sampai versi pertama ditandai.

## [Unreleased]

### Ditambahkan

- Dialog **Batch Translate dari halaman list**: pilih provider + model (daftar otomatis, default model tersimpan), bahasa, gaya, SFX, kualitas mosaic, cakupan (terpilih / folder / terlihat), lewati halaman yang sudah diterjemahkan (default aktif), dan estimasi jumlah request. Pilihan provider + opsi diingat antar sesi.
- Guard cakupan kosong di dialog batch: tombol Mulai disabled + hint bila tidak ada halaman masuk cakupan, dan scope default mengikuti ada-tidaknya seleksi.
- **Timeout per halaman batch** (default 15 menit, bisa diatur 1–1440 menit di Pengaturan): halaman yang macet dilewati sebagai gagal berpesan jelas, antrean lanjut, dan bisa dicoba lagi via Retry Failed.
- Progres fase persiapan batch ("Menyiapkan N/M") yang bergerak real-time per halaman selesai, bukan diam sampai akhir.
- **Deteksi bubble paralel**: pool session ONNX (menyesuaikan jumlah CPU) + timeout per halaman, dengan urutan hasil tetap.
- **Tandai selesai manual** per halaman: halaman yang memang sudah benar walau ada bubble kosong (mis. kepotong) bisa ditandai selesai tanpa mengubah data.
- Definisi "selesai" baru: bubble yang teks sumbernya (`original`) kosong tidak dihitung — halaman seperti itu langsung dinilai selesai.
- Export: penomoran file selalu sekuensial (`001..N`) mengikuti urutan cerita, termasuk saat ada halaman dilewati; daftar halaman yang dilewati karena belum diterjemahkan kini dilaporkan di pesan hasil.
- Panel Hasil & Ekspor: rename, filter default "semua", dan ekspor dipromosi sebagai tujuan utama panel.
- Cakupan ekspor: per folder atau halaman terpilih, bukan cuma full project.

### Diperbaiki

- Batch macet total tanpa pergerakan (0/N diam): penyebabnya fase persiapan tanpa timeout/progres + worker POST yang menggantung. Kini tiap fase berdeadline, berprogres, dan halaman macet dilewati.
- Batch berhenti di tengah (mis. 9%) tanpa pesan: tiap kegagalan sekarang tercatat + berpesan jelas di daftar gagal.
- Error tulis proyek yang polos ("Access is denied") tanpa lokasi kini menyebut file + operasinya; lock sesaat (antivirus/indexer) dicoba ulang otomatis; sync file di Windows diperbaiki (sebelumnya selalu gagal diam-diam).
- Kartu gelap ilustrasi onboarding kini ikut tema terang/gelap (token `--ks-*`), bukan warna hardcoded.

### Diubah

- Seleksi halaman eksplisit untuk export selalu dinormalisasi ke urutan cerita (urutan klik tidak dibawa ke file hasil).
- Definisi selesai batch/daftar: bubble ber-`original` kosong diabaikan di backend dan frontend secara konsisten.

[unreleased]: https://github.com/yogaputra25/Kuron-studio/compare/v0.1.0...HEAD
