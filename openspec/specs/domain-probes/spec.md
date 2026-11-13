# domain-probes Specification

## Purpose
Domain probes are pre-registered, executed validation runs over non-kata
modeling domains — Δ migration at real scale, and the rule lens on tables
extracted from prose/PDF sources. Each probe records in its own ledger
directory against an expectation catalog committed before the run, with a
floor control that must self-negate and a committed, executed conventional
baseline arm, so a unique-detection claim is arithmetic against incumbent
tooling rather than rhetoric.

## Requirements

### Requirement: Domain-probe series run in their own ledger sequence
The system SHALL record a domain probe in its own ledger directory
(`ledger/probes/<name>/`) reusing the kata entry shape and validator —
content-addressed ids, closed verdict vocabulary, append-only
supersession — with its own floor control entry that must self-negate
before any probe entry is accepted. The sealed project rollup in the
kata ledger SHALL NOT be recomputed, extended, or superseded by probe
recording.

#### Scenario: Probe series without a self-negating floor control is refused
- **WHEN** a probe ledger sequence records a non-control entry with no
  prior control entry that self-negated (verdict `killed`)
- **THEN** the ledger validator refuses the sequence with the existing
  named violations (`NegativeControlMissing` / `ControlDidNotSelfNegate`)

#### Scenario: Kata rollup is invariant under probe recording
- **WHEN** probe entries are recorded after the kata rollup was sealed
- **THEN** `ledger-check` over the kata ledger still passes with the
  original rollup id and verdict, byte-identical

### Requirement: Unique-detection claims require a committed executed baseline arm
The system SHALL refuse any ledger entry — probe or kata — recording a
non-empty `unique_detections` without a baseline arm that names a
committed, version-pinned baseline artifact (script or tool + revision)
and its detections on the same fault set. A baseline that is cited but
not executed caps the entry's verdict at `inconclusive` on uniqueness
questions. Without any baseline arm, `survived` is possible only with an
empty unique set (parity ceiling).

#### Scenario: Unique claim without a baseline arm is refused
- **WHEN** an entry records a non-empty `unique_detections` with no
  baseline arm naming a committed baseline artifact
- **THEN** the validator refuses the entry

#### Scenario: Cited-only baseline cannot carry a uniqueness verdict
- **WHEN** an entry's baseline arm is labeled cited (not executed) and
  the entry claims unique detections
- **THEN** the validator refuses a `survived` verdict on that entry;
  `inconclusive` with the missing execution recorded as a gap is the
  ceiling

#### Scenario: Parity verdict without a baseline is recorded honestly
- **WHEN** a probe fired findings but no baseline arm was run
- **THEN** the entry records the detections, an empty unique set, and the
  missing baseline as a named gap — never as a unique-detection claim

### Requirement: Probe expectations are pre-registered and retrofit-proof
The system SHALL run a probe only against expectations committed before
the run: the entry embeds its expectation rows and carries the hash of
its committed probe catalog; an unregistered probe (no catalog rows)
cannot run, and editing a sealed entry or its catalog breaks the content
address, forcing a superseding entry instead of a silent edit.

#### Scenario: Unregistered probe cannot run
- **WHEN** a probe id has no committed catalog rows
- **THEN** the harness records the empty-expectation state and refuses to
  derive a verdict from it — the same discipline an unknown kata id gets

#### Scenario: Expectations cannot be retrofitted
- **WHEN** a recorded probe's catalog file or entry is edited after its
  content hash was sealed
- **THEN** the id/hash no longer resolves and ledger validation fails,
  forcing a superseding entry

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
- **WHEN** the baseline arm detects everything the probe fired, or the
  probe fires nothing on faults the catalog predicted
- **THEN** the entry is `killed` with the comparison recorded — the null
  result is the deliverable, never suppressed
