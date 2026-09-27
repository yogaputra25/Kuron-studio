## ADDED Requirements

### Requirement: Image preview SHALL NOT block the Tauri async runtime

`get_image_preview` SHALL run its heavy work (file read, decode, resize, JPEG
encode) inside `tauri::async_runtime::spawn_blocking`, following the existing
`detect_bubbles` pattern. Error strings SHALL stay identical (`read failed`,
`decode failed`). The invoke wire key `{path, max_side}` SHALL NOT change.

#### Scenario: Large file preview stays responsive

- **WHEN** the frontend requests a 1600px preview of a large page image
- **THEN** the Tauri runtime SHALL remain responsive to other commands
- **AND** the preview SHALL resolve with a data-URL or a named error

### Requirement: Editor SHALL open progressively with thumbnail first

`openEditor` SHALL mount the editor immediately using the cached thumbnail
(or placeholder) without awaiting the full preview. The full preview SHALL
load in parallel and swap in on resolve. On failure, the thumbnail SHALL
remain visible with an error message (the editor stays usable).

#### Scenario: Open a large page

- **WHEN** the user opens a page whose full preview takes seconds
- **THEN** the editor with thumbnail SHALL appear within ~1s
- **AND** the full image SHALL replace it when ready

### Requirement: Every background load SHALL terminate with ready, error, or retry

`loadBackground` SHALL enforce a named timeout (15s): whichever fires first
(`onload`, `onerror`, timeout) wins per request token. A timed-out or failed
full load with a `fallbackUrl` SHALL try the fallback before giving up. Every
terminal failure SHALL show a message plus a "retry" action that reloads
without remounting. No "loading" state SHALL persist beyond the timeout.

#### Scenario: Full image hangs

- **WHEN** a background load neither resolves nor rejects within 15s
- **THEN** the editor SHALL show an error with a retry button
- **AND** SHALL have attempted the thumbnail fallback first (when provided)

#### Scenario: Manual retry

- **WHEN** the user clicks retry after a failed load
- **THEN** the same URL SHALL be requested again without remounting the stage
- **AND** overlay bubbles SHALL survive the retry

### Requirement: State copies SHALL use $state.snapshot, never structuredClone on reactive data

`structuredClone()` cannot clone Svelte 5 `$state` proxies and throws
`DataCloneError` exactly when bubble arrays are non-empty (post-detect),
killing the sync effect and leaving the overlay empty. All copies of reactive
bubble/translation state in `CanvasEditor.svelte` and `EditorPanel.svelte`
SHALL use the `$state.snapshot` rune (a plain deep copy, no import needed),
which accepts proxy and plain values alike.

#### Scenario: Sync non-empty bubbles after detect

- **WHEN** the sync effect runs with 4 bubbles in `initial`
- **THEN** no exception SHALL be thrown
- **AND** the 4 overlay boxes SHALL render

### Requirement: Deleting a bubble SHALL persist without a manual save step

The sidebar delete button and keyboard delete currently mutate local state
only (`splice` + `dirty`), leaving the parent `page` prop at N bubbles — so
`syncFromPage()` resets the list back to N on the next effect run (bubble
"reappears") and the canvas never learns of the deletion. Every delete path
SHALL persist via `save()` (which calls `onSaved`, updating the prop so the
sync guard stays quiet). While a save is in flight, delete controls SHALL be
disabled to prevent double-splice races; save errors SHALL surface in the
existing banner.

#### Scenario: Sidebar delete persists

- **WHEN** the user clicks a bubble's delete button
- **THEN** the list SHALL shrink to N-1 and stay N-1 after close → reopen
- **AND** the canvas overlay SHALL update accordingly

#### Scenario: Keyboard delete persists

- **WHEN** the user selects a box and presses Delete
- **THEN** the same persistence as sidebar delete SHALL apply
