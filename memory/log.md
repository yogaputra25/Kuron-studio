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

## 2026-10-29 — detect-ux-pack 15/17 di atas mainline baru (isu #17)

- Kerja: Implementasi overlay progres Detect (`DetectProgress.svelte` + `reduceDetectProgress` + `listen detect_progress` + guard `detectGen`), backend lanjut-saat-gagal (`collect_batch` + event `ok/message`, test `gagal_tengah_lanjut_dan_urutan_utuh`), 1 tombol Import + menu pola switcher, header dua baris (`icon.svg` + tagline). Base pindah tengah jalan (main merge PR #16/#19–#22 ke `fa26334`); diff diverifikasi tanpa revert + gates diulang di base baru.
- Hasil: `pnpm test` 395/395, `pnpm check` 0 error, `cargo test` 114 lulus, `clippy -D warnings` bersih.
- Keputusan: tombol overlay = "Tutup" (dismiss, bukan cancel — backend tak punya cancel-flag); transport event diperkaya `ok/message` tanpa ubah wire invoke; `move_pages` main yang sempat tertelan resolusi dikembalikan verbatim (compiler menangkap).
- Lanjut: smoke `tauri dev` 25 halaman (1.5) + E2E manual isu #17 (5.4).

## 2026-10-29 — detect-ux-pack 17/17: smoke + E2E manual lulus (isu #17)

- Kerja: Verifikasi manual oleh user (25 halaman, counter + filename + Tutup tengah jalan, 1 file rusak, mobile/sempit).
- Hasil: LULUS. Change 17/17 complete. Siap archive.
- Lanjut: `/opsx:archive` untuk detect-ux-pack.

## 2026-10-29 — Arsip massal 15 change + sync 15 spec baru (opsi 1)

- Kerja: Sync 15 main spec baru (verbatim ADDED + TBD Purpose) + 3 merge MODIFIED ke `kuron-studio` (batch import viewport/cache; 2 skenario manual-edit baru — sisanya sudah tercakup main). Perbaikan normatif: 4 badan before-after-split + 2 preview-transport tanpa SHALL/MUST. Arsip 15 change ke `archive/2026-10-29-*`. Validasi akhir: 20/20 spec strict hijau, tanpa delta header tersisa.
- Hasil: Semua change terarsip. `openspec/changes/` tinggal `archive/` (26 entri). MAIN SPECS bertambah 15.
- Catatan: `kuron-studio-mvp` tanpa delta spec → arsip langsung. Purpose 15 spec baru = TBD, perbarui menyusul.
- Lanjut: commit + push (kode detect-ux-pack + grid-preview-perf masih uncommitted di worktree).

## 2026-10-29 — page-list-view 11/13 di worktree (isu #12, opsi A frontend saja)

- Kerja: Helper sortir (`sortPagesByStatus/Bubbles`, `fileName`, `parentFolder`) + test; `PageGrid` (ekstrak + prefix folder) + `PageList` (sortir, keyboard j/k/Enter/Space, roving focus, skeleton) + `pageView.test`; switcher Grid/List (`tablist`, G/L, pref localStorage, scroll per mode); retarget 4 test pola-sumber yang pindah file (motion, panel-tablet, previewGrid, select-mode).
- Hasil: `pnpm test` 405/405, `pnpm check` 0 error, `cargo test` 114 lulus, clippy bersih. Main maju PR #23 (merge grid-preview-perf, tanpa diff konten) — tanpa konflik.
- Keputusan: kolom ukuran = dimensi `WxH` (tanpa byte, tanpa backend) — opsi A sesuai pilihan user.
- Lanjut: smoke keyboard-only + narrow viewport (3.3), E2E 52/200 halaman (4.3) — butuh GUI + data nyata.

## 2026-10-29 — Arsip grid-sort 9/9 + sync spec (isu #25)

- Kerja: Sync `openspec/specs/grid-sort/spec.md` baru (3 requirement ADDED verbatim; Purpose TBD). Arsip ke `archive/2026-10-29-grid-sort/`.
- Hasil: Valid strict hijau. Semua artifact done, semua task complete.
- Lanjut: —

## 2026-10-29 — export-scope 10/11 (filter folder + ekspor terpilih + hapus tab QA/TM)

- Kerja: Filter folder di Hasil & Ekspor + `folder_id` diteruskan; `page_ids` opsional di `export_project` + `select_pages` tunggal (skip terhitung, urutan input); scope "Semua/Folder/Yang dipilih" + hint; QA/TM dihapus dari panel (backend utuh); i18n `exportScope*` 3 bahasa.
- Hasil: `pnpm test` 405/405, `pnpm check` 0, `cargo test` 116 lulus (termasuk `select_pages_subset_skips_missing` + `pages_in_selection_order_and_skips_missing`), clippy bersih. Kontrak invoke diperbarui (tambah `page_ids`).
- Lanjut: smoke 3.3 (455 baris → filter folder → ekspor folder/terpilih/skip; mobile).

## 2026-10-29 — export-scope: breadcrumb folder ganda (temuan user, 455 hlm)

- Kerja: Filter folder menampilkan breadcrumb (`default / bleach`) untuk folder bersarang via `folderPath()`; berlaku juga di label scope ekspor. Tanpa sentuh data.
- Akar: data user memang berisi 2 folder beda bernama "bleach" (1 root + 1 di dalam default, isi halaman berbeda) — bukan bug hitung. UI lama menampilkan nama datar sehingga terlihat duplikat.
- Hasil: `pnpm test` 405/405, `pnpm check` 0 error.
