# findings delta (pullback witness)

## MODIFIED Requirements

### Requirement: Witness shapes are fixed per finding kind
Each kind's witness shape SHALL be fixed: a violation carries the
equation, key, coproduct, or pullback witness; dangling carries arrow
and source row; redundant carries derivable and reducing path;
contradiction carries explicit scope; underconstraint carries a domain
question (`ParallelPaths{lhs, rhs}` or `UncoveredInputs{class,
examples}`). No new finding kind is introduced.

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

#### Scenario: Every finding kind has a golden fixture
- **WHEN** the fixture suite runs
- **THEN** each v1 finding kind has a golden fixture whose assertions
  check witness fields structurally, including every violation witness
  shape (producers for analysis kinds are exercised via P4, but the
  taxonomy and fixtures exist here)
