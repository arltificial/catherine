# instance-morphism Specification

## Purpose
Verifies a given row mapping between two instances of the same schema: the
source document's declared `instance_map` is executed as a commutation claim,
so a mapping that violates the structure is one witnessed violation instead
of an unstated assumption. The surface checks a map — it never synthesizes
one and emits no merged document.

## Requirements

### Requirement: A claimed instance map is checked for commutation
The system SHALL provide an operation over two documents of identical
schema (source and target) that executes the source document's declared
`instance_map`: for every schema arrow and every mapped source row, the
compositions `map ∘ arrow` and `arrow ∘ map` SHALL be compared, and
each mismatch SHALL produce one `violation` finding with an `executed`
label. A square undefined on both sides SHALL be warned, never reported
as a silent, warning-free clean. No new finding kinds are introduced.

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

#### Scenario: Unresolved square is a non-answer
- **WHEN** both `map(a(x))` and `a(map(x))` are undefined for some arrow
  `a` and mapped source row `x` (a dangling cell lies on each side)
- **THEN** the run emits one labeled `unresolved_naturality_square`
  warning naming the arrow and the source row, produces no finding for
  that square, and exits 0 unless there are violations — never a
  warning-free clean that claims the square was verified; a square
  undefined on only one side remains a `violation` with the unresolved
  composed value as a null witness side

### Requirement: Malformed naturality claims are rejected before checks
The operation SHALL require identical schemas (the existing
`schema_mismatch` error), and SHALL fail with labeled errors — never
findings — when the declared map is structurally unsound: a map cell
naming a target row that does not exist fails before any check runs,
never silently followed.

#### Scenario: Same document twice is trivially clean
- **WHEN** the source and target paths name the same document whose
  declared `instance_map` is the identity
- **THEN** every commutation square holds by construction, the operation
  reports zero findings, and exits clean — accepted, not an error; a
  same-document run whose map targets rows of another instance is a
  construction-time `unknown_map_target`, not a degenerate clean

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

### Requirement: An absent instance map warns instead of a silent clean
When the source document declares no `instance_map`, the operation SHALL
emit a labeled envelope warning naming the absent map and SHALL exit 0 —
never a warning-free success indistinguishable from an executed, verified
map. The absence warns; it is not a malformed claim and produces no
labeled error. No finding kind or error code is introduced.

#### Scenario: Absent map warns, never clean
- **WHEN** the source document declares no `instance_map`
- **THEN** the run exits 0 with a success envelope whose `warnings`
  array carries a labeled warning naming the absent `instance_map` —
  never a silent, warning-free clean that looks like a verified map
