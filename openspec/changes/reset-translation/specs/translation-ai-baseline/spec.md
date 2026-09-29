## ADDED Requirements

### Requirement: AI baseline SHALL survive manual edits

Every `BubbleTranslation` SHALL carry the last pure-AI text
(`aiOriginal`/`aiReading`/`aiTranslated`) alongside the current editable text.
AI paths SHALL fill both; manual paths SHALL NOT touch the baseline.

#### Scenario: Fresh AI result sets baseline
- **WHEN** `translate_page`/batch/`retry_bubble` produces a bubble
- **THEN** `ai*` SHALL equal `original`/`reading`/`translated` and
  `isUserEdited` SHALL be false

#### Scenario: Manual edit keeps baseline intact
- **WHEN** user edits current text (sets `isUserEdited`) and saves via
  `save_translation`, then closes → reopens
- **THEN** current text SHALL be the manual text AND `ai*` SHALL still hold
  the pre-edit AI text

#### Scenario: Re-translate preserves baseline of edited bubbles
- **WHEN** an edited bubble survives re-translate via preserve
- **THEN** its `ai*` SHALL be the old baseline, not the rejected new AI text

#### Scenario: Old projects.json loads without baseline
- **WHEN** a stored translation without `ai*` fields is loaded
- **THEN** it SHALL parse with `ai*` defaulting to empty (no migration)

### Requirement: Reset SHALL restore AI without calling AI

An edited bubble SHALL offer a per-bubble reset that copies `ai*` back to
current text, clears `isUserEdited`, and saves via the existing
`saveTranslation` path — no `translate_page`/`retry_bubble` call.

#### Scenario: Reset edited bubble to normal
- **WHEN** user clicks ↺ on an edited bubble and save succeeds
- **THEN** current text SHALL equal `ai*`, badge `edited` SHALL disappear,
  other bubbles SHALL be unchanged, and no AI provider SHALL be called

#### Scenario: Reset hidden without baseline
- **WHEN** a bubble has empty AI baseline (e.g. created before this feature)
- **THEN** no ↺ button SHALL be shown for that bubble

#### Scenario: Reset survives re-translate as normal bubble
- **WHEN** a reset bubble (flag false) goes through re-translate
- **THEN** it SHALL update from new AI output like any unedited bubble

### Requirement: New AI paths SHALL fill the baseline

Any present or future code path that writes AI-produced bubble text SHALL
also write the same text to `ai*` (baseline = AI shown to user).

#### Scenario: Retry sets new baseline
- **WHEN** `retry_bubble` refreshes the target bubble from new AI output
- **THEN** that bubble's `ai*` AND current text SHALL be the new AI text
  with `isUserEdited` false
