# Spec: Kuron Studio — Manga Translation Workbench

## Context

Kuron Studio adalah app desktop Tauri v2 (Rust) terpisah dari Kuron App (Flutter) untuk membantu provider translate manga/manhwa secara batch. Reuse pipeline Kuron: ONNX bubble detection → mosaic/fallback → BYOK LLM → overlay. Target: 50 halaman mentah ke terjemahan siap export dalam < 30 menit. Bundle ID `id.kuron.studio`. Desktop (Win/macOS/Linux) stabil, mobile deferred.

## Purpose

Workbench produksi untuk provider: import batch, deteksi bubble on-device, edit bubble di canvas, translate via BYOK vision LLM (9 provider), post-edit per-bubble, glossary konsisten, export JSON/PNG/CBZ. Bukan reader, bukan scraper, bukan hosting model.

---

### Requirement: Project management SHALL support batch import

System SHALL allow creating a project and importing a folder/zip of images (JPG/PNG/WebP) as pages, with thumbnail generation and grid display.

#### Scenario: Create project and import folder
- **WHEN** user creates a project and selects a folder with 50 images
- **THEN** system SHALL create `Project{id, name, pages: Page[]}` and generate 512px thumbnails async
- **AND** display grid with status badges (idle/detecting/translated/failed) within 5s

#### Scenario: Import zip
- **WHEN** user imports a `.zip`/`.cbz` containing images
- **THEN** system SHALL extract and import as pages in filename order

#### Scenario: Invalid file skipped
- **WHEN** folder contains non-image files
- **THEN** system SHALL skip them and report count

---

### Requirement: Bubble detection SHALL run on-device via ONNX

System SHALL use Rust ONNX runtime (`ort` or `tract`) to detect `BubbleBox{x,y,w,h,confidence,shape,kind,tail}` from page image bytes, without network.

#### Scenario: Detect single page
- **WHEN** user triggers `detect_bubbles(page_id)` on a 1080p page
- **THEN** system SHALL return `Vec<BubbleBox>` within 2s, filtered by confidence >= 0.25
- **AND** remove giant false-positive boxes engulfing 2.5x smaller boxes

#### Scenario: Detect batch
- **WHEN** user triggers `detect_bubbles_batch(page_ids)` for 10 pages
- **THEN** system SHALL emit `detect_progress` events and complete within 20s

#### Scenario: Zero bubbles
- **WHEN** ONNX returns empty
- **THEN** system SHALL mark page as `NoBubbles` and allow fallback translate path

---

### Requirement: Canvas editor SHALL support bubble editing

System SHALL provide a canvas (Konva) overlaying the page image with editable bubble shapes, supporting rect/ellipse/freeform/tail tools, drag/resize/delete, and RTL/LTR chip numbering.

#### Scenario: Draw new bubble
- **WHEN** user selects Rect tool and drags on canvas
- **THEN** system SHALL create a `BubbleBox` in original px coords and add to page

#### Scenario: Edit existing bubble
- **WHEN** user drags or resizes a bubble overlay
- **THEN** system SHALL update its `x,y,w,h` (and `shape` polygon if freeform) in original px

#### Scenario: Delete bubble
- **WHEN** user selects a bubble and presses Delete
- **THEN** system SHALL remove it from page bubbles

#### Scenario: RTL/LTR numbering
- **WHEN** reading direction is RTL (manga) vs LTR (manhwa)
- **THEN** chip numbers SHALL order right-to-left vs left-to-right, top-to-bottom

---

### Requirement: Translation SHALL use mosaic when bubbles exist, fallback when empty

System SHALL build a mosaic (crop 20% pad, 2x scale, vertical stack + red labels, JPEG 85/75 cap 2MB/1MB with downscale loop) when bubbles exist; otherwise compress full page (1280px longest, JPEG85) and request % coords. Reuse Kuron prompt 1:1.

#### Scenario: Mosaic path
- **WHEN** page has >=1 bubble and user triggers `translate_page`
- **THEN** system SHALL build mosaic via `image-ops::build_mosaic` and POST to LLM with `buildMosaicPrompt(targetLang, style, skipSfx, readingDirection, glossaryContext)`
- **AND** map JSON `{1:{original,reading,translated}, ...}` to `BubbleTranslation` per BubbleBox, skipping "SKIP"

#### Scenario: Mosaic cap exceeded
- **WHEN** mosaic JPEG > cap (2MB high / 1MB low)
- **THEN** system SHALL downscale 0.75x loop until <=cap or width <=64

#### Scenario: Fallback path
- **WHEN** page has 0 bubbles
- **THEN** system SHALL compress via `image-ops::compress_page` and POST with `fullImagePrompt`, converting % coords to px via original w/h

#### Scenario: SFX handling
- **WHEN** `skipSfx=true` and bubble is SFX-only (ドドド, バキ, etc.)
- **THEN** LLM SHALL return "SKIP" and system SHALL exclude it from overlay

---

### Requirement: Provider abstraction SHALL support 9 BYOK types

System SHALL support `AiProviderType{zen, openCodeGo, gemini, openAi, openRouter, metaAi, clinePass, cohere, custom}` with `AiProviderConfig`, live model LOV, validation, and OS keychain storage. Reuse Kuron `defaultBaseUrl`, `modelsUrl`, `needsKeyForListing`, `isVisionCapable`.

#### Scenario: Add provider
- **WHEN** user adds a provider (type, displayName, apiKey, model)
- **THEN** system SHALL store apiKey in OS keychain (service `id.kuron.studio`, account `provider:{id}`) and config in `store.json` without key
- **AND** `get_providers` SHALL return `apiKey: "***"` redacted

#### Scenario: List models
- **WHEN** user requests `list_models(type, apiKey?)`
- **THEN** system SHALL GET `modelsUrl` and return `Vec<AiModelOption{id, label, isVision}>` (custom always manual entry)

#### Scenario: Validate provider
- **WHEN** user triggers `validate_provider(config)`
- **THEN** system SHALL POST minimal test request and return success or `AiTranslationException` (including 429 rate limited)

#### Scenario: 429 handling
- **WHEN** LLM returns 429
- **THEN** system SHALL surface `isRateLimited=true` and allow retry with backoff

---

### Requirement: Translation styles SHALL include 7 variants

System SHALL support `TranslationStyle{natural, genz, action, romantis, formal, kasar, literal}` with prompt-injected `instruction` per style, matching Kuron.

#### Scenario: Style injected to prompt
- **WHEN** user selects `genz` and translates
- **THEN** prompt SHALL contain "Use informal Indonesian slang: gue/lo, sih/dong/nih/deh/doang. For comedy/light romance."

---

### Requirement: Glossary SHALL support CRUD and prompt injection with relevance selection

System SHALL store `GlossaryEntry{id, sourceText, translatedText, reading, contentId, pageIndex, timestamp}` in `glossary.db` (rusqlite), select up to 5 relevant entries per translate via substring match, and inject via `appendGlossary(prompt, glossaryContext)` inside the SAME single AI request — never an extra call. Save is explicit via user long-press, not auto.

#### Scenario: Save glossary entry via long-press
- **WHEN** user long-presses a translated bubble and taps Save to Glossary
- **THEN** system SHALL persist `GlossaryEntry` (id `gl_{ms}_{rectHash}`, sourceText=original, translatedText, reading, contentId, pageIndex, timestamp seconds) to `glossary.db`
- **AND** make it available for future translates

#### Scenario: Relevant selection (limit 5, substring, timestamp desc)
- **WHEN** translating with known `bubbleTexts` and glossary has entries
- **THEN** system SHALL run `selectRelevantGlossaryEntries(entries, bubbleTexts, limit=5)` — `sourceText` case-insensitive substring of any `bubbleTexts`, sorted `timestamp` desc, take 5
- **AND** render via `buildGlossaryBlock` as `Glossary:\n"src" -> "dst"\n...`

#### Scenario: Fallback most-recent 5 on fresh page
- **WHEN** `bubbleTexts` is empty (fresh page, no known texts yet) and glossary has entries but no relevant match
- **THEN** system SHALL fallback to most-recent 5 entries (sorted `timestamp` desc) as relevance proxy — model ignores irrelevant lines

#### Scenario: True no-match with known texts
- **WHEN** `bubbleTexts` is non-empty but no entry matches (true no-match)
- **THEN** system SHALL return null — prompt unchanged, no glossary block

#### Scenario: Glossary injected to prompt (same request)
- **WHEN** relevant entries exist and `buildGlossaryBlock` returns `Glossary:\n"a" -> "b"`
- **THEN** system SHALL `appendGlossary(prompt, glossaryContext)` and POST to LLM with enriched prompt in the SAME single request
- **AND** LLM SHALL use it for consistency

#### Scenario: Glossary empty or null
- **WHEN** glossary is empty or `glossaryContext` is null/empty
- **THEN** prompt SHALL remain unchanged and no extra AI request SHALL be made
- **AND** `_glossaryContextFor` SHALL never throw

---

### Requirement: Cache SHALL key by content hash

System SHALL cache `PageTranslation` keyed by hash of `image_bytes + bubbles + targetLang + style` in `cache.db` (rusqlite), with clear support.

#### Scenario: Cache hit
- **WHEN** translating same image+bubbles+lang+style again
- **THEN** system SHALL return cached `PageTranslation` without LLM call

#### Scenario: Cache clear
- **WHEN** user triggers `cache_clear`
- **THEN** system SHALL delete all cached entries

---

### Requirement: Batch translate SHALL queue with concurrency control

System SHALL queue batch translate via `tokio::mpsc` + semaphore 3, emitting `translate_progress` events, with per-bubble retry.

#### Scenario: Batch 10 pages
- **WHEN** user triggers `translate_batch(page_ids)` for 10 pages
- **THEN** system SHALL process with concurrency 3, emit progress per page, and complete within 3 minutes

#### Scenario: Retry failed bubble
- **WHEN** a bubble fails (JSON invalid, empty)
- **THEN** user SHALL be able to `retry_bubble(page_id, bubble_idx)` for that single bubble

#### Scenario: Rate limited in batch
- **WHEN** a page hits 429 during batch
- **THEN** system SHALL backoff (2s,4s,8s) and emit `provider_rate_limited` event

---

### Requirement: Export SHALL support JSON, PNG overlay, CBZ

System SHALL export project translations as JSON (PageTranslation per page), PNG with burned-in overlay, and CBZ (zip of PNGs).

#### Scenario: Export JSON
- **WHEN** user exports as JSON
- **THEN** system SHALL write `{pages: [{pageId, bubbles: BubbleTranslation[]}]}` via save dialog

#### Scenario: Export PNG overlay
- **WHEN** user exports as PNG overlay
- **THEN** system SHALL burn translated text into image (white patch if `needsWhitePatch`, shape-aware) via `image` crate

#### Scenario: Export CBZ
- **WHEN** user exports as CBZ
- **THEN** system SHALL zip all PNG overlays into `.cbz` in filename order

---

### Requirement: Security SHALL use OS keychain and ACL

System SHALL store API keys only in OS keychain (`keyring`), redact in `get_providers`, enforce Tauri ACL (`capabilities/default.json`) whitelisting `fs:read` to project dir and `http:fetch` to 9 provider `defaultBaseUrl`, and redact base64/key in logs.

#### Scenario: Key not in plaintext
- **WHEN** inspecting `store.json` or `get_providers` response
- **THEN** apiKey SHALL be `"***"` or absent, never plaintext

#### Scenario: ACL enforced
- **WHEN** frontend attempts `http:fetch` to non-whitelisted domain
- **THEN** Tauri SHALL deny the request

---

### Requirement: Overlay SHALL render shape-aware with whitePatch

System SHALL render translated text in bubble regions using `shape` polygon when available (fallback rounded-rect), with `needsWhitePatch` heuristic for flat/wide boxes on busy artwork, and `isUserEdited` guard preventing AI overwrite.

#### Scenario: Shape polygon render
- **WHEN** BubbleBox has `shape` polygon
- **THEN** overlay SHALL clip text to polygon outline

#### Scenario: White patch
- **WHEN** `needsWhitePatch=true` (flat/wide box)
- **THEN** overlay SHALL render white patch behind text

#### Scenario: User edited guard
- **WHEN** `BubbleTranslation.isUserEdited=true`
- **THEN** subsequent translate SHALL NOT overwrite its `translated` text
