# analysis delta (P4)

## ADDED Requirements

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


#### Scenario: accepted-equations-constrain-proposed-equations-question
- **WHEN** [describe the action or condition]
- **THEN** [describe the expected observable result]
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
