# Tech Stack — Kuron Studio

## Overview

```
Frontend (WebView)  <-->  Tauri Core (Rust)  <-->  OS / External
  Vite + Svelte/TS       tokio + crates         FS, Keychain, LLM APIs
```

## Frontend

| Layer | Pilihan | Alasan |
|-------|---------|--------|
| **Framework** | **SvelteKit** (rekomendasi) atau React | Svelte bundle kecil (~30KB), React jika tim familiar |
| **Build** | Vite 5 + TypeScript 5 | Fast HMR, Tauri default |
| **Styling** | Tailwind CSS 4 | Utility-first, konsisten dengan Kuron |
| **Canvas** | Konva.js atau Fabric.js | Bubble overlay, drag/resize, polygon shape |
| **State** | Svelte stores / Zustand (React) | Project, provider, glossary |
| **i18n** | `svelte-i18n` / `i18next` | en/id/zh (reuse Kuron l10n) |
| **Test** | Vitest + Playwright | Unit + canvas E2E |

**Alternatif canvas:** `pixi.js` jika butuh performa tinggi untuk 50 halaman, tapi Konva cukup untuk MVP.

## Tauri Core (Rust)

| Crate | Versi | Fungsi |
|-------|-------|--------|
| `tauri` | 2.x | Core, commands, events, window |
| `tauri-plugin-fs` | 2.x | File system (project dir) |
| `tauri-plugin-dialog` | 2.x | Open/save dialog |
| `tauri-plugin-http` | 2.x | LLM fetch (whitelist) |
| `tauri-plugin-store` | 2.x | App config (non-secret) |
| `tauri-plugin-stronghold` | 2.x | Encrypted store fallback |
| `keyring` | 3.x | OS keychain (Keychain/Credential Manager/Secret Service) |
| `tokio` | 1.x | Async runtime, mpsc queue |
| `serde` + `serde_json` | 1.x | Serialization |
| `image` | 0.25 | Decode/encode, crop, resize |
| `mozjpeg` / `jpeg-encoder` | - | JPEG encode (quality 75/85) |
| `rayon` | 1.x | Parallel batch image ops |
| `ort` | 2.x | ONNX Runtime (alternatif: `tract`) |
| `reqwest` | 0.12 | HTTP client (provider) |
| `rusqlite` / `sled` | 0.31 / 0.34 | Cache + glossary DB |
| `uuid` | 1.x | Project/page IDs |

## Rust Crates Internal (port dari Kuron)

```
src-tauri/src/crates/
├── image-ops/       # port kuron_native/rust/image_ops.rs
│   ├── build_mosaic.rs   # crop 20% pad, 2x, label merah, cap 2MB/1MB
│   ├── compress_page.rs  # 1280px longest, JPEG85
│   └── chunk_webtoon.rs  # long-strip split
├── onnx-detect/     # BubbleBox inference
│   ├── model.rs     # load bubble.onnx
│   └── postprocess.rs # merge, confidence filter
├── provider/        # AiTranslationProvider trait
│   ├── mod.rs       # trait + factory
│   ├── openai_compatible.rs
│   ├── gemini.rs
│   └── cohere.rs
├── prompt/          # prompt builder (port Dart)
│   ├── mosaic.rs    # buildMosaicPrompt + sfxRule + appendGlossary
│   └── full_image.rs
└── parser/          # ModelJsonParser
    └── mod.rs       # strip markdown, parse JSON, preview
```

## ONNX Model

- **Source:** Reuse `kuron_native` bubble detection model (`bubble.onnx`)
- **Bundle:** `src-tauri/resources/models/bubble.onnx` (~20-50MB)
- **Runtime:** `ort` (ONNX Runtime) — butuh binary, performa terbaik. Alternatif `tract` (pure Rust, lebih lambat, bundle kecil)
- **Inference:** `tokio::task::spawn_blocking` + `intra_threads=2` agar tidak block UI
- **Postprocess:** `postProcessBoxes` (merge overlapping, drop SFX jika skipSfx)

## Storage

| Data | Lokasi | Format |
|------|--------|--------|
| Project meta | `app_data_dir/projects/{id}.json` | JSON |
| Page cache | `app_data_dir/cache.db` (rusqlite) | key = hash(image+bubbles+lang+style) |
| Glossary | `app_data_dir/glossary.db` (rusqlite) | id, sourceText, translatedText, reading |
| App config | `app_data_dir/store.json` (tauri-plugin-store) | theme, lang, default provider |
| API keys | OS Keychain (`keyring`) | service=`id.kuron.studio`, account=`provider:{id}` |
| Logs | `app_log_dir/kuron-studio.log` | redact base64 & key |

## Security

- **ACL (`capabilities/default.json`):** hanya allow `fs:read` di project dir, `http:fetch` ke whitelist `defaultBaseUrl` (9 provider), `dialog:open`, `store`
- **Key handling:** `get_providers` return `apiKey: "***"` — key hanya di keychain, tidak di JSON
- **Logs:** redact base64 (preview 200 chars), tidak log key
- **CSP:** `default-src 'self'`, `connect-src` whitelist LLM domains

## Build & Release

| Target | Output | Tool |
|--------|--------|------|
| Windows | `.msi` / `.exe` (NSIS) | `tauri build --target x86_64-pc-windows-msvc` |
| macOS | `.app` / `.dmg` | `tauri build --target aarch64-apple-darwin` (universal) |
| Linux | `.deb` / `.AppImage` | `tauri build --target x86_64-unknown-linux-gnu` |
| Updater | delta | `tauri-plugin-updater` + GitHub Releases |

**CI:** GitHub Actions — `cargo test` + `cargo clippy` + `pnpm test` + `tauri build` (matrix: macOS/Windows/Linux)

## Reuse Map Kuron → Studio

| Kuron (Dart) | Studio (Rust) | Catatan |
|--------------|---------------|---------|
| `MosaicBuilder` | `image-ops::build_mosaic` | 100% Rust, cap + downscale loop |
| `FallbackImageHandler` | `image-ops::compress_page` | 1280px, JPEG85 |
| `BubbleBox` | `onnx-detect::BubbleBox` | serde, shape polygon |
| `openai_compatible_provider` | `provider::openai_compatible` | prompt + 429 handling |
| `AiProviderType` (9) | `provider::config` | defaultBaseUrl, modelsUrl |
| `TranslationStyle` (7) | `prompt::style` | instruction injection |
| `PageTranslation` | `model::translation` | rect, original, reading, translated |

## Decisions — RESOLVED 2026-09-08

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Frontend | **SvelteKit** | User benci React, bundle kecil, Svelte + Rust |
| ONNX | **`ort` untuk MVP** | <2s/1080p, model Kuron sudah ONNX Runtime; `tract` opsi lightweight M4 |
| DB | **`rusqlite`** | Glossary `glossary.db` local (jawaban Q5), SQL familiar |
| Canvas | Konva | Mudah vs Pixi performa tinggi |
| Keys | `keyring` | OS native vs `stronghold` encrypted file |
| Export | **JSON (P0) + PNG overlay + CBZ (P1)** | JSON wajib Kuron, PNG QA, CBZ distribusi; PSD deferred |
| Batch | **3 images/request** | Hemat AI (jawaban Q6), queue 50 halaman, semaphore 3 |
| Scope | **Public** | Provider + umum, GitHub Releases |
| Lisensi | **Boleh share** | Prompt/model Kuron boleh eksternal |

## Open Questions — RESOLVED

Semua 7 pertanyaan terjawab 2026-09-08. Tidak ada open question tersisa.
3. `rusqlite` vs `sled` — butuh SQL query glossary?
