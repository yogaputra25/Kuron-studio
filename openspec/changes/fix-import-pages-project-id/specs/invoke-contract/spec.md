## ADDED Requirements

### Requirement: All Tauri commands SHALL declare rename_all snake_case so wire keys equal Rust parameter names

Every `#[tauri::command]` in `kuron-studio/src-tauri/src/commands/*.rs` SHALL declare `rename_all = "snake_case"` (30 commands), because `tauri-macros` defaults to `ArgumentCase::Camel` and `tauri/src/ipc/command.rs` looks up args by the declared key — without the attribute the backend expects camelCase (`projectId`) even though the Rust parameter is snake_case (`project_id`). With the attribute, the wire key for top-level invoke args is identical to the Rust parameter name.

- `import_pages`: wire `{ project_id, paths }` accepted — NOT rejected with `missing required key projectId`
- All commands in `project.rs`, `image.rs`, `detect.rs`, `provider.rs`, `translate.rs`, `batch.rs`, `glossary.rs`, `export.rs`, `extras.rs`: wire keys SHALL equal the Rust parameter names (`project_id`, `page_id`, `page_ids`, `provider_id`, `bubble_texts`, `path`, `format`, `query`, `limit`, `csv`, `input`, `bubbles`, `source`, `target`, `id`, `max_side`, `name`, ...)
- Struct payload fields inside `{ input }` wrappers follow serde `rename_all = "camelCase"` and are exempt from this rule.

#### Scenario: Import pages with snake_case keys succeeds

- **WHEN** frontend calls `invoke("import_pages", { project_id, paths })` with an existing project id and a list of image paths
- **THEN** backend SHALL accept the args (wire key `project_id` matches via `rename_all = "snake_case"`), import the pages in filename order, and return `ImportResult { pages, skipped }` without an invalid-args error

#### Scenario: New command without the attribute is rejected

- **WHEN** a developer adds a `#[tauri::command]` without `rename_all = "snake_case"`
- **THEN** the contract guard SHALL fail the build (or, at minimum, the review checklist SHALL flag it) before the camelCase mismatch reaches runtime

### Requirement: Invoke argument keys SHALL match Rust command parameter names exactly

Every Tauri `invoke("<command>", args)` call from the frontend SHALL use top-level argument keys identical (case-sensitive) to the `#[tauri::command]` Rust function parameter names, which are snake_case. Struct payload fields inside `{ input }` wrappers follow serde `rename_all = "camelCase"` and are exempt from this rule.

Reference contract (frontend `kuron-studio/src/lib/api.ts` — UNCHANGED, already correct — vs backend `kuron-studio/src-tauri/src/commands/*.rs` with `rename_all = "snake_case"`):

- `import_pages`: `{ project_id, paths }` — NOT `{ projectId, ... }`
- `get_project` / `export_project` / `qa_check` / `share_project`: `{ project_id, ... }`
- `detect_bubbles` / `save_translation` / `retry_bubble`: `{ page_id, ... }`, `detect_bubbles_batch` / batch inputs: `{ page_ids, ... }` / `{ input }`
- `get_image_preview`: `{ path, max_side }`; `glossary_context`: `{ bubble_texts }`; `tm_search`: `{ query, limit }`

#### Scenario: Import pages with correct keys succeeds

- **WHEN** frontend calls `invoke("import_pages", { project_id, paths })` with an existing project id and a list of image paths
- **THEN** backend SHALL import the pages in filename order and return `ImportResult { pages, skipped }` without an invalid-args error

#### Scenario: camelCase key is rejected with a clear error

- **WHEN** any caller sends `invoke("import_pages", { projectId, ... })`
- **THEN** no such call SHALL exist in `kuron-studio/src/` — the static contract guard SHALL fail the build before runtime

#### Scenario: Struct-wrapped commands keep camelCase fields

- **WHEN** frontend calls `invoke("translate_page", { input })` where `input` contains `pageId`, `providerId`, `targetLang`
- **THEN** backend SHALL accept it (inner fields are camelCase per `TranslatePageInput` serde renames) and the guard SHALL NOT flag it

### Requirement: All frontend Tauri calls SHALL go through the single invoke bridge

`kuron-studio/src/lib/api.ts` is the single authorized invoke bridge. No Svelte component, helper, or test SHALL call Tauri `invoke()` directly; all calls go through the `api` object so the NOTE contract ("invoke keys must match Rust param names exactly") is enforced in one place.

#### Scenario: No direct invoke outside the bridge

- **WHEN** the contract guard scans `kuron-studio/src/`
- **THEN** every `invoke("...")` call site SHALL be inside `kuron-studio/src/lib/api.ts`, and zero direct `invoke` imports SHALL exist in components

### Requirement: CI SHALL fail on invoke-key contract violations

A static regression guard (vitest case or `pnpm test` script, no new dependencies) SHALL extract top-level keys of each `invoke("<cmd>", {...})` literal in `kuron-studio/src/` and compare them against the expected snake_case parameter list per command. Any camelCase top-level key (e.g. `projectId`, `pageId`, `providerId`, `bubbleTexts`) SHALL fail `pnpm test`. The guard SHALL also verify every `#[tauri::command]` in `commands/*.rs` declares `rename_all = "snake_case"`.

#### Scenario: Guard catches a camelCase regression

- **WHEN** a developer adds `invoke("import_pages", { projectId, paths })` anywhere in `kuron-studio/src/`
- **THEN** `pnpm test` SHALL fail with a message naming the command, the offending key, and the expected key

#### Scenario: Gates stay green after the fix

- **WHEN** the fix is applied
- **THEN** `cargo test`, `cargo clippy --all-targets -- -D warnings`, `pnpm test`, and `pnpm check` SHALL all pass, and manual `pnpm tauri dev` import (folder + zip + drag-drop) SHALL populate the grid with thumbnails
