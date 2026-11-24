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
equation, key, coproduct, or pullback witness; dangling carries arrow
and source row; redundant carries derivable and reducing path;
contradiction carries explicit scope across the accepted constraint
families (equations, keys, coproducts, pullbacks); underconstraint
carries a domain question (`ParallelPaths{lhs, rhs}` or
`UncoveredInputs{class, examples}`). No new finding kind is introduced.

#### Scenario: Witness encodings are stable
- **WHEN** findings serialize
- **THEN** the equation witness encodes byte-identically to its pre-key
  form, the key witness encodes as `{"key": n, "rows": [lo, hi]}`, and
  the coproduct witness encodes as `{"coproduct": n, "row": r}` with
  `injections` omitted for a coverage failure and naming both injections
  for a doubly covered row, so the shapes remain distinguishable without
  consulting the schema

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

#### Scenario: Coproduct violation carries its row and injections
- **WHEN** a kernel check produces a coproduct violation
- **THEN** the finding is `Violation` carrying the coproduct's canonical
  index and the offending row id, plus both injection names when the row
  lies in two injection images — assertable structurally, not by output
  text

#### Scenario: Pullback violation carries the square and the offender
- **WHEN** a kernel check produces a pullback violation
- **THEN** the finding is `Violation` carrying the pullback's canonical
  index and, per verdict: the offending row id and both composed values
  (commutes), the uncovered pair of row ids (covers), or both
  duplicate-projecting row ids (unique) — assertable structurally, not
  by output text

#### Scenario: Contradiction scope names every enforced family
- **WHEN** bounded search exhausts and accepted keys, coproducts, or
  pullbacks are in force
- **THEN** the contradiction witness carries their canonical indices
  (`keys`, `coproducts`, `pullbacks`; empty families are omitted), so
  the scope is never empty for a contradiction those constraints
  enforced

#### Scenario: Every finding kind has a golden fixture
- **WHEN** the fixture suite runs
- **THEN** each v1 finding kind has a golden fixture whose assertions
  check witness fields structurally, including every violation witness
  shape (producers for analysis kinds are exercised via P4, but the
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

### Requirement: Findings payloads carry the execution scope
The findings payload SHALL carry `data.scope`, an object with exactly
three snake_case keys — `constraints_checked`, `rows_evaluated`,
`equations_over_zero_rows` — reporting what the producing walk actually
executed, so an empty `data.findings` array with a zero scope
(nothing was checked) is distinguishable from an empty array with a
non-zero scope (checked and clean). The field is part of the public
payload contract consumed by the wasm/web client.

#### Scenario: Empty findings with a zero scope is not a pass
- **WHEN** a findings payload serializes with an empty
  `data.findings` array and `data.scope` of all zeros
- **THEN** the payload states nothing was executed — the emptiness is
  attributable to a vacuous run, not to a verified clean result

#### Scenario: Clean pass is distinguishable from vacuity
- **WHEN** a findings payload serializes with an empty
  `data.findings` array and a non-zero `constraints_checked` and
  `rows_evaluated`
- **THEN** the payload states the walk ran over real constraints and
  rows and found nothing — a genuine clean result

#### Scenario: The web consumer binds to the scope
- **WHEN** the wasm boundary mirrors a findings payload to the web
  client
- **THEN** the mirrored payload carries `data.scope` with the same
  three keys, and the client can branch on it without parsing
  findings or warnings

### Requirement: The scope field is additive and stable
`data.scope` SHALL serialize deterministically alongside the existing
payload fields without changing any of them, their order-independent
encoding, or any exit code; repeated runs of the same document SHALL
produce byte-identical scopes.

#### Scenario: Repeated runs are diff-stable including scope
- **WHEN** the same document is checked twice
- **THEN** the serialized payloads, including `data.scope`, are
  byte-identical, and the pre-existing fields' bytes are unchanged
  from before this field existed
