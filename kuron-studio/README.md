# Kuron Studio — Manga Translation Workbench (Tauri v2 + Svelte 5)

Desktop app (Indonesian-first): import pages → detect bubbles (ONNX) →
translate (9 AI providers) → batch → export JSON/PNG/CBZ.

## Quickstart

```sh
cd kuron-studio
pnpm install
pnpm tauri dev        # backend unreachable? run via this, not `pnpm dev`
pnpm dev:fresh        # curiga cache lama? hapus .vite lalu tauri dev (`-- --nuke` juga reset profile WebView2; data project tidak tersentuh)
```

Gates (must all be green):

```sh
cd kuron-studio/src-tauri
cargo test && cargo clippy --all-targets -- -D warnings
cd ..
pnpm test && pnpm check
```

## Flow

1. Buat project → Import folder/files (JPG/PNG/WebP, filename order).
2. Detect (tunggal / semua) → edit bubbles di canvas (Rect/Ellipse/Freeform/Tail).
3. Providers → simpan key (OS keychain `id.kuron.studio`, fallback sqlite) →
   pilih model → Validate → Translate (`<15s`, overlay + per-bubble edit).
4. Batch (3 paralel, progress bar) → Review (retry page/bubble) →
   Export JSON / PNG overlay / CBZ / PSD.
   PNG export auto-typesets via system TTF with bitmap fallback.

## Config notes

- **Updater (M4-7):** GitHub Releases endpoint sudah di `tauri.conf.json`
  (`plugins.updater.endpoints`); sebelum release:
  1. `pnpm tauri signer generate -w ~/.tauri/kuron.key`
  2. Paste public key gantikan `M4-PLACEHOLDER` pubkey.
  3. Set secrets `TAURI_SIGNING_PRIVATE_KEY` (+ password) di repo.
  4. Uncomment `updater-*` upload-artifact di `.github/workflows/ci.yml`.
- **ONNX (M4-9):** `resources/models/bubble.onnx` ~11.8MB (<80MB) → tetap
  bundled via `resources`, no on-demand download.
- **i18n (M4-5):** shell keys en/id/zh di `src/lib/i18n.ts`; tombol header
  `ID/EN/中文` cycler, persist `localStorage`.
- **Theme (M4-6):** toggle 🌙/☀️, `data-theme` + Tailwind custom variant.

## Release

```sh
cd kuron-studio
pnpm tauri build   # .msi / .dmg / .deb (+ updater artifacts bila signed)
```

Screenshots: `docs/` (TODO M4-11: tambah screenshot alur import→export).
