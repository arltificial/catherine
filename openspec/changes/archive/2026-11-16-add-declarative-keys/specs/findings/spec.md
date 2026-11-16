# findings delta (declarative keys)

## MODIFIED Requirements

### Requirement: Findings are typed and carry witnesses
The system SHALL represent every diagnostic as a `Finding` value whose
variant names its kind (violation, dangling, redundant, contradiction,
underconstraint) and whose payload carries the concrete witness that
demonstrates it. A violation's witness SHALL be one of: the equation
witness (equation id + row + both path values) or the key witness (key
index + the two colliding rows). Other kinds carry their fixed witness
(arrow + source row; derivable path + reducing path; explicit scope; the
domain-question witness: `ParallelPaths{lhs, rhs}` or
`UncoveredInputs{class, examples}`). A finding without a witness SHALL be
unrepresentable. The kind set SHALL stay exactly these five — a key
collision is violation-shaped, never a sixth kind. The equation-witness
JSON encoding SHALL remain byte-identical to its pre-key form; the key
witness SHALL encode as `{"key": n, "rows": [lo, hi]}`.

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