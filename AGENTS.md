# AGENTS.md — Kuron Studio

Aturan kerja AI assistant. Baca sebelum mengubah kode.

## Wajib baca dulu

| Kalau mau | Baca |
|---|---|
| Cara jalanin, gate, struktur | `README.md` (root) |
| Peta sistem, alur runtime, aturan taruh logika | `ARCHITECTURE.md` |
| Aturan visual, token, layout | `DESIGN.md` |
| Detail frontend/backend, config M4 | `kuron-studio/README.md` |
| Spesifikasi & rencana kerja | `openspec/` via `/opsx:*` |
| Status lintas sesi | `MEMORY.md` |

## Perintah

Jalankan dari `kuron-studio/` kecuali ditulis lain:

- Dev: `pnpm tauri dev` (bukan `pnpm dev` — backend mati tanpa Tauri).
- Cache nyangkut: `pnpm dev:fresh` (`-- --nuke` reset profil WebView).
- Gate (wajib hijau sebelum push/PR, sama dengan CI):
  `cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings`
  `cd .. && pnpm test && pnpm check`
- Satu test: `cargo test <nama>` / `pnpm vitest run <file>`.

## Aturan taruh logika

1. Frontend (`src/`) = tampilan + state sementara. Backend (`src-tauri/`) = file, model ONNX, AI, antrean. Svelte tidak akses fs/model langsung.
2. Kontrak invoke: key top-level `snake_case` persis param Rust (`project_id`), field struct `camelCase`. Dijaga `invoke-contract.test.ts`.
3. `IMAGE_EXTS` di `lib/types.ts` wajib sinkron dengan `project.rs`. Dijaga `image-ext-contract.test.ts`.
4. String UI baru = 3 bahasa (`id/en/zh`) di `i18n.ts` dalam commit yang sama.
5. Style: token `--ks-*` di `app.css` saja, dark default + light. Tanpa nilai luar token. Detail di `DESIGN.md`.
6. Data proyek + API key di folder data OS / keychain (`id.kuron.studio`). Tidak pernah di repo.
7. Mobile gating di `device.ts` (tanpa folder-picker/updater/PNG-PSD). Dijaga `mobile-gating.test.ts`.
8. Secret (API key, password, private/signing key, isi keychain/`.env`) tidak pernah ditulis ke repo — termasuk `MEMORY.md`, `memory/log.md`, dan file aturan. Nemu secret di output (diagnostics, env, log) → tulis `[REDACTED]`.

## Alur kerja

- Perubahan perilaku → lewat `openspec/` (`/opsx:propose`, `/opsx:apply`, `/opsx:archive`). Jangan lompat langsung ke kode untuk fitur/fix non-trivial.
- Ikuti pola file yang diubah; jangan bikin konvensi kedua.
- Hapus scaffold/throwaway setelah smoke proof. Verifikasi = jalankan programnya, bukan cuma test hijau.

## Memori

- Riwayat = `memory/log.md`, append-only (entri baru di bawah, jangan edit baris lama). Append paralel digabung otomatis via `merge=union` di `.gitattributes`.
- Baca di awal: `MEMORY.md` + ekor log. Tulis di akhir: satu entri baru (Kerja, Hasil, Keputusan, Lanjut).
- `MEMORY.md` hanya untuk yang menetap (fokus, keputusan, backlog). Satu sumber kebenaran, tanpa duplikasi ke file lain.
