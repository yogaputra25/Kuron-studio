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

- #14 (OPEN) — drawer mobile + pisah UI desktop/tablet vs mobile. Satu-satunya sisa gelombang redesign; baru ada spec (`mobile-home-drawer`), belum implementasi.
- #29 (OPEN), #38/#39 (epic), #50–#67 (OPEN) — refactor backend + fitur batch/AI; di luar scope kerja Yoga 2026-10-31.
- Ditutup 2026-10-31 (kerja Yoga, diverifikasi 2026-10-31): #7 folder + MoveDialog, #12 grid/list + #25 sorting, #17 detect-ux, #26/#27 provider + #28 inpaint, #37 epic frontend, #40–#49 (FE-1 domain-move … FE-9 i18n-keys + token ilustrasi).

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
