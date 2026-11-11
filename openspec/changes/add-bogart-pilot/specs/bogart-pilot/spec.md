# bogart-pilot delta (P9)

## ADDED Requirements

### Requirement: Five prose invariants become executable pilot models
The system SHALL provide `.ct.json` pilot models in bogart's repo for the
five DESIGN §5 mappings (FTS-anchor equation + totality; identity-domain
disjointness; capture-selector overlap/gap; raw→serving Δ migration;
alerts tiered rules), each traceable to the prose invariant it encodes.

#### Scenario: Drifted anchor is caught
- **WHEN** a bogart text_block row carries an edition anchor inconsistent
  with its document path (or dangles)
- **THEN** `catherine check` on the FTS-anchor model emits the
  corresponding violation/dangling finding with the offending row

### Requirement: The pilot runs in bogart's CI with class exit codes
The system SHALL wire `catherine check` into bogart's CI over the pilot
models, failing the build on violation/dangling/contradiction classes via
the P3 exit-code contract — no output text parsing.

#### Scenario: CI fails on a real violation
- **WHEN** bogart CI runs the pilot suite against a corpus state with a
  drifted anchor
- **THEN** the catherine step exits with the violation-class code and the
  build fails

### Requirement: The pilot report accounts for real vs noise
The system SHALL record a pilot report listing every finding fired on the
live corpus, classified as real defect or noise with cause — noise is
never silently suppressed, and confirmed defects link to their bogart
fixes.

#### Scenario: Noise is accounted
- **WHEN** a pilot finding corresponds to no real bogart defect
- **THEN** the report classifies it as noise with a stated cause (model
  imprecision, granularity), and the count is part of the ledger record
