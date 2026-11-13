# analysis Specification

## Purpose
Bounded, deterministic question-answering over a model document: whether
constraints can be satisfied at all (witness or proven contradiction),
whether a declaration is derivable, which parallel paths were never
reconciled, and where two models differ. Every answer is scope-honest —
`UNKNOWN` carries the exact explored scope instead of an unbounded claim.

## Requirements

### Requirement: Bounded search returns a witness or an honest UNKNOWN
The system SHALL provide bounded instance search with caller-supplied
bounds, returning a satisfying instance (witness), a proven-unsatisfiable
verdict (contradiction finding, when exhaustion completes within bounds),
or `UNKNOWN` with the exact scope explored — never an unbounded claim,
never a naked "no violation found" implying exhaustion. A zero-size
bound yields `UNKNOWN` with the empty scope explored; it is never
reported as a proven contradiction.

#### Scenario: Contradiction proven within bounds
- **WHEN** the search exhausts the declared bound and finds no instance
  satisfying the accepted equations
- **THEN** the outcome is a `contradiction` finding with the bound restated

#### Scenario: Budget exhaustion is UNKNOWN, not success
- **WHEN** the bound expires before exhaustion
- **THEN** the outcome is `UNKNOWN` labeled with the explored scope —
  never collapsed into "no contradiction found"

#### Scenario: Accepted equations constrain, proposed equations question
- **WHEN** the search evaluates a candidate against a schema carrying
  both accepted and proposed equations
- **THEN** only the accepted equations judge satisfiability — a
  candidate the accepted equations admit is clean even when it fails a
  proposed equation, the failure surfaces as a `hypothesized` finding,
  and a proposed-equation violation never blocks a witness or turns the
  outcome into a contradiction



### Requirement: Redundancy is proven by derivation
The system SHALL report a path (or declared equation) as `redundant` only
when normal-form rewriting under the declared equations proves it equal
to another path, carrying both the redundant path and the reducing path
as witnesses.

#### Scenario: Derivable path
- **WHEN** path `p` rewrites to declared path `q` under the equation set
- **THEN** a `redundant` finding names `p` with `by: q`; absent a proof,
  no redundancy claim is emitted

### Requirement: Underconstraint is raised as a hypothesis
The system SHALL detect parallel paths (identical endpoints, no declared
equation relating them) and emit `underconstraint` findings labeled
`hypothesized`, phrased as the domain question "should these agree?" —
never as violations.

#### Scenario: Parallel paths without equation
- **WHEN** two paths share endpoints and no equation relates them
- **THEN** an `underconstraint` finding names both paths and carries the
  hypothesis label

### Requirement: Distinguish exhibits its witness
The system SHALL decide bounded behavioral distinguishability of two
instances over an explicit observation set, returning `distinguishable`
with a witness input where they differ, or `equivalent-in-scope` — the
observation set is part of the result, and schema-equivalence claims are
never made.

#### Scenario: Witnessed difference
- **WHEN** two instances differ on some observation within the declared
  set
- **THEN** the result names the differing observation and the two values

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
