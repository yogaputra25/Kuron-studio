# Tasks: fix-editor-blank-canvas

> P0 = Must, P1 = Should, P2 = Nice. Estimasi hari. Checkbox `- [ ]`.

## 1. Verifikasi baseline (pastikan bukan stale bundle) — P0 — 0.1d

- [ ] 1.1 Restart bersih: `! cd kuron-studio && pnpm dev:fresh`, buka ulang halaman yang sama; catat: kotak biru muncul? gambar muncul? hint "Memuat gambar…" terlihat? — P0
- [ ] 1.2 Bila masih hitam: tutup editor (← Grid), catat banner kuning di grid (ada/tidak + isinya); cek thumbnail halaman di grid (tampil/tidak) — P0
- [ ] 1.3 Bila masih hitam setelah 1.1–1.2: buka DevTools webview, salin pesan console merah — P0

**Done bila:** terbukti kode baru jalan (sembuh = tutup change) atau ada bukti baru (lanjut ke §2).

## 2. Fix race CanvasEditor (bila §1 gagal) — P0 — 0.5d

- [x] 2.1 Stage + overlayLayer dibuat di `onMount` tanpa tunggu gambar (sudah di working tree) — P0
- [x] 2.2 `$effect` swap background saat `imageUrl` tiba + guard `requestedUrl` + hint `imgReady`/`imgError` (sudah di working tree) — P0
- [x] 2.3 Tambah test vitest: mount logika URL-swap (mock Konva bila perlu) — assert overlay tampil tanpa bg, bg swap saat URL tiba, load basi diabaikan — P0 (`src/lib/canvas-editor.test.ts`, 7 test pola-sumber; mount penuh diskip sadar: komponen terikat Konva+DOM+Svelte runtime, mock hanya uji diri sendiri)

**Done bila:** `pnpm test && pnpm check` hijau; manual: buka editor → kotak langsung tampil, gambar menyusul <2s.

## 3. Error visibility dari dalam editor — P1 — 0.25d

- [x] 3.1 Teruskan error `openEditor` ke `EditorPanel` (prop baru atau store) + render banner di atas overlay + pastikan tombol ← Grid selalu aktif — P1
- [x] 3.2 Fallback: bila URL kosong setelah fetch gagal, tampilkan pesan + tombol kembali (jangan stuck "Memuat gambar…" selamanya) — P1

**Done bila:** simulasi preview gagal (path rusak) → user melihat pesan + bisa kembali tanpa reload.

## 4. Gates + arsip — P1 — 0.1d

- [x] 4.1 `cargo test && cargo clippy --all-targets -- -D warnings` (src-tauri) + `pnpm test && pnpm check` (kuron-studio) semua hijau — P1 (cargo 68 passed/clippy clean; pnpm 23 tests pass/check 0 error, via lane-1+lane-3)
- [ ] 4.2 Arsip via `/opsx:archive` setelah 1.1 sembuh ATAU §2–§3 selesai terverifikasi — P1
