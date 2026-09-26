# Proposal: Kuron Studio MVP — Manga Translation Workbench (Tauri v2)

## Why

Provider translate manga/manhwa kerja manual: crop bubble satu-satu, copy-paste ke LLM tanpa konteks reading order, tidak ada batch, glossary tidak konsisten, SFX sering salah. Kuron App sudah punya pipeline translate matang (ONNX on-device → `MosaicBuilder`/`FallbackImageHandler` → BYOK LLM 9 provider → overlay) tapi untuk **end-reader**, bukan untuk **provider** yang butuh workbench produksi batch + edit + export.

**Kuron Studio** adalah app desktop terpisah berbasis **Tauri v2 (Rust)** yang reuse 100% logic Kuron (prompt, mosaic, provider, BubbleBox) dengan UX untuk produksi: import 50 halaman → detect otomatis → edit di canvas → translate BYOK → post-edit → export JSON/PNG/CBZ. Target: 50 halaman mentah ke terjemahan siap export dalam < 30 menit.

Kenapa Tauri v2 + Rust:
- Reuse `kuron_native/rust` (`image_ops`, `imageproc`, ONNX) — port langsung, bukan rewrite
- Performa batch (rayon, tokio) + ONNX native, bukan Dart isolate
- Akses FS native (drag-drop folder/zip, export CBZ) — butuh desktop
- Bundle ~10MB vs Electron ~150MB, cross-platform 1 codebase (Win/macOS/Linux)
- BYOK aman via OS keychain (Keychain/Credential Manager/Secret Service)

Tauri v2 output: **Desktop stabil** (Win .exe/.msi, macOS .app/.dmg, Linux .deb/.AppImage). Mobile (Android/iOS) masih beta — deferred.

## What Changes

**App baru terpisah** (`id.kuron.studio`, repo `kuron-studio`) — tidak mengubah `nhasixapp` kecuali reuse model/prompt sebagai referensi.

- **Project & Import:** `create_project`, `import_pages` (folder/zip, JPG/PNG/WebP), thumbnail 512px, grid 50 halaman, status badges
- **ONNX Detect:** Rust `ort`/`tract` → `BubbleBox{x,y,w,h,confidence,shape,kind,tail}`, `detect_bubbles` + `detect_bubbles_batch` + `detect_progress` events, postprocess (confidence 0.25, giant-box filter)
- **Canvas Editor:** Konva overlay di atas page image, tools Rect/Ellipse/Freeform/Tail, drag/resize/delete, chip numbering RTL/LTR, shape polygon persistence
- **Translate:** `translate_page` (mosaic 20% pad 2x label merah cap 2MB/1MB downscale loop vs fallback 1280px JPEG85), reuse prompt Kuron 1:1 (`buildMosaicPrompt`, `fullImagePrompt`, `sfxRule`, `appendGlossary`), `ModelJsonParser`, `mapMosaicResult` + reattach shape + whitePatch
- **Provider:** 9 types (zen, openCodeGo, gemini, openAi, openRouter, metaAi, clinePass, cohere, custom), `AiProviderConfig`, live `list_models` (`modelsUrl`), `validate`, `isVisionCapable`, 429 handling, keychain storage (`keyring` + `stronghold` fallback), `get_providers` redacted
- **Glossary & Cache:** `glossary.db` (rusqlite) CRUD + prompt injection tanpa extra AI call, `cache.db` key hash(image+bubbles+lang+style)
- **Batch & Export:** `tokio::mpsc` + semaphore 3, `translate_batch` + `translate_progress` + `retry_bubble`, export JSON/PNG overlay/CBZ via dialog
- **Security & Polish:** `capabilities/default.json` ACL (fs:read project dir, http:fetch whitelist 9 `defaultBaseUrl`), redact base64/key di logs, i18n en/id/zh, theming, updater

## Capabilities

### New Capabilities
- `kuron-studio-project`: Project CRUD, import folder/zip, thumbnail, grid
- `kuron-studio-detect`: ONNX bubble detection on-device (Rust), batch + progress
- `kuron-studio-canvas`: Canvas editor bubble (Konva, shape-aware)
- `kuron-studio-translate`: Mosaic/fallback → BYOK LLM → PageTranslation overlay
- `kuron-studio-provider`: 9 BYOK provider types, LOV, validate, keychain
- `kuron-studio-glossary`: Glossary CRUD + prompt injection
- `kuron-studio-cache`: Translation cache (hash key)
- `kuron-studio-batch`: Queue batch translate (concurrency 3) + retry per-bubble
- `kuron-studio-export`: Export JSON/PNG overlay/CBZ
- `kuron-studio-security`: OS keychain + Tauri ACL + log redaction

### Modified Capabilities
- (none — app terpisah, tidak modifikasi `nhasixapp` capabilities)

## Impact

- **New repo:** `kuron-studio/` — `src/` (Svelte/React + Vite + Tailwind + Konva) + `src-tauri/` (Rust crates: `image-ops`, `onnx-detect`, `provider`, `prompt`, `parser`) + `resources/models/bubble.onnx`
- **Reuse map Kuron → Studio:** `MosaicBuilder`→`image-ops::build_mosaic`, `FallbackImageHandler`→`compress_page`, `BubbleBox`→`onnx-detect::BubbleBox`, `openai_compatible_provider`→`provider::openai_compatible`, `AiProviderType/Config`→`provider::config`, `TranslationStyle`→`prompt::style`, `PageTranslation`→`model::translation`, `GlossaryEntry`→`glossary::entry`, `TranslationCacheRepository`→`cache::sqlite`
- **No breaking change** ke Kuron App — hanya referensi prompt/model
- **Docs:** `Studio/docs/` (ide, target, tech-stack, flow, roadmap) + `Studio/openspec/` (config, specs, changes) — private storage sesuai preferensi user
- **CI:** GitHub Actions matrix (macOS/Win/Linux) — `cargo test/clippy` + `pnpm test` + `tauri build`

## Risks

| Risk | Mitigasi |
|------|----------|
| ONNX model Kuron tidak akurat di desktop resolusi berbeda | Threshold tuning (0.25) + giant-box filter + manual override di canvas |
| LLM JSON invalid / hallucinate | `ModelJsonParser` robust (strip markdown) + retry 1x + fallback ke full-image |
| 429 rate limit saat batch | Semaphore 3 + exponential backoff (2s,4s,8s) + `provider_rate_limited` event |
| Payload mosaic > limit provider | Cap 1MB low / 2MB high + downscale 0.75x loop sampai w<=64 |
| Key bocor | `keyring` OS keychain + `get_providers` redacted + redact logs (preview 200 chars) |
| Bundle >80MB (ONNX ~50MB) | Bundle on-demand download atau compress, updater delta |

## Non-Goals (MVP)

- Mobile (Android/iOS) — Tauri mobile masih beta
- Cloud sync / kolaborasi real-time
- Scraper source / reader
- Hosting LLM (BYOK saja)
- PSD export (post-MVP)

## Open Questions — RESOLVED 2026-09-08

| # | Pertanyaan | Jawaban User | Keputusan |
|---|------------|--------------|-----------|
| 1 | Scope provider | Public — provider + siapa saja yang mau pakai | App public, update via GitHub Releases, no auth gate |
| 2 | Frontend | Svelte + Rust (benci React) | **SvelteKit** + Tauri Rust, no React |
| 3 | ONNX runtime | Bagusnya gimana? | **Rekomendasi: `ort` untuk MVP** — <2s/1080p, model Kuron sudah ONNX Runtime, bundle 50MB OK desktop; `tract` opsi lightweight M4 |
| 4 | Export format | Bagusnya gimana? | **Rekomendasi: JSON (P0) + PNG overlay + CBZ (P1)**, PSD deferred — JSON wajib integrasi Kuron, PNG QA, CBZ distribusi |
| 5 | Glossary | DB local | **`rusqlite` `glossary.db`** local, bukan SharedPreferences JSON |
| 6 | Batch size | 3 image biar tidak berat ke AI | **3 images per translate request** (hemat token), project queue tetap support 50 halaman, semaphore 3 |
| 7 | Lisensi prompt/model | Boleh banget | Prompt/model Kuron boleh share ke eksternal |

## Success Criteria (MVP)

- Import 50 halaman <5s, detect 1 halaman 1080p <2s, mosaic <500ms, translate 1 halaman (3 images mosaic) <15s, batch 10 halaman (3 concurrency) <3m, bundle <80MB, 0 crash happy path
