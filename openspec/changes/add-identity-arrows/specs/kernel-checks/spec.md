# kernel-checks delta (identity paths)

## MODIFIED Requirements

### Requirement: Every declared equation is executed on every applicable row
The system SHALL evaluate each `accepted` path equation against every row
of the equation's domain object, and SHALL emit a violation outcome with
a witness (the offending row id and both composed path values) whenever
the paths disagree — enforcement, not representation (E3). An equation
side SHALL be permitted to be the empty path — the **explicit identity**
on the equation's domain object — with the domain derived from the
non-empty side's source object. An equation with the empty path on both
sides has no object and SHALL be rejected as ill-typed at the equation's
node: identity is explicit, never inferred, and an ambiguous or
unresolvable path is never defaulted to identity.

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

#### Scenario: One-sided-empty equation executes on its derived domain
- **WHEN** an accepted equation has the empty path on one side (e.g.
  `[realize, abstract] = []`) and a non-empty side whose source object
  is `O`
- **THEN** the equation's domain object is `O`, and the equation is
  evaluated against every row of `O` — the empty path composes to the
  row it starts at, so a row violates exactly when the non-empty path
  does not return it to itself

#### Scenario: Both sides empty is rejected, never defaulted
- **WHEN** an equation declares the empty path on both sides
- **THEN** parsing fails with an ill-typed-equation error at the
  equation's node — the equation has no object, so it is never accepted
  as a vacuous identity or defaulted to "always true"

#### Scenario: Identity composition evaluates as the identity on its domain
- **WHEN** an instance row of object `O` satisfies an accepted
  `[realize, abstract] = []` law (the composed path returns the row to
  itself), and some other row does not
- **THEN** the satisfying row emits nothing and the failing row emits one
  violation with the identity side's witness value equal to the row's
  own id — the identity is executed, not assumed

#### Scenario: Unresolvable paths are not quietly identity
- **WHEN** a non-empty side of an equation traverses a dangling arrow
  value
- **THEN** the affected rows are reported as
  not-evaluable-through-dangling (the existing dangling rule) — the
  unresolvable path is never treated as if it were the identity

#### Scenario: Format-1.0 documents parse byte-identically
- **WHEN** a document that uses no identity path — at any known minor,
  including 1.0 — is parsed and canonicalized after this change
- **THEN** its canonical bytes and hash are unchanged (hash-neutral), and
  the `KNOWN_MINOR` version bump is the only rejection boundary added —
  no per-feature version floor and no new rejection of previously
  accepted documents
