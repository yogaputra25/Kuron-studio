## ADDED Requirements

### Requirement: Dev loop SHALL guarantee served frontend equals source on disk

`pnpm dev:fresh` (dari `kuron-studio/`) SHALL remove the Vite optimizer cache and then start `pnpm tauri dev`, so the webview serves the current source. It SHALL abort with a clear message when port 1420 is occupied, and SHALL remind the developer to hard-reload (`Ctrl+Shift+R`) after startup.

#### Scenario: Fresh start with no cache

- **WHEN** developer runs `pnpm dev:fresh` with port 1420 free
- **THEN** `kuron-studio/node_modules/.vite` SHALL be removed (when present; absent is not an error)
- **AND** `pnpm tauri dev` SHALL start with inherited stdio
- **AND** a `Ctrl+Shift+R` reminder SHALL be printed

#### Scenario: Occupied port aborts fast

- **WHEN** developer runs `pnpm dev:fresh` while port 1420 is already bound (zombie Vite)
- **THEN** the script SHALL abort before deleting anything
- **AND** SHALL print which command occupies the port and how to stop it

### Requirement: Nuke mode SHALL clear the WebView2 profile only with an explicit flag

`pnpm dev:fresh -- --nuke` SHALL additionally delete the dev WebView2 profile directory (Windows: `%LOCALAPPDATA%\id.kuron.studio\EBWebView`; macOS/Linux equivalents by platform), printing the exact path before deletion. Without the flag the profile SHALL be left untouched. A missing profile directory SHALL be skipped with an informational message, never an error.

#### Scenario: Nuke removes a bandel profile

- **WHEN** developer runs `pnpm dev:fresh -- --nuke` and the profile directory exists
- **THEN** the exact path SHALL be printed
- **AND** the directory SHALL be removed before starting `tauri dev`

#### Scenario: Default run never touches the profile

- **WHEN** developer runs `pnpm dev:fresh` without `--nuke`
- **THEN** the WebView2 profile directory SHALL NOT be modified, even when it exists

### Requirement: User data SHALL never be deleted by the dev script

Neither default nor `--nuke` mode SHALL create, modify, or delete anything under the app-data directory (`%APPDATA%\id.kuron.studio\` on Windows: `projects.json`, `kuron-studio.db`, `*_extracted/` folders). The Rust `target/` directory SHALL likewise never be touched.

#### Scenario: Data survives a nuke run

- **WHEN** developer runs `pnpm dev:fresh -- --nuke` with existing projects and translations
- **THEN** `projects.json`, `kuron-studio.db`, and all page/extracted files SHALL be byte-identical afterwards

### Requirement: The script SHALL be dependency-free and cross-platform

`scripts/dev-fresh.mjs` SHALL use only Node.js stdlib (`node:fs`, `node:net`, `node:child_process`, `node:path`, `node:os`) so no new entry appears in `package.json` dependencies. It SHALL run on Windows, macOS, and Linux (platform-specific profile paths resolved via `process.platform`).

#### Scenario: No new dependencies

- **WHEN** `pnpm install` runs after this change
- **THEN** `pnpm-lock.yaml` SHALL contain zero new packages attributable to `dev-fresh`
