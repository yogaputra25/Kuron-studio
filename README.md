# Kuron Studio — Manga Translation Workbench

> **Tagline:** Desktop workbench untuk provider translate manga/manhwa.  
> **Stack:** Tauri v2 (Rust) + Vite Frontend + ONNX + BYOK LLM  
> **Bundle ID:** `id.kuron.studio`  
> **Status:** MVP selesai (M0–M4 + sebagian besar M5)

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

## Install, Build & Run

Prasyarat: [Node.js 22](https://nodejs.org/), [pnpm](https://pnpm.io/installation),
[Rust stable](https://rustup.rs/) (+ Linux: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`).

```bash
cd kuron-studio
pnpm install

pnpm tauri dev      # dev run (jangan `pnpm dev` — backend tidak ikut jalan)
pnpm tauri build    # installer .msi / .dmg / .deb
```

Gates (wajib hijau semua, sama seperti CI):

```bash
cd kuron-studio/src-tauri && cargo test && cargo clippy --all-targets -- -D warnings
cd kuron-studio && pnpm test && pnpm check
```

Satu test: `cargo test <substring>` (mis. `cargo test --test translate_mock retry`), `pnpm vitest run src/lib/batch.test.ts`.

> Catatan Windows: bila `cargo` tidak ditemukan, awali dengan
> `export PATH="$HOME/.cargo/bin:$PATH"` (git-bash).

## Hubungan dengan Kuron App

- Reuse prompt & mapping Kuron 1:1 (`openai_compatible_provider.dart` → Rust `prompt` crate)
- Reuse ONNX model bubble detection (`kuron_native/rust`)
- Reuse `MosaicBuilder` / `FallbackImageHandler` logic (port ke Rust 100%)
- Glossary & cache format kompatibel (JSON) untuk migrasi

## Next Step

1. Baca detail app di `kuron-studio/README.md`
2. Roadmap & status task: `openspec/changes/kuron-studio-mvp/tasks.md`
   (sisa terbuka: **M5-5** plugin prompt per genre/provider)
