## ADDED Requirements

### Requirement: Editor SHALL offer Before|Split|After modes

A `Before | Split | After` switcher controls the body: Before = single editable
pane without text overlay; After = single read-only pane with text overlay;
Split = both side by side.

#### Scenario: Each mode renders correctly
- **WHEN** user picks Before, Split, or After
- **THEN** one editable pane, two panes, or one read-only overlaid pane SHALL show

### Requirement: Split panes SHALL scroll together

Scrolling one pane copies `scrollTop`/`scrollLeft` to the other under a guard
flag (no loop); bubble overlays survive the scroll.

#### Scenario: Scroll left follows right
- **WHEN** user scrolls the left pane 300px
- **THEN** the right pane SHALL scroll to the same 300px without feedback loop

### Requirement: After pane SHALL be read-only with selection forwarding

Clicks/drags in After change no state; clicking a bubble in After selects the
same bubble in Before (single source of state).

#### Scenario: Click bubble in After selects in Before
- **WHEN** user clicks bubble #3 in the After pane
- **THEN** `selectedIdx` in Before SHALL be 3 and bubbles SHALL be unchanged

### Requirement: Both panes SHALL share one image URL

Split mode fetches `get_image_preview` once and shares the reference — never
two requests for the same image.

#### Scenario: Open split fetches once
- **WHEN** user opens Split mode
- **THEN** `get_image_preview` SHALL be called once (same reference both panes)
