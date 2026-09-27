# Tasks: Kuron Studio MVP

> Urut sesuai roadmap M0-M4. P0 = Must, P1 = Should, P2 = Nice. Estimasi hari. Checkbox `- [ ]`.

## M0 — Scaffold (Minggu 1) — P0

- [x] **M0-1** Scaffold `pnpm create tauri-app kuron-studio --template svelte` (atau react sesuai keputusan) — 1d — P0
- [x] **M0-2** Konfigurasi `src-tauri/tauri.conf.json` (bundleId `id.kuron.studio`, window 1280x800, bundle targets) — 0.5d — P0
- [x] **M0-3** Setup `src-tauri/capabilities/default.json` ACL (fs:read/write project dir, dialog, http whitelist, store) — 0.5d — P0
- [x] **M0-4** Implement `AppState` (Mutex<HashMap<String, Project>>, OnceLock<BubbleDetector>, reqwest Client, rusqlite Connection) + `lib.rs` setup — 1d — P0
- [x] **M0-5** Command `create_project` + `get_project` + `store.json` persistence — 1d — P0
- [x] **M0-6** Command `import_pages` (folder/zip, JPG/PNG/WebP, filename order, skip non-image) — 1.5d — P0
- [x] **M0-7** Command `get_image_preview` (thumbnail 512px, image crate resize) — 0.5d — P0
- [x] **M0-8** Frontend: Project grid UI (Svelte, Tailwind, thumbnail grid, status badges Idle/Detecting/Translated/Failed) — 2d — P0
- [x] **M0-9** Frontend: Drag-drop folder/zip + file dialog integration — 1d — P0
- [x] **M0-10** CI scaffold: `cargo test` + `cargo clippy` + `pnpm test` — 0.5d — P0

**Exit:** `pnpm tauri dev` jalan, drag-drop folder → grid thumbnails <5s untuk 50 halaman.

---

## M1 — Detect & Edit (Minggu 2-3) — P0

- [x] **M1-1** Port `image-ops` crate dari `kuron_native/rust` (build_mosaic, compress_page, chunk_webtoon, IMAGE_OPS_LOCK) — 2d — P0
- [x] **M1-2** Unit test `image-ops` (mosaic size, cap 2MB/1MB, downscale loop, compress 1280px) — 1d — P0
- [x] **M1-3** Bundle `bubble.onnx` ke `src-tauri/resources/models/bubble.onnx` + `ort` integration (session load, preprocess, infer) — 3d — P0
- [x] **M1-4** Implement `onnx-detect` crate: `BubbleDetector::new` + `detect` (spawn_blocking, intra_threads=2) — 2d — P0
- [x] **M1-5** Implement `post_process` (confidence >=0.25, giant-box filter 2.5x, optional NMS) — 1d — P0
- [x] **M1-6** Command `detect_bubbles` (single page, return Vec<BubbleBox>) — 1d — P0
- [x] **M1-7** Command `detect_bubbles_batch` (Vec<page_ids>, emit `detect_progress` events) — 1d — P0
- [x] **M1-8** Frontend: Canvas editor base (Konva, image fitWidth, coordinate mapping original px ↔ screen px) — 2d — P0
- [x] **M1-9** Frontend: Tools Rect/Ellipse/Freeform/Tail (draw, create BubbleBox) — 2d — P0
- [x] **M1-10** Frontend: Bubble interaction (drag/resize/delete, select, chip numbering RTL/LTR) — 2d — P0
- [x] **M1-11** Shape polygon persistence (BubbleBox.shape, tail) + overlay render (polygon clip vs rounded-rect fallback) — 1d — P0
- [x] **M1-12** Integration test: import 1 halaman → Detect → 5 bubbles di canvas → drag/resize/delete — 1d — P0

**Exit:** Import 1 halaman → Detect <2s (1080p) → bubbles di canvas → bisa edit.

---

## M2 — Translate Single (Minggu 4-5) — P0

- [x] **M2-1** Port `prompt` crate (buildMosaicPrompt, fullImagePrompt, sfxRule, appendGlossary) — reuse Kuron 1:1 — 2d — P0
- [x] **M2-2** Snapshot test `prompt` (compare dengan Kuron Dart prompt output) — 0.5d — P0
- [x] **M2-3** Port `parser` crate (ModelJsonParser: strip markdown, parse JSON, preview 200 chars) — 1d — P0
- [x] **M2-4** Unit test `parser` (JSON valid, markdown wrapped, invalid, empty) — 0.5d — P0
- [x] **M2-5** Implement `provider` trait `AiTranslationProvider` + `AiProviderFactory` — 1d — P0
- [x] **M2-6** Implement `OpenAICompatibleProvider` (POST /chat/completions, base64 mosaic, temp 0.3, timeout 90s, 429 handling) — 2d — P0
- [x] **M2-7** Implement `GeminiProvider` (Google REST, native shape) — 1.5d — P0
- [x] **M2-8** Implement `CohereProvider` (POST /v2/chat, native format) — 1.5d — P0
- [x] **M2-9** Implement `AiProviderConfig` + `AiProviderType` (9 types, defaultBaseUrl, modelsUrl, needsKeyForListing, isVisionCapable) — 1d — P0
- [x] **M2-10** Command `list_models` (GET modelsUrl, parse AiModelOption) — 1d — P0
- [x] **M2-11** Command `validate_provider` (minimal test POST) — 0.5d — P0
- [x] **M2-12** Command `save_provider` / `get_providers` (redacted) / `delete_provider` (keyring integration) — 1.5d — P0
- [x] **M2-13** Command `translate_page` (mosaic vs fallback, POST LLM, mapMosaicResult + reattach shape + whitePatch, cache put) — 2d — P0
- [x] **M2-14** Implement `cache` (rusqlite, key hash(image+bubbles+lang+style), get/put/clear) — 1d — P0
- [x] **M2-15** Frontend: Provider settings UI (CRUD 9 types, LOV, validate, keychain status) — 3d — P0
- [x] **M2-16** Frontend: Translate panel (targetLang, style 7, skipSfx, mosaicQuality, readingDirection, glossary preview) — 2d — P0
- [x] **M2-17** Frontend: Overlay render (shape-aware, whitePatch, isUserEdited guard, per-bubble edit original/reading/translated/font) — 2d — P0
- [x] **M2-18** Integration test: translate_page dengan mock LLM (wiremock) → PageTranslation overlay — 1d — P0

**Exit:** Pilih provider → Translate <15s → overlay terjemahan → bisa edit per-bubble.

---

## M3 — Batch & Export (Minggu 6) — P1

- [x] **M3-1** Implement queue `tokio::mpsc` + semaphore 3 untuk `translate_batch` — 2d — P1
- [x] **M3-2** Command `translate_batch` (Vec<page_ids>, emit `translate_progress`, backoff 2s/4s/8s on 429) — 1d — P1
- [x] **M3-3** Command `retry_bubble` (single bubble retry) — 0.5d — P1
- [x] **M3-4** Frontend: Batch UI (progress bar, concurrency indicator, failed bubbles highlighted) — 2d — P1
- [x] **M3-5** Implement `glossary` crate (rusqlite, GlossaryEntry CRUD, `selectRelevantGlossaryEntries` limit 5 substring case-insensitive timestamp desc, `buildGlossaryBlock`, `glossaryContextFor` dengan fallback most-recent 5 saat bubbleTexts empty, true no-match → None) — 2d — P1
- [x] **M3-6** Glossary prompt injection (`appendGlossary` ke mosaic prompt, SAME single AI request — never extra call, null/empty → prompt unchanged, never throws) — 0.5d — P1
- [x] **M3-7** Frontend: Glossary panel (CRUD, import/export CSV, long-press bubble → Save to Glossary, glossary preview di translate panel) — 2d — P1
- [x] **M3-8** Command `export_project` JSON (PageTranslation per page via save dialog) — 1d — P1
- [x] **M3-9** Command `export_project` PNG overlay (burn overlay via image crate, whitePatch, shape-aware) — 2d — P1
- [x] **M3-10** Command `export_project` CBZ (zip PNGs, filename order) — 1d — P1
- [x] **M3-11** Frontend: Review grid (failed bubbles, retry, export buttons) — 1.5d — P1
- [x] **M3-12** Integration test: batch 10 halaman → progress → export JSON/PNG — 1d — P1

**Exit:** Batch 10 halaman <3m → progress bar → export JSON/PNG/CBZ.

---

## M4 — Polish (Minggu 7-8) — P1/P2

- [x] **M4-1** OS keychain (`keyring` crate, service `id.kuron.studio`, account `provider:{id}`) + sqlite fallback (no stronghold dep; sqlite legacy column is the fallback) — 2d — P1
- [x] **M4-2** Log redaction (base64 preview 200 chars, never log key; `secrets::redacted` + keychain never logged) — 0.5d — P1
- [x] **M4-3** MosaicQuality toggle (low 75%/1MB vs high 85%/2MB — done in M2/M3: backend `MosaicQuality` + frontend select) — 1d — P1
- [x] **M4-4** RTL/LTR reading direction + whitePatch heuristic (flat/wide box — done in M2/M3: `order_indices` rtl/ltr + `needs_white_patch` aspect>2.5|area>0.25 + header toggle + overlay) — 1d — P1
- [x] **M4-5** i18n shell-level (en/id/zh flat keys `src/lib/i18n.ts` + header toggle + vitest coverage; panel strings follow-up) — 2d — P1
- [x] **M4-6** Theming (light/dark toggle, `data-theme` + Tailwind custom variant, persisted localStorage) — 1d — P2
- [x] **M4-7** Tauri updater (plugin desktop-only + endpoint GitHub Releases + header ↻ button + signing docs; pubkey placeholder sampai `signer generate`) — 2d — P1
- [x] **M4-8** CI matrix (pnpm check + clippy --all-targets + tauri build ubuntu/win/macos) — 1d — P1
- [x] **M4-9** Bundle optimization (bubble.onnx 11.8MB <80MB → tetap bundled, documented) — 1d — P2
- [x] **M4-10** E2E test backend-chain (`tests/e2e_flow_mock.rs`: import 3 → detect mock → translate mock → export JSON/PNG/CBZ; Playwright webview deferred — no driver in CI) — 1.5d — P1
- [x] **M4-11** Docs polish (README quickstart/gates/flow/config/release; screenshots deferred to `docs/`) — 1d — P2

**Exit:** Build `.msi`/`.dmg`/`.deb`, updater jalan, i18n lengkap, CI green.

---

## M5 — Post-MVP Backlog — P2/P3

- [x] **M5-1** TM Search (translation memory, search previous translations) — 1w — P2
- [x] **M5-2** QA Checks (untranslated, overflow, SFX leak) — 1w — P2
- [x] **M5-3** PSD export (layer per bubble) — 1w — P2
- [ ] **M5-4** Auto-typeset (burned-in text rendering) — 2w — P3
- [ ] **M5-5** Plugin prompt per genre/provider — 1w — P3
- [x] **M5-6** Kolaborasi (zip share via share_project; git = push repo biasa) — 1w — P3

---

## Dependencies

```
M0 -> M1 -> M2 -> M3 -> M4 -> M5
M1-1 (image-ops) -.-> M1-6
M1-3 (ONNX) -.-> M1-6
M2-1 (prompt) -.-> M2-13
M2-5 (provider) -.-> M2-13
M3-1 (queue) -.-> M3-2
```

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
- [ ] Batch size — 10/50/200 halaman per project
