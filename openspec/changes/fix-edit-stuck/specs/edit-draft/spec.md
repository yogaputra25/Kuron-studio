## ADDED Requirements

### Requirement: Card text fields SHALL edit from local draft state

Typing in the single-card textareas SHALL write to a local draft, not to
`translation.bubbles`; the global state SHALL be written once on commit
(blur or debounce), so the cursor never jumps on keystroke.

#### Scenario: Typing mid-text keeps cursor position
- **WHEN** user types or backspaces in the middle of a card field
- **THEN** no global translation state SHALL update until commit (the
  textarea value SHALL NOT be re-set from state per keystroke)

#### Scenario: Blur commits through the existing save path
- **WHEN** user blurs a card textarea with a dirty draft
- **THEN** the value SHALL flow into `editTr(row, field, v)` (autosave
  debounce + `save_translation` + `ai*` baseline untouched), the bubble
  SHALL become `edited`

#### Scenario: Continuous typing commits via debounce
- **WHEN** user keeps typing without blurring
- **THEN** the draft SHALL commit through `editTr` on debounce (same
  ~500ms rhythm as the existing autosave)

#### Scenario: Switching bubble auto-commits the draft
- **WHEN** user selects another bubble while a card draft is dirty
- **THEN** the running draft SHALL commit first, then selection SHALL move
  (no typed text is silently lost; consistent with in-bubble behavior)

### Requirement: Save responses SHALL NOT drop uncommitted keystrokes

A `save_translation` response arriving while the user is typing SHALL NOT
erase keystrokes typed after the sent snapshot.

#### Scenario: Fast typing across a save round-trip loses nothing
- **WHEN** user types A, a save for A is sent, user types B before the
  response arrives, then the response arrives
- **THEN** B SHALL survive (draft lives outside `translation`, so
  `translation = out` cannot touch it) and a follow-up save SHALL persist B

### Requirement: In-bubble editor SHALL use two-way binding

The in-bubble `<textarea>` SHALL bind two-way to its (already local) editing
state, removing the last one-way write path in text editors.

#### Scenario: In-bubble typing keeps cursor position
- **WHEN** user types or backspaces mid-text in the in-bubble editor
- **THEN** the cursor SHALL stay where the user put it

### Requirement: Overlapping saves SHALL resolve last-write-wins (§4)

Each `save_translation` request SHALL carry a generation; a response SHALL
only write state when its generation is still the latest. Stale responses
SHALL be ignored.

#### Scenario: Backspaced text does not resurrect
- **WHEN** save("abc") is in flight, user backspaces to "ab", save("ab")
  is sent, and the save("abc") response arrives LAST
- **THEN** the stale response SHALL be ignored; "ab" SHALL remain displayed
  and the persisted state SHALL converge to "ab" via the follow-up save

#### Scenario: Restore (↺) wins over in-flight saves
- **WHEN** a save carrying manual text is in flight and the user clicks ↺
- **THEN** the AI baseline SHALL be sent immediately (not debounced) under
  a newer generation; when the stale manual-text response arrives it SHALL
  be ignored, so the AI text SHALL persist and the `edited` badge SHALL stay
  off
