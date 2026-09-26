# Flow — Kuron Studio

## 1. Arsitektur High-Level

```mermaid
flowchart TB
    subgraph FE [Frontend - Vite + Svelte/React]
        UI[Project / Page Grid]
        Canvas[Canvas Editor - Konva]
        Settings[Provider & Style Settings]
        Review[Review & Glossary Panel]
    end
    subgraph Core [Tauri v2 Core - Rust]
        Cmd[Commands - tauri::command]
        State[AppState - Mutex<ProjectStore>]
        ImgOps[image-ops crate]
        Onnx[onnx crate - ort/tract]
        Provider[provider crate]
        Cache[cache - rusqlite]
        Glossary[glossary - rusqlite]
    end
    subgraph Ext [External]
        LLM[BYOK LLM APIs]
        FS[Local FS - images/export]
        Keychain[OS Keychain]
    end
    UI --> Cmd
    Canvas --> Cmd
    Settings --> Cmd
    Review --> Cmd
    Cmd --> State
    Cmd --> ImgOps
    Cmd --> Onnx
    Cmd --> Provider
    Provider --> LLM
    Cmd --> Cache
    Cmd --> Glossary
    Cmd --> FS
    Cmd --> Keychain
```

## 2. Pipeline Kuron (As-Is) → Studio (To-Be)

**Kuron App (Flutter):**
```
capturePage(bytes,w,h) -> prefetchDetection (ONNX bg) -> translatePage:
  1. WebtoonDetector (long-strip? -> chunk)
  2. detectBubbles (ONNX via kuron_native) -> BubbleBox
  3. postProcessBoxes (merge, drop SFX)
  4. if empty -> FallbackImageHandler.compressPage (1280px, JPEG85) + fullImagePrompt (% coords)
     else -> MosaicBuilder.buildMosaic (crop 20% pad, 2x, label merah, cap 2MB/1MB)
  5. AiProviderFactory -> OpenAI/Gemini/Cohere -> translatePage(...)
  6. ModelJsonParser + mapMosaicResult -> PageTranslation
  7. TranslationCacheRepository.put + overlay render
```

**Kuron Studio (Tauri Rust) — reuse 1:1:**
```
import_pages -> detect_bubbles (ONNX Rust) -> edit bubbles (canvas) -> translate_page (mosaic/compress -> LLM) -> post-edit -> export
```

## 3. Sequence: Translate Satu Halaman

```mermaid
sequenceDiagram
    participant U as User
    participant FE as Frontend
    participant CMD as Rust Command
    participant ONNX as ONNX Detect
    participant IMG as ImageOps
    participant LLM as BYOK LLM
    participant CACHE as Cache

    U->>FE: Drop image / Open project
    FE->>CMD: import_pages(paths)
    CMD->>CACHE: put page meta
    U->>FE: Click Detect
    FE->>CMD: detect_bubbles(page_id)
    CMD->>ONNX: infer(image_bytes)
    ONNX-->>CMD: Vec<BubbleBox>
    CMD->>FE: bubbles + overlay
    U->>FE: Edit boxes (draw/rect/ellipse/freeform) + set style/lang
    FE->>CMD: translate_page(page_id, bubbles, targetLang, style, glossary)
    CMD->>IMG: build_mosaic(crops 20% pad, 2x, labels) OR compress_page
    IMG-->>CMD: mosaic_jpeg
    CMD->>LLM: POST /chat/completions (base64 mosaic + prompt)
    LLM-->>CMD: JSON {1:{original,reading,translated}, ...}
    CMD->>CMD: mapMosaicResult + reattach shape + whitePatch
    CMD->>CACHE: put PageTranslation
    CMD->>FE: PageTranslation
    FE->>U: Render overlay (editable per-bubble)
```

## 4. State Machine: Page

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Detecting: detect_bubbles
    Detecting --> Detected: bubbles found
    Detecting --> NoBubbles: empty
    Detected --> Editing: user draw/edit
    Editing --> Detected: confirm
    Detected --> BuildingMosaic: translate
    NoBubbles --> Compressing: translate (fallback)
    BuildingMosaic --> Translating: POST LLM
    Compressing --> Translating: POST LLM
    Translating --> Translated: success
    Translating --> Failed: error/429
    Failed --> Translating: retry
    Translated --> Editing: post-edit
    Translated --> Cached: save
    Cached --> [*]
```

## 5. Data Flow: Mosaic

```mermaid
flowchart LR
    A[Original Page JPEG] --> B[Decode image]
    B --> C[For each BubbleBox: crop x±20% y±20%]
    C --> D[Resize 2x linear]
    D --> E[Stack vertical + gap 10 + label 56x40 merah]
    E --> F[Encode JPEG 85/75]
    F --> G{> cap? 2MB/1MB}
    G -- ya --> H[Downscale 0.75x loop until <=cap or w<=64]
    H --> F
    G -- tidak --> I[Mosaic JPEG + bubbles order]
```

**Detail Mosaic (port dari `MosaicBuilder`):**
- Crop: `padX = w*0.2`, `padY = h*0.2`, clamp ke image bounds
- Scale: `copyResize 2x linear`
- Layout: `totalWidth = max(chip.width)`, `totalHeight = sum(chip.height+gap)-gap+labelHeight`
- Fill: white `255,255,255`, label red `255,0,0` font `arial48`
- Cap: `high=2MB/85%`, `low=1MB/75%` — Rust fast path hanya untuk high, low pakai Dart encoder (di Studio: 100% Rust)

## 6. Data Flow: Fallback (No Bubble)

```mermaid
flowchart LR
    A[Original Page JPEG] --> B[Decode]
    B --> C{longest > 1280?}
    C -- ya --> D[Resize ratio=1280/longest linear]
    C -- tidak --> E[Keep original]
    D --> F[Encode JPEG 85]
    E --> F
    F --> G[POST LLM + fullImagePrompt % coords]
    G --> H[Map % -> px via original w/h]
```

## 7. Batch Flow

```mermaid
flowchart TB
    P[Project: 50 pages] --> Q[Queue - tokio mpsc + semaphore 3]
    Q --> R1[Page1: detect -> mosaic -> LLM]
    Q --> R2[Page2: ...]
    Q --> R3[Page3: ...]
    R1 --> S[Progress event -> FE]
    R2 --> S
    R3 --> S
    S --> T[Review grid - failed bubbles highlighted]
    T --> U[Export JSON/PNG overlay/CBZ]
```

**Concurrency:** semaphore 3 untuk hindari 429 & OOM. Progress via `tauri::emit("translate_progress", {page_id, done, total})`.

## 8. Provider Flow

```mermaid
flowchart TB
    A[User add provider] --> B{type needsKeyForListing?}
    B -- ya --> C[Input apiKey -> keyring]
    B -- tidak --> D[No key needed]
    C --> E[list_models -> GET modelsUrl]
    D --> E
    E --> F[Parse AiModelOption isVision]
    F --> G[User pick model -> save_provider]
    G --> H[validate -> POST minimal test]
    H --> I{success?}
    I -- ya --> J[Saved, isVisionCapable]
    I -- tidak --> K[Error + retry]
```

**9 Provider Types:**
`zen` (opencode.ai/zen/v1), `openCodeGo` (zen/go/v1), `gemini` (Google REST), `openAi`, `openRouter`, `metaAi`, `clinePass`, `cohere` (/v2/chat), `custom` (manual baseUrl)

## 9. Glossary Flow

> **Bug sebelumnya:** label mermaid pakai `"` dan `->` di dalam node (`Prompt += 'Glossary:\n\"a\" -> \"b\"'` dan `Save new terms? -> glossary_save`) bikin parser mermaid error + flow kelewat linear (tidak tunjukkan seleksi limit 5, fallback, dan save via long-press). Sudah diperbaiki di bawah — sekarang mirror logic Kuron `selectRelevantGlossaryEntries` + `buildGlossaryBlock` + `_glossaryContextFor`.

```mermaid
flowchart TB
    A[Glossary DB - rusqlite] --> B[User CRUD - list save delete]
    B --> C{Translate triggered?}
    C -- No --> B
    C -- Yes --> D[Load all entries - getAll]
    D --> E[selectRelevant - substring match case-insensitive - sort timestamp desc - limit 5]
    E --> F{Relevant found?}
    F -- Yes --> G[buildGlossaryBlock - Glossary lines]
    F -- No --> H{bubbleTexts empty?}
    H -- Yes --> I[Fallback - most recent 5]
    H -- No --> J[No block - prompt unchanged]
    I --> G
    G --> K[appendGlossary - inject into mosaic prompt - same single AI request]
    J --> K
    K --> L[POST LLM with enriched prompt]
    L --> M[Receive PageTranslation]
    M --> N{User long-press bubble?}
    N -- Yes --> O[Save to Glossary - glossary_save]
    N -- No --> P[Done]
    O --> P
```

```mermaid
sequenceDiagram
    participant FE as Frontend
    participant CMD as Rust Command
    participant DB as Glossary DB
    participant LLM as BYOK LLM

    FE->>CMD: translate_page with bubbleTexts
    CMD->>DB: getAll entries
    DB-->>CMD: entries
    CMD->>CMD: selectRelevantGlossaryEntries - limit 5
    CMD->>CMD: buildGlossaryBlock
    CMD->>CMD: appendGlossary to prompt
    CMD->>LLM: POST with enriched prompt - single request
    LLM-->>CMD: translation JSON
    CMD-->>FE: PageTranslation
    FE->>FE: User long-press bubble
    FE->>CMD: glossary_save
    CMD->>DB: persist entry
```

**Logic detail (port 1:1 dari Kuron `reader_translation_cubit.dart`):**
- `glossaryMaxEntries = 5` — max entries per prompt
- `selectRelevantGlossaryEntries(entries, bubbleTexts)` — `sourceText` substring case-insensitive di `bubbleTexts`, sort `timestamp` desc, take 5. Empty `bubbleTexts` atau no match dengan known texts → fallback most-recent 5 (fresh page). Empty glossary atau true no-match → `null` (prompt unchanged).
- `buildGlossaryBlock(entries)` → `Glossary:\n"src" -> "dst"\n...` — embedded di **satu** AI request yang sama, **tidak pernah** extra call.
- `appendGlossary(prompt, glossaryContext)` — null/empty = prompt unchanged.
- `_glossaryContextFor(bubbleTexts)` — load via `GlossaryRepository.getAll()` (Kuron: `SharedPreferences` JSON list; Studio: `rusqlite` `glossary.db`), never throws.
- Save: **bukan** auto setelah translate — user **long-press bubble → Save to Glossary** (`SaveGlossaryEntryUsecase` → `GlossaryEntry{id, sourceText, translatedText, reading, contentId, pageIndex, timestamp}`), lalu tersedia untuk translate berikutnya.

## 10. Export Flow

```mermaid
flowchart TB
    A[Project Translated] --> B{Export format}
    B -- JSON --> C[PageTranslation JSON per page]
    B -- PNG Overlay --> D[Burn overlay to PNG - image crate]
    B -- CBZ --> E[Zip PNGs -> .cbz]
    C --> F[Save to FS via dialog]
    D --> F
    E --> F
```

## 11. Error & Retry Flow

```mermaid
flowchart TB
    A[translate_page] --> B{LLM response}
    B -- 429 --> C[Rate limited -> backoff 2s,4s,8s]
    B -- JSON invalid --> D[ModelJsonParser retry 1x]
    B -- empty bubbles --> E[Fallback to fullImage]
    B -- success --> F[Cache + overlay]
    C --> G{retry < 3?}
    G -- ya --> A
    G -- tidak --> H[Mark failed_bubbles]
    D --> I{retry success?}
    I -- ya --> F
    I -- tidak --> H
```

## 12. File Structure Flow

```
kuron-studio/
├── src/                 # frontend
│   ├── routes/
│   │   ├── +layout.svelte
│   │   ├── project/[id]/+page.svelte  # grid + batch
│   │   ├── editor/[pageId]/+page.svelte # canvas
│   │   └── settings/+page.svelte      # providers
│   ├── lib/canvas/      # Konva overlay, shape editor
│   └── lib/stores/      # project, provider, glossary
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json  # bundleId id.kuron.studio
    ├── capabilities/default.json
    ├── resources/models/bubble.onnx
    └── src/
        ├── lib.rs       # setup, AppState
        ├── commands/    # project, image, detect, translate, glossary, provider
        └── crates/      # image-ops, onnx-detect, provider, prompt, parser
```
