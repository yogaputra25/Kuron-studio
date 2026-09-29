## ADDED Requirements

### Requirement: Per-page translate SHALL be cancellable

A running per-page translate SHALL offer Batal: the UI releases instantly,
the backend stops at the next check without persisting or failing the page,
and the page status returns to what it was before the translate started.

#### Scenario: Cancel during backoff wait stops retries
- **WHEN** user clicks Batal while `backoff_translate` sleeps (2s/4s/8s)
- **THEN** no further provider attempt SHALL fire, nothing SHALL persist,
  and status SHALL return to pre-claim (not failed)

#### Scenario: Cancel during in-flight POST discards the result
- **WHEN** cancel lands while a provider POST is in flight
- **THEN** the late result SHALL be discarded before map/store (one call's
  tokens still spent — documented, not refunded)

#### Scenario: Late response after cancel is ignored
- **WHEN** the cancelled `translatePage` promise resolves or rejects late
- **THEN** UI SHALL ignore it: no `onTranslated`, no error banner

#### Scenario: Cancel restores pre-claim status
- **WHEN** the page was `translated` (re-translate) or `detected` before claim
- **THEN** after cancel it SHALL be that status again, never a default

#### Scenario: Batch queue keeps running
- **WHEN** a page inside a running batch is cancelled via `cancel_translate`
- **THEN** that page SHALL stop quickly, but the batch queue/progress SHALL
  continue (full batch-cancel stays out of scope)
