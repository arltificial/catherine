# domain-probes delta

## ADDED Requirements

### Requirement: Probe entries are pre-registered before the run
The system SHALL record a domain probe only against an expectation
catalog committed before the run: fault → class → expected catherine
outcome, referenced from the entry by content hash. A probe entry whose
catalog hash does not resolve to a committed catalog is refused.

#### Scenario: Probe without a pre-registered catalog is refused
- **WHEN** a probe entry is recorded whose `catalog_hash` does not resolve
  to a committed expectation catalog
- **THEN** the ledger validator refuses the entry with a named violation
  and the probe is not written

#### Scenario: Expectations cannot be retrofitted
- **WHEN** a recorded probe's catalog file is edited after the entry's
  content hash was sealed
- **THEN** the catalog hash no longer resolves and the ledger fails
  validation, forcing a superseding entry instead of a silent edit

### Requirement: Unique-detection claims require a recorded baseline arm
The system SHALL refuse any probe verdict claiming unique detections
without a conventional-baseline arm recorded in the entry: the baseline
tool or practice, what it detects on the same fault set, and the derived
unique set. Without a baseline arm, `survived` is possible only with
empty `unique_detections` (parity ceiling).

#### Scenario: Unique claim without a baseline is refused
- **WHEN** a probe entry records a non-empty `unique_detections` with no
  baseline arm
- **THEN** the validator refuses the entry

#### Scenario: Parity verdict without a baseline is recorded honestly
- **WHEN** a probe fired findings but no conventional baseline was run
- **THEN** the entry records the detections, an empty unique set, and the
  missing baseline as a named gap — never as a unique-detection claim

### Requirement: Probes never touch the sealed kata rollup
The system SHALL keep probe entries in their own ledger sequence
(`ledger/probes/`), refused inside the kata ledger; the kata rollup's
sealed ids, verdict, and control-first ordering SHALL NOT be recomputed,
extended, or superseded by any probe entry.

#### Scenario: Probe entry in the kata ledger is refused
- **WHEN** an entry with kind `probe` appears in the kata ledger sequence
- **THEN** the validator refuses the ledger

#### Scenario: Kata rollup is invariant under probe recording
- **WHEN** probe entries are recorded after the kata rollup was sealed
- **THEN** `ledger-check` over the kata ledger still passes with the
  original rollup id and verdict, byte-identical

### Requirement: Probe verdicts use the closed vocabulary with witnesses
The system SHALL record each probe run with verdict `killed | survived |
inconclusive | invalid-run`, the executed findings by id (or their
absence), the model-document hashes validated, the expressibility gaps,
and the procedure the verdict was derived by — the same honesty
requirements the kata ledger enforces.

#### Scenario: Survived probe cites its findings and gaps
- **WHEN** a probe's expectations are met by executed findings
- **THEN** the entry is `survived`, cites each fired finding by id, maps
  each detection to its pre-registered expectation, and lists every
  expectation the run could not reach as a gap

#### Scenario: Killed probe records the null result
- **WHEN** the conventional baseline arm detects everything the probe
  fired, or the probe fires nothing on faults the catalog predicted
- **THEN** the entry is `killed` with the comparison recorded — the null
  result is the deliverable, never suppressed
