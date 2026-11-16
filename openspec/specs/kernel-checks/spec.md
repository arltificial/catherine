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

### Requirement: Declared keys are executed as injectivity checks
The system SHALL evaluate every declared key against the rows of its object
and SHALL emit one `violation` finding per unordered pair of rows whose key
arrow tuple agrees — `executed` for an accepted key, `hypothesized` for a
proposed one. A row with an unresolved cell on a key arrow SHALL be
excluded from grouping: that cell is already reported by its own finding,
never double-reported. The witness SHALL name the key's canonical index and
both colliding rows.

#### Scenario: Colliding tuple carries its witness
- **WHEN** two rows of a key's object agree on every key arrow's value
- **THEN** checking emits exactly one `violation` for that pair, naming the
  key index and both row ids (ascending), labeled `executed` for an accepted
  key and `hypothesized` for a proposed one

#### Scenario: Three-way collision is three pairs
- **WHEN** three rows agree on the key tuple
- **THEN** checking emits three findings, one per unordered pair — each pair
  is a demonstrated failure with an exact witness, not a summary count

#### Scenario: Dangling key cell is not a collision
- **WHEN** a row's key arrow value is dangling
- **THEN** checking emits the dangling finding for that cell and no key
  violation involving that row

#### Scenario: Clean key emits nothing
- **WHEN** every row of the key's object resolves its key arrows to distinct
  tuples
- **THEN** checking emits no key finding — repeated checks serialize
  byte-identically under the taxonomy's canonical order

### Requirement: Accepted coproducts execute coverage with witnesses
The system SHALL execute accepted coproduct declarations against the
instance: a row of the declared object in no injection image SHALL
produce one `violation` finding labeled `executed` naming the uncovered
row. Proposed coproducts SHALL produce the same findings labeled
`hypothesized`. No new finding kinds are introduced.

#### Scenario: Rogue variant is a violation with a witness
- **WHEN** an instance carries a row of the declared object outside every
  injection image (the natural data evolution the declaration exists to
  catch)
- **THEN** exactly one `violation`/`executed` finding names the row, and
  the run exits with the violation code — never a clean exit on a
  declared-closed type

#### Scenario: Clean closed table is silent
- **WHEN** every row of the declared object is in exactly one injection
  image
- **THEN** zero coproduct findings are produced

### Requirement: Coproduct disjointness violations name the colliding injections
The system SHALL produce one `violation` finding labeled `executed` for
each row of the declared object that lies in the image of two declared
injections, with a witness naming the row and both injections. An
injection with a dangling or typing-violated cell has a partial image, so
coverage and disjointness are undecidable for that declaration: its
coproduct findings SHALL be suppressed, because the unresolved cell is
already reported by its own dangling finding — never double-reported.

#### Scenario: Dual coverage is a violation naming both injections
- **WHEN** a row lies in the image of two declared injections
- **THEN** exactly one `violation`/`executed` finding names the row and
  both injections as the disjointness witness

#### Scenario: Dangling injection cells are not double-reported
- **WHEN** a variant row's injection cell is dangling
- **THEN** only the dangling finding is produced — the declaration's
  coverage and disjointness findings are suppressed (the partial image
  cannot decide them), so the row is never double-reported
