# cli delta

## ADDED Requirements

### Requirement: Check and findings payloads carry the execution scope
Every `catherine check` and `catherine find` JSON envelope SHALL carry
`data.scope`, an object with exactly three snake_case keys:
`constraints_checked` (accepted constraints the check walk executed),
`rows_evaluated` (instance rows the walk evaluated), and
`equations_over_zero_rows` (accepted equations that bind zero rows).
The scope SHALL be computed in one place shared by both subcommands so
their reports cannot disagree.

#### Scenario: Every check envelope carries the scope
- **WHEN** `catherine check model.ct.json --json` completes a check
  walk over a document with accepted constraints and rows
- **THEN** the envelope's `data` carries a `scope` object whose keys
  are exactly `constraints_checked`, `rows_evaluated`, and
  `equations_over_zero_rows`, with counts matching the executed walk

#### Scenario: Check and findings agree
- **WHEN** the same document is run through `check --json` and
  `findings --json` in one session
- **THEN** both envelopes report the identical `data.scope` values

### Requirement: A vacuous run reports a zero scope, never a clean-shaped silence
A run whose walk executes nothing — a document below the execution
floor, a constraint-free schema, or a zero-row instance — SHALL report
`data.scope` with all three keys `0` instead of an empty findings
array indistinguishable from a genuine clean pass. The exit code for
such a run is unchanged.

#### Scenario: Zero-row instance is visible in the payload
- **WHEN** a document with accepted equations is checked against an
  instance whose tables contain no rows, and it exits clean
- **THEN** `data.scope` reports `rows_evaluated: 0` (with
  `equations_over_zero_rows` counting the equations bound over the
  empty rows), so the clean exit is visibly vacuous

#### Scenario: Constraint-free document is visible in the payload
- **WHEN** a document with no accepted constraints is checked, and it
  exits clean
- **THEN** `data.scope` reports `constraints_checked: 0`, so nothing
  was checked is distinguishable from clean

### Requirement: The scope field is additive and bytes elsewhere are frozen
Adding `data.scope` SHALL NOT alter any existing payload field, any
other envelope byte, or any exit code. Single-document runs SHALL keep
today's encoding exactly, plus the new field. Commands that do not
execute accepted constraints over an instance (`search`, `diff`,
`migrate`, `rules`, the pattern and naturality lenses, `query`) SHALL
NOT carry a scope field.

#### Scenario: Single-document bytes gain only the new field
- **WHEN** the golden fixture for a single-document check is compared
  byte-for-byte before and after the change
- **THEN** the only difference is the added `data.scope` object;
  findings, warnings, envelope kind, and version fields are identical

#### Scenario: Exit codes are untouched
- **WHEN** the same fixture corpus is run before and after the change
- **THEN** every exit code, clean or finding-classed, is unchanged,
  including the vacuous runs that exit 0 with a zero scope

#### Scenario: Non-executing commands stay scope-free
- **WHEN** `catherine search`, `diff`, `migrate`, `rules`, `query`,
  or the pattern/naturality lenses emit their JSON payloads
- **THEN** no `data.scope` field appears — these commands execute no
  accepted constraints over an instance
