# Kuron Studio — Manga Translation Workbench

> **Tagline:** Desktop workbench untuk provider translate manga/manhwa.  
> **Stack:** Tauri v2 (Rust) + Vite Frontend + ONNX + BYOK LLM  
> **Bundle ID:** `id.kuron.studio`  
> **Status:** Ideation / Pre-MVP — catatan SDD & OpenSpec siap

## Apa itu Kuron Studio?

Kuron Studio adalah app desktop terpisah dari Kuron App (Flutter) yang membantu **provider / tim translate** mengerjakan manga/manhwa secara batch:

- Import puluhan halaman → deteksi bubble otomatis (ONNX on-device)
- Edit bubble di canvas (rect/ellipse/freeform + tail)
- Translate via BYOK vision LLM (OpenAI-compatible, Gemini, Cohere)
- Post-edit per-bubble + glossary + export

**Bukan:** reader, scraper, atau hosting model. Fokus ke **tooling provider**.

## Struktur Folder

```
Studio/
├── README.md              # overview ini
├── MEMORY.md              # project memory (konteks lintas sesi)
├── docs/
│   ├── ide.md             # ide & problem/solution
│   ├── target.md          # target MVP & success criteria
│   ├── roadmap.md         # phased roadmap 0-5
│   ├── tech-stack.md      # stack & crate pilihan
│   └── flow.md            # flow + mermaid diagrams
└── openspec/
    ├── config.yaml        # OpenSpec config
    ├── specs/kuron-studio/spec.md
    └── changes/kuron-studio-mvp/
        ├── proposal.md
        ├── design.md
        └── tasks.md
```

## Keputusan — RESOLVED 2026-09-08

| # | Pertanyaan | Jawaban |
|---|------------|---------|
| 1 | Scope | Public — provider + umum |
| 2 | Frontend | **Svelte + Rust** (benci React) |
| 3 | ONNX | **`ort` untuk MVP** — <2s, bundle 50MB OK; `tract` opsi lightweight M4 |
| 4 | Export | **JSON (P0) + PNG overlay + CBZ (P1)**, PSD deferred |
| 5 | Glossary | **DB local** — `rusqlite` `glossary.db` |
| 6 | Batch | **3 images per translate** (hemat AI), queue 50 halaman, semaphore 3 |
| 7 | Lisensi | **Boleh share** — prompt/model Kuron boleh eksternal |

## Quick Start (nanti)

```bash
# scaffold Tauri (Svelte + Rust — resolved 2026-09-08)
pnpm create tauri-app@latest kuron-studio --template svelte
cd kuron-studio
pnpm install
pnpm tauri dev
```

## Hubungan dengan Kuron App

- Reuse prompt & mapping Kuron 1:1 (`openai_compatible_provider.dart` → Rust `prompt` crate)
- Reuse ONNX model bubble detection (`kuron_native/rust`)
- Reuse `MosaicBuilder` / `FallbackImageHandler` logic (port ke Rust 100%)
- Glossary & cache format kompatibel (JSON) untuk migrasi

## Next Step

1. Baca `docs/ide.md` → `docs/target.md` → `docs/flow.md`
2. Review `openspec/changes/kuron-studio-mvp/proposal.md`
3. Jawab 7 pertanyaan terbuka di proposal, lalu `pnpm create tauri-app`
