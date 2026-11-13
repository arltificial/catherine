# kernel-checks Specification

## Purpose
Executed checks over instance tables: every declared equation runs on
every applicable row and every declared constraint is enforced, with a
witness for each failure. Declarations alone do not enforce — the
executed check is the product.

## Requirements

### Requirement: Every declared equation is executed on every applicable row
The system SHALL evaluate each `accepted` path equation against every row
of the equation's domain object, and SHALL emit a violation outcome with
a witness (the offending row id and both composed path values) whenever
the paths disagree — enforcement, not representation (E3).

#### Scenario: Violation carries its witness
- **WHEN** an instance row makes an accepted equation's lhs and rhs paths
  compose to different rows
- **THEN** checking emits a violation outcome naming the equation, the
  row id, and both path values — one outcome per violating row, not one
  per equation

#### Scenario: Consistent row emits nothing
- **WHEN** every row satisfies every accepted equation
- **THEN** checking returns zero outcomes — silence is real success, not
  an unevaluated declaration

### Requirement: Equation status decides the epistemic weight of outcomes
The system SHALL treat `accepted` equation failures as violations and
`proposed` equation failures as hypothesis-labeled outcomes — the two
SHALL be distinguishable in the output, never merged into one severity.

#### Scenario: Proposed equation failure is a question, not a violation
- **WHEN** a `proposed` equation fails on a row
- **THEN** the outcome is labeled as hypothesis/proposed — a domain
  question under review — and is countable separately from violations

### Requirement: Dangling values are checked, not fatal
The system SHALL classify an arrow value that points at no existing row
as a dangling outcome (with the arrow and source row), and SHALL
short-circuit path composition through a dangling value for equation
evaluation — a dangling reference on one arrow does not crash or
silently skip the whole check run.

#### Scenario: Dangling arrow value
- **WHEN** a row's arrow value references a nonexistent row id
- **THEN** checking emits a dangling outcome (arrow, source row) and
  equations traversing that arrow report the affected rows as
  not-evaluable-through-dangling rather than as violations

### Requirement: The instance builder is total and typing-enforcing
The system SHALL store every parsed arrow value as a morphism value, a
dangling record, or a typing violation — none silently dropped — and a
resolved reference to a row of the wrong object SHALL be recorded as a
typing violation, not stored (adapted acset-core `adapter_total`,
`typing_enforced_at_build`).

#### Scenario: Typing violation at build
- **WHEN** an instance assigns an arrow value naming a row of an object
  that is not the arrow's target
- **THEN** the build records a typing violation naming the row and arrow,
  and the value does not become a morphism table entry

#### Scenario: Parity with the donor implementation
- **WHEN** the ported instance tables and checks run over the specodelic
  corpus representations (parity gate)
- **THEN** findings match specodelic's acset behavior on every shared
  case before any rewrite of ported logic is merged
