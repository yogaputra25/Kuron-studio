# Tasks: fix-import-pages-project-id

> P0 = Must (blocker M0), P1 = Should, P2 = Nice. Estimasi hari. Checkbox `- [ ]`.
> Arah fix = OPSI B: Rust menyesuaikan via `rename_all = "snake_case"` (30 command, sudah dikerjakan main session); frontend TIDAK diubah.

## 1. Reproduksi & bukti akar — P0 — 0.5d

- [x] 1.1 Salin exact error satu baris penuh + catat jalur pemicu (klik import / drag-drop / dialog / invoke manual di console) — P0
- [ ] 1.2 Jalankan `pnpm tauri dev` fresh (bukan `pnpm dev`, bukan binary lama) lalu ulangi jalur pemicu; catat apakah error tetap muncul — P0. Catatan: `dev:fresh` sudah jalan tapi error tetap → bukti BUKAN cache/stale bundle, melainkan default macro Tauri.
- [x] 1.3 Cari semua call-site `import_pages` di luar `kuron-studio/src/lib/api.ts` (grep `invoke(` + `import_pages` + `projectId` di `kuron-studio/src/`) — P0. Hasil: NIHIL — hipotesis pemanggil liar gugur; akar = `tauri-macros` default camelCase (`wrapper.rs:51`) + `command.rs:112` (`v.get(self.key)`).

**Done bila:** akar terbukti macro default (screenshot + source Tauri di mesin), bukan pemanggil liar / stale bundle.

## 2. Tambah rename_all ke 30 command + audit se-pola — P0 — 0.5d (SUDAH DIKERJAKAN main session)

- [x] 2.1 Tambah `#[tauri::command(rename_all = "snake_case")]` ke 30 command di `commands/*.rs` (project, image, detect, translate, batch, provider, glossary, export, extras) — P0
- [x] 2.2 Audit semua key lapis-1 di `api.ts` vs param Rust (tabel D1–D9 di design.md): `project_id`, `page_id`, `page_ids`, `provider_id`, `bubble_texts`, `path`, `format`, `query`, `limit`, `csv`, `bubbles`, `source/target/id`, `max_side`, `name`, `input` — P0. Hasil: `api.ts` sudah benar, TIDAK diubah.
- [x] 2.3 Mismatch se-pola di command lain (`export_project`, `share_project`, `qa_check`, dst.) ikut ter-cover oleh atribut yang sama; yang beda pola jadi follow-up task — P1

**Done bila:** 30 command ber-atribut; `api.ts` 100% match param Rust; tidak ada `invoke` langsung di luar `api.ts`.

## 3. Regression guard kontrak invoke — P0 — 0.5d

- [x] 3.1 Tambah guard statis di `pnpm test` (vitest kecil / script, tanpa dependency baru): ekstrak top-level keys tiap `invoke("<cmd>", {...})` literal di `kuron-studio/src/`, bandingkan dengan daftar param snake_case per command; isi `{ input }` dikecualikan — P0
- [x] 3.2 Guard gagal dengan pesan jelas (command + key salah + key yang diharapkan) saat diberi `invoke("import_pages", { projectId, paths })` — buktikan dengan satu kasus negatif sementara lalu hapus — P0
- [x] 3.3 Pertegas NOTE di atas `api.ts` menjadi kontrak dua lapis (lapis-1 snake_case vs isi-`input` camelCase) — P2

**Done bila:** `pnpm test` merah saat ada key camelCase lapis-1, hijau saat kontrak dipenuhi.

## 4. Verifikasi gates + manual — P0 — 0.5d

- [x] 4.1 `cd kuron-studio/src-tauri && cargo test && cargo clippy --all-targets -- -D warnings` hijau — P0
- [x] 4.2 `cd kuron-studio && pnpm test && pnpm check` hijau — P0
- [ ] 4.3 Manual `pnpm tauri dev`: import folder + zip + drag-drop → grid thumbnails tampil, skip non-image terlaporkan, tanpa error invalid args — P0. Catatan: butuh run manual ulang SETELAH atribut rename_all terpasang untuk konfirmasi error hilang.

**Done bila:** semua gates hijau + exit M0 (import → grid) kembali jalan.
