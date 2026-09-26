# Kuron Studio — Project Memory

> Catatan lintas sesi. Baca di awal setiap sesi. Update setelah perubahan penting.

## Identity

| Key | Value |
|-----|-------|
| **Name** | Kuron Studio |
| **Tagline** | Manga Translation Workbench |
| **Bundle ID** | `id.kuron.studio` |
| **Repo (rencana)** | `kuron-studio` (terpisah dari `nhasixapp`) |
| **Stack** | Tauri v2 (Rust) + Vite + Svelte/React + ONNX |
| **Platform** | Desktop: Windows, macOS, Linux (Tauri v2 stable). Mobile: deferred (beta) |
| **Status** | Ideation — belum scaffold `create-tauri-app` |
| **Parent** | Kuron App (Flutter, `nhasixapp`) — reuse prompt/model/logic |

## Decisions Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-09-08 | Nama `Kuron Studio` untuk awal | Manfaatkan brand Kuron, mudah rebrand nanti ke Kotoba/PanelForge |
| 2026-09-08 | Tauri v2 + Rust sebagai source of truth | Reuse `kuron_native/rust` (image_ops, ONNX), performa batch, akses FS native |
| 2026-09-08 | BYOK + OS keychain | Aman, tidak simpan key plaintext, konsisten dengan Kuron App |
| 2026-09-08 | Reuse prompt Kuron 1:1 | Konsistensi hasil translate, tidak reinvent prompt |
| 2026-09-08 | Mosaic + Fallback port 100% Rust | Hilangkan fallback Dart, cap 2MB/1MB + downscale loop |
| 2026-09-08 | Struktur docs + openspec di `Studio/` | Catatan SDD siap sebelum scaffold, private storage (user pref) |
| 2026-09-08 | Scope: public (provider + umum) | Jawaban Q1 — app untuk provider dan siapa saja yang mau pakai |
| 2026-09-08 | Frontend: Svelte + Rust (benci React) | Jawaban Q2 — Svelte ringan, Rust backend, no React |
| 2026-09-08 | ONNX: `ort` untuk MVP (rekomendasi) | Jawaban Q3 — performa <2s, bundle 50MB OK desktop; `tract` opsi lightweight nanti |
| 2026-09-08 | Export: JSON (P0) + PNG overlay + CBZ (P1) | Jawaban Q4 — JSON wajib integrasi Kuron, PNG QA, CBZ distribusi; PSD deferred |
| 2026-09-08 | Glossary: rusqlite local DB | Jawaban Q5 — `glossary.db` local, bukan SharedPreferences JSON |
| 2026-09-08 | Batch: 3 images per translate | Jawaban Q6 — hemat token/AI, semaphore 3, queue tetap support 50 halaman project |
| 2026-09-08 | Prompt/model boleh share eksternal | Jawaban Q7 — lisensi OK untuk provider eksternal |

## Kuron App Reuse Map

| Kuron (Flutter/Dart) | Kuron Studio (Rust) | Catatan |
|----------------------|---------------------|---------|
| `MosaicBuilder.buildMosaic` | `image-ops::build_mosaic` | Crop 20% pad, 2x scale, label merah, JPEG 85/75, cap 2MB/1MB |
| `FallbackImageHandler.compressPage` | `image-ops::compress_page` | 1280px longest, JPEG85 |
| `image_ops_chunk_webtoon` | `image-ops::chunk_webtoon` | Long-strip split |
| `BubbleBox` | `onnx-detect::BubbleBox` | x,y,w,h, confidence, shape, kind, tail |
| `openai_compatible_provider.dart` | `provider::openai_compatible` | Prompt mosaic/full, sfxRule, glossary append |
| `gemini_translation_provider.dart` | `provider::gemini` | Google REST |
| `cohere_translation_provider.dart` | `provider::cohere` | /v2/chat |
| `AiProviderType` + `AiProviderConfig` | `provider::config` | 9 types, defaultBaseUrl, modelsUrl |
| `TranslationStyle` (7) | `prompt::style` | natural/genz/action/romantis/formal/kasar/literal |
| `PageTranslation` / `BubbleTranslation` | `model::translation` | rect, original, reading, translated, shape |
| `GlossaryEntry` | `glossary::entry` | sourceText, translatedText, reading |
| `TranslationCacheRepository` | `cache::sqlite` | key = hash(image+bubbles+lang+style) |

## Open Questions — RESOLVED 2026-09-08

| # | Pertanyaan | Jawaban | Status |
|---|------------|---------|--------|
| 1 | Scope provider | Public — untuk provider + siapa saja yang mau pakai app | ✅ |
| 2 | Frontend | Svelte + Rust (benci React) — SvelteKit + Tauri Rust backend | ✅ |
| 3 | ONNX runtime | `ort` untuk MVP (rekomendasi, lihat bawah) | ✅ |
| 4 | Export format | JSON (P0) + PNG overlay + CBZ (P1), PSD deferred (rekomendasi) | ✅ |
| 5 | Glossary | DB local — `rusqlite` `glossary.db` | ✅ |
| 6 | Batch size | 3 images per translate (hemat AI), project tetap support 50 halaman queue | ✅ |
| 7 | Lisensi prompt/model | Boleh share ke eksternal | ✅ |

**Rekomendasi Q3 — ONNX runtime:**
- **MVP: `ort` (ONNX Runtime)** — performa <2s/1080p, model Kuron sudah ONNX Runtime, bundle ~50MB OK untuk desktop. `tract` (pure Rust) lebih kecil (~5MB) tapi ~2x lebih lambat dan perlu konversi model — cocok sebagai opsi lightweight nanti (M4 bundle optimization).

**Rekomendasi Q4 — Export:**
- **P0 JSON** — wajib untuk integrasi Kuron App (`PageTranslation` per page)
- **P1 PNG overlay + CBZ** — PNG burned-in untuk QA, CBZ (zip PNGs) untuk distribusi chapter
- **P2 PSD** — layer per bubble untuk editor profesional, deferred post-MVP

## Next Actions

- [x] Jawab 7 pertanyaan — resolved 2026-09-08
- [ ] `pnpm create tauri-app kuron-studio --template svelte`
- [ ] Port `image-ops` crate dari `kuron_native/rust`
- [ ] Bundle ONNX model `bubble.onnx` ke `src-tauri/resources/models/` (ort)
- [ ] Implement `detect_bubbles` command + canvas editor (Konva, Svelte)
