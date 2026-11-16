# gap-suite Specification

## Purpose
The registry of known expressibility limitations: every claim that the tool
cannot state or see something lives as a table row bound to an executable CLI
case, so fixing a limitation without flipping its disposition fails CI and a
stale claim cannot outlive the conversation it came from.

## Requirements

### Requirement: Every registered expressibility limitation is an executable disposition
The system SHALL maintain a disposition table mapping each known
expressibility limitation to an executable case, and the suite SHALL
fail whenever a case's observed behavior contradicts its row's
disposition — a `gap` row whose limitation no longer holds fails until
the row is flipped to `closed` citing the fixing evidence, and a
`closed` row whose limitation reappears fails until the row is flipped
back with the regression recorded.

#### Scenario: Fixed limitation must flip its row
- **WHEN** a `gap` row's case no longer exhibits the claimed limitation
  (e.g. a previously unparseable equation form now parses)
- **THEN** the suite fails naming the row, and passes only after the row
  is flipped to `closed` citing the fixing commit in the same change

#### Scenario: Regressed limitation must flip its row back
- **WHEN** a `closed` row's case exhibits the limitation again
- **THEN** the suite fails naming the row, and passes only after the row
  records the regression and flips back to `gap`

### Requirement: Rows cite chat provenance and assert HEAD truth
The system SHALL register each row with its source investigation
(provider and conversation id from the research corpus) for provenance,
and SHALL verify the row's claimed behavior against the current build
at registration time — a row whose claim contradicts observed HEAD
behavior SHALL be registered as `closed` with the fixing evidence, not
imported as a stale `gap`.

#### Scenario: Stale claim registers closed, not as a gap
- **WHEN** an investigation claims a limitation that the current build
  no longer exhibits (behavior changed since the investigation ran)
- **THEN** the row is registered `closed` citing the fixing evidence, and
  the suite passes with the row's case asserting the current behavior

#### Scenario: Unregistered case fails the suite
- **WHEN** the harness runs a case that has no disposition row
- **THEN** the suite fails naming the case — every executable case must
  be a table row, every table row must have a case

### Requirement: Limitation claims outside the table fail the gate
The system SHALL enforce, as a CI gate over known doc locations (README,
CHANGELOG, `docs/**`, openspec close notes — machine files excluded),
that any sentence registering an expressibility limitation names a
disposition row id; a claim without a row fails the gate, and the gate
itself runs under `just ci` like the other mechanical gates.

#### Scenario: Claim without a row fails the gate
- **WHEN** a doc, CHANGELOG entry, or close note registers an
  expressibility limitation without naming a disposition row id
- **THEN** the gate fails naming the file and line — the claim cannot
  exist outside the table

#### Scenario: Gate runs in CI
- **WHEN** `just ci` runs
- **THEN** the gate executes with the other mechanical gates and passes
  when every limitation claim names a registered row
