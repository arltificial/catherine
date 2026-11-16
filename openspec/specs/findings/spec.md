# findings Specification

## Purpose
The output contract with agents and humans: diagnostics are typed
findings, each naming its kind, carrying the witness that proves it, and
stating how it was established. No booleans, no bare "no violation
found".

## Requirements

### Requirement: Findings are typed and carry witnesses
The system SHALL represent every diagnostic as a `Finding` value whose
variant names its kind and whose payload carries the concrete witness
that demonstrates it. The kind set SHALL stay exactly five — violation,
dangling, redundant, contradiction, underconstraint — and a finding
without a witness SHALL be unrepresentable. A key collision is
violation-shaped, never a sixth kind.

#### Scenario: A finding always carries its witness
- **WHEN** any diagnostic is constructed
- **THEN** its payload carries the witness for its kind (construction
  without one is unrepresentable), and a key collision produces a
  violation witness rather than a sixth kind

### Requirement: Witness shapes are fixed per finding kind
Each kind's witness shape SHALL be fixed: a violation carries the
equation witness (equation id, row, both path values) or the key witness
(key index, both colliding rows); dangling carries arrow and source row;
redundant carries derivable and reducing path; contradiction carries
explicit scope; underconstraint carries a domain question
(`ParallelPaths{lhs, rhs}` or `UncoveredInputs{class, examples}`).

#### Scenario: Witness encodings are stable
- **WHEN** findings serialize
- **THEN** the equation witness encodes byte-identically to its pre-key
  form and the key witness encodes as `{"key": n, "rows": [lo, hi]}`, so
  both remain distinguishable without consulting the schema

#### Scenario: Violation finding carries row and values
- **WHEN** a kernel check produces a violation
- **THEN** the finding is `Violation` carrying the equation id, the
  offending row id, and both composed path values — assertable
  structurally, not by output text

#### Scenario: Key violation carries its rows
- **WHEN** a kernel check produces a key violation
- **THEN** the finding is `Violation` carrying the key's canonical index and
  both colliding row ids, and the encoding is distinguishable from an
  equation witness without consulting the schema

#### Scenario: Every finding kind has a golden fixture
- **WHEN** the fixture suite runs
- **THEN** each v1 finding kind has a golden fixture whose assertions
  check witness fields structurally, including both violation witness
  shapes (producers for analysis kinds are exercised via P4, but the
  taxonomy and fixtures exist here)

### Requirement: Every finding states how it was established
The system SHALL attach an epistemic label to every finding — `executed`
for kernel-computed results, `hypothesized` for proposed-equation
failures and underconstraint questions, `unknown` for scope-bounded
non-answers — and SHALL never emit `proved`/`illustrated` from the kernel
layer (those labels belong to later layers, if ever).

#### Scenario: Label matches producer
- **WHEN** a finding is produced by kernel execution of an accepted
  equation
- **THEN** its label is `executed`; a finding raised as a domain question
  (proposed equation, parallel paths) is `hypothesized`

### Requirement: Findings serialize stably and order deterministically
The system SHALL encode findings as stable machine-readable JSON and
SHALL order findings deterministically (by kind, then witness ids) so
that repeated checks of the same model produce byte-identical finding
lists.

#### Scenario: Repeated runs are diff-stable
- **WHEN** the same model is checked twice in the same process or across
  processes
- **THEN** the serialized finding lists are byte-identical
