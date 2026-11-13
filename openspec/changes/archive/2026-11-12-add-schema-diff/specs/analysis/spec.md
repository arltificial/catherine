# analysis delta (P10)

## ADDED Requirements

### Requirement: Schema edits are classified on the enforced constraints
The system SHALL classify a schema edit v1 → v2 by diffing the enforced
equation sets — the accepted equations' `(lhs, rhs)` pairs — as
`cosmetic` (no enforced-constraint change), `accreted` (constraints only
added), `weakened`, or an equivalence-scoped classification; equation
rationale, document order, and proposed equations are presentation and
SHALL NOT classify an edit.

#### Scenario: Presentation-only edit is silent
- **WHEN** v2 differs from v1 only in equation rationale or document
  order
- **THEN** the classification is `cosmetic` and it emits zero findings

#### Scenario: Constraints only added
- **WHEN** every equation v1 enforces is still enforced in v2 and v2
  adds enforced equations
- **THEN** the classification is `accreted` naming the added equations,
  with zero findings — adding a constraint never widens the instance
  space

### Requirement: Weakening is caught with a witness
The system SHALL report `weakened` only when a v1-enforced equation is
no longer enforced in v2 and a bounded search exhibits an instance that
v2 accepts and v1 rejects; the finding SHALL be the existing
`violation` kind labeled `executed`, carrying the violated v1 equation's
canonical index and the composed values at its row. No new finding kind
is introduced.

#### Scenario: Silent constraint-weakening detected
- **WHEN** v2 drops (or weakens) an equation v1 enforced and a
  fully-resolved bounded instance satisfies every accepted v2 equation
  while violating the dropped v1 equation
- **THEN** the classification is `weakened` with exactly one
  `violation`/`executed` finding, and the witness instance re-checks
  clean under v2 and rejected under v1

### Requirement: Equivalence is scope-bounded and never silent
The system SHALL classify `equivalent-in-scope` only after the bounded
enumeration of fully resolved instances exhausts without a separating
instance, carrying the explored scope (per-object row counts and
candidates enumerated) in the result. A budget that expires before
exhaustion SHALL be `unknown` with the exact scope, never equivalence;
a structural difference (objects or arrows) SHALL be `incomparable` and
SHALL NOT be guessed across vocabularies.

#### Scenario: Refactor proven equivalent in scope
- **WHEN** two schemas enforce the same instance space and the bounded
  enumeration finds no separating instance
- **THEN** the classification is `equivalent-in-scope` with the scope
  echoed and zero findings

#### Scenario: Budget expiry is UNKNOWN
- **WHEN** the candidate budget expires before the space is exhausted
- **THEN** the classification is `unknown` with the exact scope explored
  — it is never reported as equivalence

#### Scenario: Structural change is refused
- **WHEN** the two schemas differ in objects or arrows
- **THEN** the classification is `incomparable` — instance spaces across
  different vocabularies require a declared schema map, not a guess