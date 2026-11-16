# findings delta (coproduct witness)

## MODIFIED Requirements

### Requirement: Witness shapes are fixed per finding kind
Each kind's witness shape SHALL be fixed: a violation carries one of the
equation witness (equation id, row, both path values), the key witness
(key index, both colliding rows), or the coproduct witness (coproduct
index, row, and the covering injections exactly when the row lies in more
than one injection image); dangling carries arrow and source row;
redundant carries derivable and reducing path; contradiction carries
explicit scope; underconstraint carries a domain question
(`ParallelPaths{lhs, rhs}` or `UncoveredInputs{class, examples}`). The
equation-witness and key-witness JSON encodings SHALL remain
byte-identical to their pre-coproduct forms; the coproduct witness SHALL
encode as `{"coproduct": n, "row": r}` for a coverage failure and as
`{"coproduct": n, "row": r, "injections": [i1, i2]}` for a disjointness
failure, with `injections` present exactly when the row lies in more than
one injection image. No new finding kind is introduced.

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

#### Scenario: Every finding kind has a golden fixture
- **WHEN** the fixture suite runs
- **THEN** each v1 finding kind has a golden fixture whose assertions
  check witness fields structurally, including both violation witness
  shapes (producers for analysis kinds are exercised via P4, but the
  taxonomy and fixtures exist here)