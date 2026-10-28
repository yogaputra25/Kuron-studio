# MEMORY.md — Penyimpan Pekerjaan Kuron Studio

Catatan lintas sesi untuk AI assistant. `AGENTS.md` bikin aturan, file ini + `memory/log.md` simpan pekerjaan.

## Cara pakai (anti-bentrok)

- Riwayat sesi = `memory/log.md`, **append-only**: entri baru selalu di bawah,
  tidak pernah edit/sisip di entri lama. Format: `## YYYY-MM-DD — slug` + Kerja, Hasil, Keputusan, Lanjut.
- Append paralel aman: `.gitattributes` (`merge=union`, built-in git) menggabungkan
  append dari branch berbeda otomatis tanpa konflik.
- Awal sesi baca: file ini + ekor log (`tail -30 memory/log.md`).
- Akhir sesi tulis: satu entri baru di bawah log.
- Secret tidak pernah masuk log: API key, password, isi keychain/`.env` → tulis `[REDACTED]`.
- Bagian di bawah (Fokus, Keputusan, Backlog) hanya untuk yang **menetap**; sentuh seperlunya.
- Log >200 baris → ringkas entri tertua jadi satu paragraf (satu-satunya edit
  yang boleh menyentuh baris lama).

## Fokus

- #17 (OPEN) — progres Detect semua (counter + %), tombol Import gabungan, header logo dua baris.
- #14 (OPEN) — drawer mobile + pisah UI desktop/tablet vs mobile.
- #12 (OPEN) — mode Grid dan List daftar halaman.
- #7 (OPEN, sebagian) — nested folders + move dialog sudah merge (PR #19); sisa tracking todo/done + dashboard.

## Keputusan

- 2026-10-29: `AGENTS.md` = aturan, `MEMORY.md` + `memory/log.md` = pekerjaan. Alasan: satu sumber kebenaran, tanpa duplikasi.
- 2026-10-29: Riwayat append-only di satu file + `merge=union`. Alasan: tabel di file bersama = semua sesi edit hunk yang sama = bentrok; banyak file = ganggu.
- 2026-10-29: Backlog = cermin GitHub issues yang OPEN; issue ditutup → coret di sini. Riwayat = `git log` + `memory/log.md`.
- ONNX `bubble.onnx` tetap bundled via `resources`, tanpa download on-demand.
- Kontrak invoke `snake_case` top-level / `camelCase` field; `IMAGE_EXTS` sinkron dua sisi.
- Translasi tidak menimpa edit user; cancel tanpa invoke AI ulang (2026-10-10, PR #3).

## Backlog

- Selesai: #8 panel resize + tablet (PR #10); #11 perf preview cache (PR #16).
- M4-11 TODO: screenshot alur import→export di `docs/`.
- Updater M4-7: ganti pubkey `M4-PLACEHOLDER`, set secret signing, uncomment upload `updater-*` sebelum release.

## Riwayat sesi

Sengaja tidak ada tabel di sini — tabel bersama adalah sumber bentrok.
Baca `memory/log.md`; entri terbawah = status terakhir.
