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

- `App.svelte`: state root — `projects`, `currentId`, `folderId`,
  `selectedIds` (multi-select), `thumbs`, panel mana yang terbuka.
  Dashboard + breadcrumb + grid + header + panel di sini.
- `lib/library.ts`: agregat murni — `subtreePages/Stats`,
  `folderPath/childrenOf/hasNesting`, `rangeSelect`,
  `isSiblingNameTaken` (cermin guard backend). Unit-testable.
- `lib/api.ts`: satu-satunya jembatan invoke. **Kontrak**:
  key top-level `snake_case` persis nama param Rust
  (`project_id`, `page_ids`); field struct `camelCase`.
  Dijaga `invoke-contract.test.ts`.
- `lib/types.ts`: cermin tipe Rust. `IMAGE_EXTS` wajib sinkron dengan
  `project.rs` (dijaga `image-ext-contract.test.ts`). Jangan tambah
  ekstensi di satu sisi saja.
- Panel: `EditorPanel` (canvas Rect/Ellipse/Freeform/Tail),
  `ReviewGrid` (progres, QA, TM, ekspor), `BatchPanel`,
  `GlossaryPanel`, `ProviderSettings`, `DiagnosticsPanel`,
  `MoveDialog` (pindah halaman + LOV tujuan), `FolderChip`
  (rename inline + hapus), `ui/Lov` (satu-satunya dropdown —
  tombol + search + listbox, dipakai semua form, ganti `Select`).
- `i18n.ts`: id/en/zh, tombol header cycler, persist localStorage.
  `i18n.test.ts` gagal kalau ada key hilang di satu bahasa.
- `app.css`: token `--ks-*` (netral warm, aksen coral, sekunder cyan),
  dark default + light. Jangan tambah nilai di luar token.
- `device.ts`: deteksi mobile → sembunyikan folder-picker/updater/
  PNG-PSD (dijaga `mobile-gating.test.ts`).

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
    `bubble.rs::sanitize_bubbles`: buang nol-area, cap 500/page.
  - `translate`: 5 provider BYOK (Gemini, OpenAI, Meta AI, Cohere,
    Custom), validate, `<15s`, status
    `translating/translated/failed`, retry per page/bubble.
    Preset hapus (OpenRouter/Zen/OpenCodeGo/ClinePass) terbaca sebagai
    Custom; label vision dari allowlist per provider (#26/#27).
    Model per-run: override opsional di tiap request translate
    (default = simpanan provider), tercatat di `translation.model`.
  - `export`: JSON / PNG (overlay, typeset TTF sistem + fallback
    bitmap) / CBZ / PSD (folder = dir, file = path).
  - `extras`: `tm_search` lintas project, `qa_check`,
    `share_project` (zip slice projects.json + gambar), diagnostics/log.
  - `glossary`: CRUD + CSV import/export, dipakai otomatis + manual.
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
