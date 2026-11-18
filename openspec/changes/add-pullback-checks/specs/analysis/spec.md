# analysis delta (pullbacks in the enforced set)

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
  accepted and proposed equations, keys, coproducts, or pullbacks
- **THEN** only the accepted constraints (equations, keys,
  coproducts, and pullbacks) judge satisfiability — a candidate the
  accepted constraints admit is clean even when it fails a proposed
  equation, key, coproduct, or pullback, the failure surfaces as a
  `hypothesized` finding, and a proposed-constraint violation never
  blocks a witness or turns the outcome into a contradiction
