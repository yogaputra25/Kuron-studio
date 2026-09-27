## ADDED Requirements

### Requirement: Manual text entry SHALL NOT require AI

User selects a bubble, types `original`/`reading`/`translated` in textareas, and
saves via `saveTranslation` — no `translate_page` call.

#### Scenario: Fill AI-failed bubble manually
- **WHEN** bubble translation is null and user clicks Isi manual, types, saves
- **THEN** translation persists via `save_translation` and survives close → reopen

#### Scenario: Manual original survives re-translate
- **WHEN** user edits AI-produced `original` (sets `isUserEdited`) then re-translates
- **THEN** that bubble's manual `original`/`reading` SHALL NOT be overwritten
- **AND** other bubbles SHALL still update normally

### Requirement: Manual save SHALL reuse translation path

Saves go through the existing `saveTranslation` path with ~500ms debounce;
errors show in the existing banner; bubble indexes validated unique.

#### Scenario: Fast typing triggers one save
- **WHEN** user types rapidly in the textarea
- **THEN** one debounced save SHALL fire, not one per keystroke

#### Scenario: Failed save keeps local text
- **WHEN** save fails
- **THEN** banner SHALL show the error and local text SHALL remain
