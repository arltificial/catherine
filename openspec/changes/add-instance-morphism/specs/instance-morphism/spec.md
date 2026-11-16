# instance-morphism delta (naturality check)

## ADDED Requirements

### Requirement: A claimed instance map is checked for commutation
The system SHALL provide an operation over two documents of identical
schema (source and target) that executes the source document's declared
`instance_map`: for every schema arrow and every mapped source row, the
compositions `map ∘ arrow` and `arrow ∘ map` SHALL be compared, and
each mismatch SHALL produce one `violation` finding with an `executed`
label. No new finding kinds are introduced.

#### Scenario: Commuting map checks clean
- **WHEN** the declared map commutes with every arrow of the schema on
  the given instances
- **THEN** the operation reports zero findings and exits clean

#### Scenario: Naturality break is a violation with a witness
- **WHEN** the claimed map sends a source row to a target row such that
  `map(a(x))` and `a(map(x))` differ for some arrow `a` and row `x`
- **THEN** exactly one `violation`/`executed` finding is produced naming
  `a` and `x`, with both composed values as the witness, and the run
  exits with the violation code

### Requirement: Malformed naturality claims are rejected before checks
The operation SHALL require identical schemas (the existing
`schema_mismatch` error), and SHALL fail with labeled errors — never
findings — when the declared map is structurally unsound: a map cell
naming a target row that does not exist fails before any check runs,
never silently followed.

#### Scenario: Same document twice is trivially clean
- **WHEN** the source and target paths name the same document
- **THEN** every commutation square holds by construction, the operation
  reports zero findings, and exits clean — accepted, not an error

#### Scenario: Schemas must be identical
- **WHEN** the two documents declare different schemas
- **THEN** the operation fails with the existing `schema_mismatch`
  labeled error — never a guessed alignment or partial comparison

#### Scenario: Unknown target is a construction-time error
- **WHEN** a map cell names a target row that does not exist in the
  target instance
- **THEN** the operation fails with a labeled `unknown_map_target` error
  naming the object, row, and map cell — before any findings are
  produced
