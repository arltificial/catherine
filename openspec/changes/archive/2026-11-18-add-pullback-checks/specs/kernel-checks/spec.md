# kernel-checks delta (executed pullback checks)

## ADDED Requirements

### Requirement: Declared pullbacks are executed as commutes, covers, and unique
The system SHALL evaluate each `accepted` pullback against the
instance as three verdicts — **commutes** (`f∘p1 = g∘p2` on every
apex row), **covers** (every fiber pair of the cospan is projected by
some apex row), and **unique** (no two apex rows project to the same
pair) — emitting one witness-bearing violation per offending row or
pair. `proposed` pullbacks SHALL follow the equation status rule:
their failures are hypothesis-labeled outcomes, never merged with
violations.

#### Scenario: Non-commuting apex row carries both composed values
- **WHEN** an apex row's `f∘p1` and `g∘p2` compose to different rows
- **THEN** checking emits one violation outcome naming the pullback,
  the row id, and both composed path values

#### Scenario: Missing fiber pair is a coverage violation
- **WHEN** two resolved rows `a` and `b` compose to the same cospan
  target but no apex row projects to `(a, b)`
- **THEN** checking emits one violation outcome naming the pullback
  and the uncovered pair — in canonical row order, countable and
  diff-stable

#### Scenario: Duplicated projection is a uniqueness violation
- **WHEN** two distinct apex rows project to the same pair `(a, b)`
- **THEN** checking emits one violation outcome naming the pullback
  and both rows — the legs are not jointly injective, so the apex is
  not the pullback even though it commutes and covers

#### Scenario: Dangling cells keep verdicts undecidable, never silent
- **WHEN** an apex row's leg cell dangles, or a fiber pair's cospan
  cell dangles
- **THEN** the affected verdicts are reported as undefined rather than
  violated — the existing dangling findings carry the offending rows
  and are never double-reported — and a run with only undefined
  verdicts is never presented as a verified clean

#### Scenario: Evaluated silence is real success
- **WHEN** every apex row commutes, every resolvable fiber pair is
  covered, and no pair is projected twice
- **THEN** checking returns zero outcomes for the pullback — including
  the zero-row apex over an empty fiber, whose check ran and
  trivially held
