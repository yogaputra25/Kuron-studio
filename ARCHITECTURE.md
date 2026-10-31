# ARCHITECTURE.md — Kuron Studio

Peta sistem. Baca ini sebelum mengubah kode. Pasangan: `DESIGN.md`
(aturan UI), `kuron-studio/README.md` (cara jalanin), dan
`design/references-v1/` (bukti visual v1).

## 1. Stack

- **Tauri v2 (Rust)** backend + **Svelte 5** frontend. Semua di
  `kuron-studio/` (`src/` UI, `src-tauri/` logika).
- Semua proses di perangkat sendiri. Model ONNX di-bundle
  (`resources/models/bubble.onnx`). Kunci API di keychain OS
  (`id.kuron.studio`), fallback sqlite. Tidak ada server.
- Bundle: `id.kuron.studio`, Windows/macOS/Linux (+ iOS sebagian).

## 2. Alur runtime

```
Import folder/files/zip → Detect bubble (ONNX, per halaman)
→ edit bubble di canvas → Providers (simpan key, validate)
→ Translate (<15s, overlay + edit per bubble)
→ Batch (3 paralel, 1 gagal ≠ semua gagal) → Review + QA
→ Ekspor JSON / PNG overlay / CBZ / PSD (+ share ZIP)
```

## 3. Frontend (`kuron-studio/src/`)

`src/lib/` hanya berisi 9 folder domain (tanpa file loose). Aturan import:
`app/ → domain/ → ui/ + contracts/`; pengecualian tunggal `editor/ → editor/text/`.

| Folder | Isi |
|---|---|
| `contracts/` | `types.ts` (impor `*.json` — sumber kebenaran kontrak lintas bahasa; dijaga `image-ext-contract.test.ts` + `provider-preset-contract.test.ts`), `api.ts` (satu-satunya jembatan invoke; key top-level `snake_case`, field `camelCase`; dijaga `invoke-contract.test.ts`), `log.ts` + 5 contract test |
| `app/` | `i18n.ts` (id/en/zh; `i18n.test.ts` gagal bila key hilang), `device.ts` (deteksi mobile; dijaga `mobile-gating.test.ts`), `Onboarding/DiagnosticsPanel/UpdateButton.svelte` + 5 test shell |
| `ui/` | 6 primitif (`Button/Field/Lov/Panel/Snackbar/StatusBadge`) + `__fixtures__/` + 7 test |
| `library/` | `library.ts` (agregat murni), `status.ts`, `PageGrid/PageList/FolderChip/MoveDialog.svelte` + 6 test |
| `editor/` | `EditorPanel/CanvasEditor.svelte`, `bubble.ts`, `translation.ts`, `panel.ts` + 13 test; `editor/text/` = `textLayout/textMeasure.ts` + test |
| `detection/` | `detectProgress.ts` + `DetectProgress.svelte` + 2 test |
| `provider/` | `batch.ts`, `BatchPanel/ProviderSettings/TranslatePanel/GlossaryPanel.svelte` + 4 test |
| `export/` | `ReviewGrid.svelte` (progres, QA, TM, ekspor) |
| `previews/` | `previewQueue.ts`, `previewMetrics.ts` + test |

- `App.svelte`: state root — `projects`, `currentId`, `folderId`,
  `selectedIds` (multi-select), `thumbs`, panel mana yang terbuka.
  Dashboard + breadcrumb + grid + header + panel di sini.
- `app.css`: token `--ks-*` (netral warm, aksen coral, sekunder cyan),
  dark default + light. Jangan tambah nilai di luar token.

## 4. Backend (`kuron-studio/src-tauri/src/`)

- `lib.rs` `AppState`: `ProjectStore` + detector + reqwest client +
  sqlite (cache+glossary) + secrets. `data_dir` = folder data aplikasi
  OS (**data + key tidak pernah di repo**).
- `commands/`:
  - `project`: `list/create/get/delete_project`, `import_pages`
    (JPG/PNG/WebP, urut nama file; ZIP/CBZ/PDF diekstrak; sidecar
    `._*`/0-byte/korup = skipped, bukan halaman rusak),
    `clean_pages` (+ scope `folder_id` = subtree), preview 512px (`image.rs`).
  - `folders`: `create_folder` (opsional `parent_id`), `rename_folder`
    (unik per saudara), `delete_folder` (rekursif subtree),
    `move_pages` (`{moved, skipped}`, tolak no-op penuh).
    Guard siklus di `set_parent`; agregat/export selalu subtree.
  - `detect`: satu/Batch, status `detecting → detected/noBubbles`.
    `save_bubbles` tak mendemosikan halaman berterjemahan
    (page-status-truth). `bubble.rs::sanitize_bubbles`: buang nol-area,
    cap 500/page.
  - `translate`: 5 provider BYOK (Gemini, OpenAI, Meta AI, Cohere,
    Custom), validate, `<15s`, status
    `translating/translated/failed`, retry per page/bubble.
    `translated` hanya bila semua bubble terisi ATAU flag manual
    (`mark_page_completed`, mis. SFX kosong); sebagian = status lama.
    Load menormalisasi status gantung/overstated (page-status-truth).
    Preset hapus (OpenRouter/Zen/OpenCodeGo/ClinePass) terbaca sebagai
    Custom; label vision dari allowlist per provider (#26/#27).
    Model per-run: override opsional di tiap request translate
    (default = simpanan provider), tercatat di `translation.model`.
    Alur provider: `diagrams/provider-list-models-draft.mmd` dan
    `diagrams/provider-validate.mmd`.
  - `export`: JSON / PNG (overlay, typeset TTF sistem + fallback
    bitmap) / CBZ / PSD (folder = dir, file = path).
  - `extras`: `tm_search` lintas project, `qa_check`,
    `share_project` (zip slice projects.json + gambar), diagnostics/log.
  - `glossary`: CRUD + CSV import/export, dipakai otomatis + manual.
- **Layer backend & arah dependensi (BE-layer, lane #51–#57):**

  ```
  domain ← store · db (+repo) · render · provider · jobs · prompt/translation
    ← commands (boundary tipis; termasuk commands/import/)
  error dipakai semua layer (internal); AppState + lib.rs merakit semuanya.
  ```

  | Dari → ke | Aturan |
  |---|---|
  | `commands` → `domain`/`store`/`db`/`render`/`provider`/`jobs` | ✅ boundary: parsing arg, panggil layer, kembalikan `Result<T, String>` |
  | `store`/`db`/`repo`/`render`/`provider` → `domain`, `error` | ✅ ambil tipe + `AppError`; tidak memanggil kembali ke `commands` (rujukan `crate::commands` yang ada hanya di `#[cfg(test)]`/doc) |
  | `domain` → `store`/`db`/`commands`/I/O | ❌ `domain` hanya tipe data — tanpa I/O, Tauri, HTTP, SQLite; satu-satunya rujukan keluar adalah tipe daun (`bubble`, `translation`) + `error` |
  | `#[tauri::command]` mengembalikan selain `Result<T, String>` | ❌ konversi `AppError → String` hanya di titik command via `?`/`From` |

  Modul: `domain/` (`page`, `project`, `status`, `tree` — dipindah verbatim
  dari `commands/project.rs`); `store/` (`ProjectStore` + `atomic.rs`: tulis
  `*.tmp` + `sync` + `rename`, putar `*.bak`, pulih dari `.bak` saat main
  korup); `db/` (`migrate` + `SCHEMA_SQL` 4 tabel; repo per-domain tinggal di
  modul pemilik: `provider/repo.rs`, `glossary/repo.rs`,
  `preview_cache/repo.rs`); `render/` (`image_ops`, `detector`,
  `detect_decode`, `outline`, `psd`, `page` teks); `jobs.rs` (`JobRegistry`:
  dedup + semaphore 3 + cancel milik `AppState`); `error/` (`AppError`
  thiserror); `provider/` (dispatch `match` + `http.rs` retry-429);
  `commands/import/` (pakai `crate::domain::` langsung).
- **Batas kontrak FE↔BE** (dijaga `invoke-contract.test.ts`): dua lapis.
  Lapis 1 — key top-level invoke `snake_case` persis nama param Rust
  (`project_id`, `page_ids`). Lapis 2 — field struct `camelCase` via serde
  `rename_all` (`pageId`, `targetLang`). Konstanta lintas bahasa hanya di
  `src/lib/contracts/*.json` (`extensions`, `layout`, `providers`): Rust via
  `include_str!` + `OnceLock`, TS via `import` langsung; test kontrak
  membandingkan pemakaian lawan JSON, bukan me-regex source lawan.
- `projects.json`: `Project { id, name, folders[], updatedAt }`,
  `ChapterFolder { id, name, pages[], createdAt, parentId? }`,
  `Page { id, path, w/h, status, bubbles[], translation? }`.
  1 project = 1 judul; folder = tree rekursif (`None` = root).
  File lama flat `{ pages[] }` / tanpa `parentId` dimigrasi otomatis
  ke 1 folder `default` root saat load (#7, nested-folders-page-move).

## 5. iOS (sebagian, bukan target penuh)

- `tauri.ios.conf.json` min 15.0; capability `mobile` tanpa
  plugin desktop (updater/process); `default` untuk desktop.
- Batasan nyata: tanpa folder picker (PNG/PSD folder hanya CBZ/JSON),
  dev server bind `0.0.0.0:1420` (`vite.config.ts`).
- Jalur sim: `pnpm ios:run:sim`; fisik butuh signing di Xcode.

## 6. Gate (wajib hijau = CI)

```bash
cd kuron-studio/src-tauri && cargo test && cargo clippy --all-targets -- -D warnings
cd .. && pnpm test && pnpm check
```

Satu test: `cargo test <nama>` / `pnpm vitest run <file>`.

## 7. Aturan taruh logika

- Frontend = tampilan + state sementara. Backend = file, model, AI,
  antrean. Jangan panggil fs/model dari Svelte langsung.
- Perubahan kontrak invoke → update `api.ts` + kedua sisi + test kontrak.
- Konstanta lintas bahasa hanya hidup di `kuron-studio/src/lib/contracts/*.json`
  (ekstensi import, konstanta layout + font, base URL provider): Rust membaca
  via `include_str!` + `OnceLock`, TS via `import` langsung; test kontrak di
  kedua sisi membandingkan pemakaian lawan JSON, tidak pernah me-regex source
  lawan — jadi file boleh pindah modul tanpa test ikut merah.
- String UI baru → `i18n.ts` 3 bahasa di commit yang sama.

## 8. Peta folder repo

```
Kuron-studio/
├── README.md / DESIGN.md / ARCHITECTURE.md  # dokumen induk
├── kuron-studio/    # kode (src + src-tauri)
├── design/          # .pen asli, .figcraft convert, mockup, references-v1/
└── .github/         # template issue + CI
```

Backlog aktif: #8 (panel resize + mode tablet). #7 selesai: folder
chapter + dashboard ringkasan + kanban + export per folder.

## 9. ADR & PR checklist

Keputusan arsitektur BE-layer tercatat di `adr/` (satu ADR per
keputusan): ADR-001 domain keluar dari commands · ADR-002 JSON + atomic
write · ADR-003 provider `match` bukan trait · ADR-004 opsi translate satu
perakit · ADR-005 test behavioral bukan source-pattern.

Checklist sebelum PR (semua wajib centang):

- [ ] Gate hijau (= CI): `cd kuron-studio/src-tauri && cargo test &&
  cargo clippy --all-targets -- -D warnings`, lalu `cd .. && pnpm test &&
  pnpm check`.
- [ ] Tanpa secret di diff: API key, password, private/signing key, isi
  keychain, `.env`. Nemu secret di output (diagnostics, env, log) → tulis
  `[REDACTED]`.
- [ ] String UI baru lengkap 3 bahasa (`id`/`en`/`zh`) di `i18n.ts` dalam
  commit yang sama.
- [ ] Style hanya token `--ks-*` di `app.css` (dark default + light); tanpa
  nilai luar token.
- [ ] Kontrak invoke berubah → `api.ts` + kedua sisi + test kontrak
  diperbarui; konstanta baru lewat `src/lib/contracts/*.json`.
- [ ] Perubahan perilaku non-trivial lewat `openspec/`; keputusan arsitektur
  baru dicatat sebagai ADR di `adr/`.
