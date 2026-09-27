## ADDED Requirements

### Requirement: Editor canvas SHALL render bubbles even when the page image is not yet loaded

`CanvasEditor` SHALL create its Konva stage and overlay layer on mount without
waiting for the page image. Bubbles from `initial` SHALL be drawn on the
overlay within one frame of mount, even when `imageUrl` is empty. When
`imageUrl` arrives or changes afterwards, the background image SHALL be
swapped in without destroying the overlay.

#### Scenario: Open editor before preview fetch completes

- **WHEN** the editor mounts with `imageUrl === ""` and 4 bubbles in `initial`
- **THEN** the stage and 4 overlay boxes SHALL be visible immediately
- **AND** a "loading image" hint SHALL be shown

#### Scenario: Late image arrival swaps background

- **WHEN** `imageUrl` changes from `""` to a valid data-URL
- **THEN** the background SHALL appear behind the existing overlay
- **AND** the overlay boxes SHALL NOT be destroyed or duplicated

#### Scenario: Stale load is ignored

- **WHEN** the user navigates to another page while an image is still loading
- **THEN** the late `onload` from the previous page SHALL be ignored

### Requirement: Every image failure state SHALL show a message and an exit

The editor SHALL never sit on a silent black canvas. Each failure (empty URL
after fetch, fetch error, decode error) SHALL display a message inside the
editor itself (not only on the grid behind the overlay) and the "back to grid"
control SHALL remain enabled.

#### Scenario: Preview fetch fails

- **WHEN** `get_image_preview` rejects for the opened page
- **THEN** the editor SHALL show an error message naming the failure
- **AND** the ← Grid button SHALL return to the grid without reload
