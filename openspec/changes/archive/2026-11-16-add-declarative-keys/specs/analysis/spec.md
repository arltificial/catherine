# analysis delta (declarative keys)

## MODIFIED Requirements

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
  satisfying the accepted constraints
- **THEN** the outcome is a `contradiction` finding with the bound restated

#### Scenario: Budget exhaustion is UNKNOWN, not success
- **WHEN** the bound expires before exhaustion
- **THEN** the outcome is `UNKNOWN` labeled with the explored scope —
  never collapsed into "no contradiction found"

#### Scenario: Accepted equations constrain, proposed equations question
- **WHEN** the search evaluates a candidate against a schema carrying
  accepted and proposed equations or keys
- **THEN** only the accepted constraints (equations and keys) judge
  satisfiability — a candidate the accepted constraints admit is clean even
  when it fails a proposed equation or key, the failure surfaces as a
  `hypothesized` finding, and a proposed-constraint violation never blocks a
  witness or turns the outcome into a contradiction

### Requirement: Schema edits are classified on the enforced constraints
The system SHALL classify a schema edit v1 → v2 by diffing the enforced
constraint sets — the accepted equations' `(lhs, rhs)` pairs together with
the accepted keys' `(object, arrows)` tuples — as `cosmetic` (no
enforced-constraint change), `accreted` (constraints only added),
`weakened`, or an equivalence-scoped classification; constraint rationale,
document order, and proposed equations or keys are presentation and SHALL
NOT classify an edit.

#### Scenario: Presentation-only edit is silent
- **WHEN** v2 differs from v1 only in equation rationale, document
  order, or a proposed equation or key
- **THEN** the classification is `cosmetic` and it emits zero findings

#### Scenario: Constraints only added
- **WHEN** every equation or key v1 enforces is still enforced in v2 and v2
  adds enforced constraints, including an accepted key
- **THEN** the classification is `accreted` naming the added constraints,
  with zero findings — adding a constraint never widens the instance
  space

### Requirement: Weakening is caught with a witness
The system SHALL report `weakened` only when a v1-enforced constraint is no
longer enforced in v2 and a bounded search exhibits an instance that v2
accepts and v1 rejects; the finding SHALL be the existing `violation` kind
labeled `executed`, carrying the violated v1 constraint — the equation's
canonical index and the composed values at its row, or the key's canonical
index and its two colliding rows. No new finding kind is introduced.

#### Scenario: Silent constraint-weakening detected
- **WHEN** v2 drops (or weakens) an equation or an accepted key v1 enforced
  and a fully-resolved bounded instance satisfies every accepted v2
  constraint while violating the dropped v1 constraint
- **THEN** the classification is `weakened` with exactly one
  `violation`/`executed` finding whose witness names the lost v1 constraint,
  and the witness instance re-checks clean under v2 and rejected under v1