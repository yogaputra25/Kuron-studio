# Kebijakan Privasi — Kuron Studio

Versi 1.0 — 1 November 2026.

Kuron Studio dirancang local-first: data kamu tinggal di perangkatmu.

## Data yang TIDAK pernah keluar dari perangkat

- Project, gambar halaman, hasil deteksi, dan terjemahan tersimpan di
  folder app-data OS (`Application Support/id.kuron.studio`), tidak pernah
  di repo kode atau server kami — karena memang tidak ada server kami.
- API key disimpan di keychain OS (macOS Keychain / Credential Manager).
  Aplikasi tidak pernah menulis secret ke log, file, atau clipboard.
- Tidak ada akun, tidak ada analitik, tidak ada pelacakan.

## Data yang keluar — hanya saat kamu meminta

- Saat kamu menekan Translate, gambar halaman + teks terkait dikirim ke
  provider AI yang kamu pilih sendiri (kunci API milikmu).
- Tidak ada pengiriman latar belakang untuk tujuan lain.

## Log lokal

- Log diagnostik tersimpan lokal di perangkatmu dan hanya dibaca saat
  kamu membukanya dari panel Diagnostics (mis. untuk ditempel ke issue).
  API key selalu disamarkan (`[REDACTED]`) sebelum ditulis ke log.
