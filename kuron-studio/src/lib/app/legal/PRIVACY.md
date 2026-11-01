# Kebijakan Privasi — Kuron Studio

**Versi 1.0 · 1 November 2026**

Kuron Studio dirancang dengan prinsip **local-first**: data kamu tinggal
di perangkatmu, bukan di server kami — karena memang tidak ada server kami.

## Data yang tidak pernah meninggalkan perangkat

- **Project dan gambar** — tersimpan di folder app-data sistem operasi
  (`Application Support/id.kuron.studio`), tidak pernah masuk ke repo
  kode, email, atau layanan pihak ketiga mana pun.
- **API key** — tersimpan di keychain sistem operasi (Keychain di macOS,
  Credential Manager di Windows). Aplikasi **tidak pernah** menulis
  secret ke log, file, database, atau clipboard.
- **Tanpa akun, tanpa analitik, tanpa pelacakan** — tidak ada identitas
  pengguna yang dikumpulkan, tidak ada perilaku yang direkam.

## Data yang keluar — hanya atas perintahmu

- Saat kamu menekan **Translate**, gambar halaman dan teks terkait dikirim
  ke **provider AI yang kamu pilih sendiri**, memakai **kunci API milikmu**.
- Tidak ada pengiriman data latar belakang untuk tujuan lain apa pun.

## Log diagnostik lokal

- Log tersimpan **lokal di perangkatmu** dan hanya dibaca saat kamu
  membukanya dari panel Diagnostics (misalnya untuk ditempel ke issue).
- API key selalu disamarkan menjadi `[REDACTED]` sebelum ditulis ke log.
