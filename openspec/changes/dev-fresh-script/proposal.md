## Why

Debug error `import_pages` kemarin mentok di satu pertanyaan: "yang jalan di `pnpm tauri dev` ini kode terbaru atau cache lama?" Tidak ada cara satu perintah untuk menjawabnya — cache tersebar di 4 lapisan (Vite `.vite`, cargo `target/`, WebView2 profile, dan data `projects.json` yang disangka cache). Verifikasi di mesin ini menemukan lokasi persisnya: profile WebView2 di `%LOCALAPPDATA%\id.kuron.studio\EBWebView`, data user di `%APPDATA%\id.kuron.studio\kuron-studio\` (`projects.json` + `kuron-studio.db`). Perlu satu perintah `pnpm dev:fresh` yang menjamin frontend segar tanpa menghancurkan data.

## What Changes

- Script baru `kuron-studio/scripts/dev-fresh.mjs` (Node stdlib only, tanpa dependency baru) + script npm `dev:fresh` di `kuron-studio/package.json`.
- Default: hapus `node_modules/.vite` (dep-optimizer Vite), cek port 1420 tidak dipakai proses lain, lalu jalankan `pnpm tauri dev`.
- Flag `--nuke`: hapus juga profile WebView2 dev (`%LOCALAPPDATA%\id.kuron.studio\EBWebView`) untuk kasus HTTP-cache/profile bandel. Default OFF.
- Tidak menyentuh: `target/` (cargo), `%APPDATA%\id.kuron.studio\` (projects.json + sqlite = DATA user), `dist/`.

## Scope

- `kuron-studio/scripts/dev-fresh.mjs` (file baru), `kuron-studio/package.json` (1 baris script), dan catatan 3-baris di `kuron-studio/README.md` bagian dev.
- Berlaku untuk dev loop Windows/macOS/Linux (script Node, bukan `.ps1`/`.sh`).

## Non-goals

- Tidak wipe `target/` (rebuild 30 detik jadi 5+ menit; fingerprint cargo terpercaya — verifikasi cukup `cargo build`).
- Tidak hapus `projects.json` / `kuron-studio.db` / folder `_extracted` — itu data, bukan cache, selamanya di luar script ini.
- Tidak ubah `vite.config.ts`, `tauri.conf.json`, atau perilaku `pnpm dev` / `pnpm tauri dev` yang sudah ada.
- Tidak menyelesaikan verifikasi manual `fix-import-pages-project-id` 1.2/4.3 — script ini alatnya, verifikasinya tetap manual.

## Capabilities

### New Capabilities
- `dev-fresh`: Perintah dev-loop yang menjamin frontend yang jalan = source di disk (bersih dari Vite optimizer cache + opsional WebView2 profile), tanpa menyentuh data user dan tanpa rebuild Rust yang tidak perlu.

### Modified Capabilities
- (none — tidak ada requirement di `openspec/specs/kuron-studio/spec.md` yang berubah.)

## Impact

- **Code:** 1 file baru (`scripts/dev-fresh.mjs`), 1 baris `package.json`, ±3 baris `README.md`. Tanpa perubahan Rust, tanpa dependency baru, tanpa perubahan CI.
- **Dev UX:** `pnpm dev:fresh` (aman, default) dan `pnpm dev:fresh -- --nuke` (destruktif terhadap profile dev: localStorage dev hilang — eksplisit via flag).
- **CI:** tidak tersentuh (script hanya untuk loop lokal).

## Risks

| Risk | Mitigasi |
|------|----------|
| `--nuke` menghapus localStorage/profile yang masih dibutuhkan | Default OFF; flag eksplisit; script print path yang dihapus sebelum menghapus; tidak pernah menyentuh `%APPDATA%` (data) |
| Lokasi EBWebView beda di macOS/Linux (`~/Library/...`, `~/.local/share/...`) | Script deteksi `process.platform` dengan path per-OS; bila folder tidak ada → skip dengan pesan, bukan error |
| Port 1420 dipakai Vite zombie → `tauri dev` gagal dengan error `strictPort` yang samar | Script cek port dulu dan abort dengan pesan jelas (pid/proses bila bisa dibaca) |
| Script jadi tempat sampah flag ("sekalian wipe target") | Kontrak di spec: daftar hapus bersifat closed-list; penambahan target baru butuh amend spec |
