# rule-lens Specification

## Purpose
Rule tables modelled directly: rules as subobjects of the input domain,
action as a required total function, and resolution as a declared policy.
Gap, overlap, shadow, and derivable rule-table defects become exact
findings, and simplification is a provably equivalent rewrite. Status:
hypothesized — validated against conventional practice, never assumed.

## Requirements

### Requirement: Rule tables are presentations with a declared policy
The system SHALL accept a `RulePresentation` — input domain X, rules
(decidable predicates + action + order), and an explicit resolution
policy (`PriorityOrder | FirstMatch | DeclaredDisjoint`) — and SHALL
apply no implicit precedence when policy is undeclared.

#### Scenario: Action assignment must be total
- **WHEN** a rule table's rule regions do not cover the declared input
  domain
- **THEN** analysis reports a gap finding naming the uncovered input
  class (via enumeration within bounds)

### Requirement: Rule-lens findings map onto the existing taxonomy
The system SHALL emit rule-lens diagnostics exclusively as existing
finding kinds: gap → underconstraint (hypothesized until totality is
declared); overlap under `DeclaredDisjoint` → violation; shadow →
redundant (witness: effective-region proof); derivable → redundant
(witness: the combining rules). No new finding variants are introduced.

#### Scenario: Shadow rule is redundant with proof
- **WHEN** rule i's effective region is empty under the policy
- **THEN** a redundant finding names rule i and derives its death from
  the earlier rules + policy

#### Scenario: Overlap is policy-dependent
- **WHEN** two rules' regions intersect
- **THEN** `DeclaredDisjoint` yields a violation on the intersecting
  inputs; `PriorityOrder`/`FirstMatch` yield no finding (policy resolves)

### Requirement: Rule-table equivalence is proven by witness
The system SHALL check equivalence of two rule tables as equality of the
induced total action on X within bounds, returning `equivalent-in-scope`
or a differing witness input — simplification claims are proven, never
asserted.

#### Scenario: Simplification proven
- **WHEN** a 3-rule table is replaced by 2 rules + precedence and no
  distinguishing witness exists in-domain
- **THEN** the equivalence check returns equivalent-in-scope with the
  bound stated

### Requirement: Rule discovery validates candidates against examples
The system SHALL check a candidate rule table (from agent or human)
against declared input→output examples: partition equivalence per
example, with uncovered examples emitted as gap findings — candidates are
proposed, the tool decides (C5 discipline).

#### Scenario: Uncovered example
- **WHEN** a candidate table assigns no action to a provided example
- **THEN** that example is reported as a gap finding
