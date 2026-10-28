# Log sesi Kuron Studio

Append-only: entri baru selalu di **bawah**, tidak pernah edit/sisip di entri lama.
`merge=union` di `.gitattributes` membuat append paralel dari branch berbeda
bergabung otomatis tanpa konflik. Kalau >200 baris, ringkas entri tertua jadi
satu paragraf dan hapus yang diringkas (kompaksi = satu-satunya edit yang boleh
menyentuh baris lama).

## 2026-09-26 — Implementasi awal (rekonstruksi dari git history)

- Kerja: Implementasi awal Kuron Studio (`777aed0`).
- Hasil: Pondasi Tauri v2 + Svelte 5 jalan (import, deteksi ONNX, translate, ekspor).

## 2026-09-29 — Bugfix UI + in-bubble edit (rekonstruksi dari git history)

- Kerja: PR #1 (binding Field/Select, overlay terjemahan, token desain, logging diagnostik) dan PR #2 (design system + diagnostics + in-bubble edit, draft-local, last-write-wins, reset-translation cancel). `9c8263c` tulis panduan AGENTS.md + MEMORY.md pertama.
- Hasil: Keduanya MERGED. Editor bisa edit langsung di bubble.

## 2026-10-10 — Reset-translation dihapus, cancel per-page (rekonstruksi dari git history)

- Kerja: PR #3 — typing comfort + pane switching (`dadda1c`); hapus fitur reset-translation, ganti cancel per-page yang tidak menimpa edit user + jaga backward compatibility (`ea7387a`).
- Hasil: MERGED. Edit user tidak lagi tertimpa baseline AI.
- Keputusan: translasi tidak menimpa edit user; cancel tanpa invoke AI ulang.

## 2026-10-12 — PDF import + iOS picker (rekonstruksi dari git history)

- Kerja: PDF import + feedback drawing tools (`63117c8`); iOS document picker + normalisasi path (`2731c3f`); openspec skills diperkaya (`5740cac`).
- Hasil: Masuk main. Import terima PDF; iOS bisa pilih dokumen.

## 2026-10-18 — Bubble edit + draw gestures + export wrap (rekonstruksi dari git history)

- Kerja: PR #4 bubble bisa diedit (`3573782`); PR #5 isolasi gesture draw-tool + reset tool setelah draw (`9507a7e`); PR #6 selaraskan layout teks canvas vs ekspor + wrapping (`61a0e1b`).
- Hasil: Ketiganya MERGED.

## 2026-10-28 — Gelombang redesign: tablet, perf, folder, onboarding (rekonstruksi dari git history)

- Kerja: PR #9/#10 sidebar resizable + tablet mode (tutup #8); PR #13 docs arsitektur + aset redesign; PR #15/#18 redesign UI + dropdown project + referensi desain; PR #16 antrean thumbnail viewport + cache preview Rust 12/15 (tutup #11); PR #19 nested folders + move dialog + onboarding (sebagian #7); PR #20 busy-state onboarding; PR #21 motion polish.
- Hasil: Semua MERGED ke main hari yang sama. Bukti visual v1 di `design/references-v1/` (19 screenshot + temuan: Escape tak tutup panel, detect tanpa progres, batch disabled).
- Lanjut: issue terbuka #7 (sisa), #12, #14, #17.

## 2026-10-29 — Setup AGENTS.md + MEMORY.md + DESIGN.md

- Kerja: Setup `AGENTS.md` (aturan) + `MEMORY.md` (pekerjaan); tulis `DESIGN.md` (token `--ks-*`, peran coral/cyan, 6 pola layout), tautkan dari `AGENTS.md`.
- Hasil: Selesai, gate tidak tersentuh.
- Keputusan: `AGENTS.md` = aturan, `MEMORY.md` + log = pekerjaan; riwayat awalnya per-file lalu digabung ke log ini + `merge=union`.

## 2026-10-29 — Backlog diluruskan dari GitHub issues

- Kerja: Rekonsiliasi dengan `gh issue list` + `git log`. Koreksi: #8 CLOSED (bukan aktif), #7 OPEN (bukan selesai).
- Hasil: Selesai, kode tidak tersentuh.
- Keputusan: backlog = cermin GitHub issues yang OPEN; issue ditutup → coret di `MEMORY.md`.
- Lanjut: kerjakan issue terbuka #17, #14, #12, sisa #7.

## 2026-10-29 — Aturan no-secret + log jadi satu file

- Kerja: Gabung 9 file `memory/` ke `memory/log.md` + `.gitattributes merge=union` (terbukti auto-merge). Scan docs: bersih, tanpa secret nyata.
- Hasil: Selesai, kode tidak tersentuh.
- Keputusan: secret (API key, password, signing key, isi keychain/`.env`) tidak pernah ke repo termasuk log/rules; temuan di output → `[REDACTED]` (aturan no. 8 di `AGENTS.md`).
- Lanjut: —
