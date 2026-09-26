# Target — Kuron Studio

## Vision

> Menjadi workbench desktop #1 untuk provider translate manga/manhwa di ekosistem Kuron — dari 50 halaman mentah ke terjemahan siap export dalam < 30 menit.

## MVP Scope (V0.1 — 6-8 minggu)

### Must Have (P0)

| # | Fitur | Kriteria Selesai |
|---|-------|------------------|
| 1 | **Project & Import** | Buat project, import folder/zip (JPG/PNG/WebP), grid 50 halaman, thumbnail 512px |
| 2 | **ONNX Detect** | `detect_bubbles` per halaman + batch, overlay di canvas, confidence threshold |
| 3 | **Canvas Editor** | Rect/Ellipse/Freeform + tail, drag/resize, delete, chip numbering RTL/LTR |
| 4 | **Translate Single** | Pilih provider BYOK + style (7) + targetLang, mosaic/fallback, POST LLM, render overlay |
| 5 | **Per-Bubble Edit** | Edit original/reading/translated, font, whitePatch toggle, isUserEdited guard |
| 6 | **Provider Settings** | CRUD 9 provider types, live model LOV, validate, keychain storage |
| 7 | **Cache** | `TranslationCacheRepository` — key hash(image+bubbles+lang+style), clear |
| 8 | **Export JSON** | Export `PageTranslation` JSON per project (untuk integrasi Kuron App) |

### Should Have (P1 — MVP+)

| # | Fitur | Kriteria |
|---|-------|----------|
| 9 | **Batch Translate** | Queue 3 concurrency, progress events, retry failed bubbles |
| 10 | **Glossary** | CRUD + prompt injection (`appendGlossary`), import/export CSV |
| 11 | **Export PNG Overlay** | PNG dengan bubble overlay burned-in (untuk QA) |
| 12 | **Mosaic Quality** | Low (75%/1MB) vs High (85%/2MB) toggle |

### Nice to Have (P2 — Post-MVP)

| # | Fitur | Kriteria |
|---|-------|----------|
| 13 | Export CBZ/PDF burned-in | Full chapter export |
| 14 | TM Search | Cari terjemahan sebelumnya |
| 15 | QA Checks | Untranslated, overflow, SFX leak |
| 16 | Auto-updater | Tauri updater |

## Decisions — RESOLVED 2026-09-08

| # | Pertanyaan | Jawaban | Keputusan |
|---|------------|---------|-----------|
| 1 | Scope | Public — provider + umum | App public, GitHub Releases |
| 2 | Frontend | Svelte + Rust (benci React) | **SvelteKit** + Tauri Rust |
| 3 | ONNX | Bagusnya gimana? | **`ort` untuk MVP** — <2s, bundle 50MB OK |
| 4 | Export | Bagusnya gimana? | **JSON (P0) + PNG overlay + CBZ (P1)**, PSD deferred |
| 5 | Glossary | DB local | **`rusqlite` `glossary.db`** |
| 6 | Batch | 3 image per translate | **3 images/request** (hemat AI), queue 50 halaman, semaphore 3 |
| 7 | Lisensi | Boleh share | Prompt/model boleh eksternal |

## Success Criteria (MVP)

| Metric | Target |
|--------|--------|
| Import 50 halaman | < 5 detik (thumbnail async) |
| Detect 1 halaman (1080p) | < 2 detik (ONNX `ort`) |
| Mosaic build | < 500ms |
| Translate 1 halaman (3 images mosaic) | < 15 detik (tergantung LLM) |
| Batch 10 halaman (3 concurrency) | < 3 menit |
| Bundle size | < 30MB (tanpa model), < 80MB (dengan ONNX) |
| Crash rate | 0 pada happy path |

## Non-Goals MVP

- Mobile (Android/iOS) — deferred, Tauri mobile masih beta
- Cloud sync / kolaborasi real-time
- Scraper source / reader
- Hosting LLM (BYOK saja)
- PSD export

## Milestone

| Milestone | Durasi | Deliverable |
|-----------|--------|-------------|
| M0 Scaffold | 1 minggu | `create-tauri-app`, AppState, image preview |
| M1 Detect & Edit | 2 minggu | ONNX + canvas editor |
| M2 Translate | 2 minggu | Provider + prompt + cache + overlay |
| M3 Batch & Export | 2 minggu | Queue + glossary + export |
| M4 Polish | 1-2 minggu | Keychain, updater, i18n, theming |

## KPIs Post-Launch

- Provider adoption: 10 tim aktif dalam 3 bulan
- Avg time per chapter (20 halaman): < 20 menit
- Glossary hit rate: > 30%
- User retention (weekly): > 40%
