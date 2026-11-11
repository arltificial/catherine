# findings delta (P2)

## ADDED Requirements

### Requirement: Findings are typed and carry witnesses
The system SHALL represent every diagnostic as a `Finding` value whose
variant names its kind (violation, dangling, redundant, contradiction,
underconstraint) and whose payload carries the concrete witness that
demonstrates it (equation id + row + both path values; arrow + source
row; derivable path + reducing path; explicit scope; the domain-question
witness: `ParallelPaths{lhs, rhs}` or `UncoveredInputs{class, examples}`).
A finding without a witness SHALL be unrepresentable.

#### Scenario: Violation finding carries row and values
- **WHEN** a kernel check produces a violation
- **THEN** the finding is `Violation` carrying the equation id, the
  offending row id, and both composed path values — assertable
  structurally, not by output text

#### Scenario: Every finding kind has a golden fixture
- **WHEN** the fixture suite runs
- **THEN** each v1 finding kind has a golden fixture whose assertions
  check witness fields structurally (producers for analysis kinds are
  exercised via P4, but the taxonomy and fixtures exist here)

### Requirement: Every finding states how it was established
The system SHALL attach an epistemic label to every finding — `executed`
for kernel-computed results, `hypothesized` for proposed-equation
failures and underconstraint questions, `unknown` for scope-bounded
non-answers — and SHALL never emit `proved`/`illustrated` from the kernel
layer (those labels belong to later layers, if ever).

#### Scenario: Label matches producer
- **WHEN** a finding is produced by kernel execution of an accepted
  equation
- **THEN** its label is `executed`; a finding raised as a domain question
  (proposed equation, parallel paths) is `hypothesized`

### Requirement: Findings serialize stably and order deterministically
The system SHALL encode findings as stable machine-readable JSON and
SHALL order findings deterministically (by kind, then witness ids) so
that repeated checks of the same model produce byte-identical finding
lists.

#### Scenario: Repeated runs are diff-stable
- **WHEN** the same model is checked twice in the same process or across
  processes
- **THEN** the serialized finding lists are byte-identical
