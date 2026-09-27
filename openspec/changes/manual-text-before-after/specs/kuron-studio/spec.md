## MODIFIED Requirements

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

#### Scenario: Edit bubble text manually
- **WHEN** user types `original`/`reading` in the selected-bubble textarea
- **THEN** system SHALL persist via `saveTranslation` without calling `translate_page`

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

#### Scenario: Manual original preserved
- **WHEN** `isUserEdited=true` on a manually typed `original`/`reading`
- **THEN** re-translate SHALL NOT overwrite `original`/`reading` either
