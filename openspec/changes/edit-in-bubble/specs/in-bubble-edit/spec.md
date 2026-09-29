## ADDED Requirements

### Requirement: Translated text SHALL be editable in the bubble body

A double-click on a translated bubble in the After pane SHALL open an
in-place text editor at the bubble position; typing edits the `translated`
text in place, committing via blur or Ctrl+Enter, cancelling via Esc.

#### Scenario: Double-click opens editor with current AI/manual text
- **WHEN** user double-clicks bubble #3 in the After pane
- **THEN** a textarea SHALL appear at bubble #3's position prefilled with
  its current `translated` value

#### Scenario: Typing does not disturb focus or overlay per keystroke
- **WHEN** user types in the in-bubble editor
- **THEN** no global translation state SHALL update until commit (typing
  stays local; overlay is not rebuilt per keystroke)

#### Scenario: Blur or Ctrl+Enter commits through the existing save path
- **WHEN** user blurs the editor or presses Ctrl+Enter
- **THEN** the value SHALL flow into `editTr(i, "translated", v)` (autosave
  debounce + `save_translation` + `ai*` baseline untouched), the bubble
  SHALL become `edited`

#### Scenario: Esc cancels without writing
- **WHEN** user presses Esc while editing
- **THEN** the editor SHALL close and no translation state SHALL change

#### Scenario: Switching bubble auto-commits
- **WHEN** user clicks another bubble while the editor is open
- **THEN** the running value SHALL commit first, then selection SHALL move
  (no typed text is silently lost)

### Requirement: Sidebar SHALL show one selected-bubble card

The translated sidebar list SHALL collapse from N per-bubble cards to a
single card for the selected bubble; the header stays.

#### Scenario: Single card follows selection
- **WHEN** user selects bubble #3 (click After/Before or card header)
- **THEN** sidebar SHALL show exactly bubble #3's card (Original, Reading,
  Translated, ↺ when `isUserEdited && aiTranslated`, `edited` badge,
  long-press glossary)

#### Scenario: No selection shows placeholder
- **WHEN** no bubble is selected
- **THEN** sidebar SHALL show a hint ("klik / double-klik bubble di After")
  instead of a card

#### Scenario: Header with Save edits stays
- **WHEN** a translation exists
- **THEN** the `translated` badge + `Save edits` button + overlay hint
  SHALL remain above the single card
