# kata-validation Specification

## Purpose
The project's stop condition, recorded as a ledger: every claim of the
proposal is falsified or confirmed on katas, the FizzBuzz negative
control runs first and must self-negate, verdicts are pre-registered, and
each ledger rollup is sealed so later results cannot be retrofitted.

## Requirements

### Requirement: The negative control is recorded first and must self-negate
The system's validation run SHALL record the FizzBuzz negative-control
ledger entry before any other kata entry. The entry's verdict SHALL be
derived from the tool's own finding output via a comparison procedure
recorded in the entry itself. A run whose finding output cannot support
the "no categorical advantage" verdict is itself a failed validation.

#### Scenario: FizzBuzz self-negation
- **WHEN** the FizzBuzz kata model is checked and analyzed
- **THEN** the ledger entry records the verdict with the comparison
  procedure (findings fired vs conventional baseline) attached, with no
  claimed unique detection over conventional trivial tests, and it is the
  first kata entry in the ledger

### Requirement: Kata verdicts use the closed vocabulary with witnesses
The system SHALL record each kata run with verdict `killed | survived |
inconclusive | invalid-run`, the executed findings (or their absence),
the model-document hashes validated, the fixture revision, and the
expressibility gap where validated invariants exceeded path-equation
expressiveness.

#### Scenario: Survived kata cites its findings
- **WHEN** a kata reproduces its ct-catlab-validated findings as executed
  checks
- **THEN** the ledger entry is `survived` and cites each reproduced
  finding by id, with any inexpressible invariant listed in the gap field

#### Scenario: Defective run is invalid-run, never retried silently
- **WHEN** a kata run hits a tooling defect
- **THEN** the entry records `invalid-run` with the defect; a re-run is a
  new entry referencing the invalid one

### Requirement: A killed project verdict stops the build
The system SHALL treat a project-level `killed` ledger verdict as the
stop condition: the negative result is written, the ledger is kept, and
gated phases (P7 migration, P8 rule lens) do not start.

#### Scenario: Killed stops P7/P8
- **WHEN** the P6 ledger records a project-level killed verdict
- **THEN** the gated beads tickets remain closed-unevaluated and the
  negative-result document exists
