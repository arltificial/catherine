# lean-kernel-model delta

## ADDED Requirements

### Requirement: The Lean model mirrors the kernel semantics as an external artifact
The system SHALL maintain a Lean 4 reference semantics of the kernel
(schema, typed instance with total typing-enforcing build, path composition
with dangling short-circuit, the `check` walk, the findings taxonomy with
the three constructible labels) as an external artifact under `lean/`. The
Cargo workspace SHALL gain no dependency on Lean; `parse` and malformed-
input rejection stay Rust-side.

#### Scenario: The twin builds standalone on a mirror document
- **WHEN** `lake build` runs in `lean/` on a machine without the Rust
  toolchain, and a mirror-format presentation (the oracle's normalized
  JSON) is fed to the Lean `check`
- **THEN** the Lean project builds and produces a `check` result over the
  presentation, with no Cargo crate referencing Lean and vice versa

### Requirement: The parity oracle gates the twins on the fixture corpus
The system SHALL run a parity oracle over the fixture corpus comparing
Lean and Rust `check` results as normalized finding multisets (kind +
witness + label, not byte output).

#### Scenario: Parity holds across the check-applicable fixture corpus
- **WHEN** the `lean-parity` harness runs Lean `check` and Rust `catherine
  check --json` over every check-applicable fixture document (enumerated
  dynamically — katas, parity, probes check inputs)
- **THEN** the normalized finding multisets agree exactly, and the oracle
  exits green

#### Scenario: Divergence is loud and attributed
- **WHEN** a fixture produces differing findings between the twins
- **THEN** the oracle fails, naming the fixture and the attributed side
  (Lean-model defect or Rust kernel defect with its beads ticket), and
  parity task 6 stays unchecked — the change does not archive — until the
  divergence is resolved

### Requirement: Checker soundness is proven
The Lean model SHALL prove that every finding its `check` emits carries a
semantic witness: a violated equation on an applicable row, an unresolved
cell accounted by a dangling report, a duplicate resolved key tuple, or a
coproduct coverage/disjointness failure — witness extraction is total over
emitted findings.

#### Scenario: No witnessless finding is representable
- **WHEN** the soundness theorem is applied to any finding of the Lean
  `check` result
- **THEN** a semantic violation matching the finding's witness is
  extracted, and the theorem compiles `sorry`-free

### Requirement: Equation completeness is proven with label agreement
The Lean model SHALL prove that every semantic equation violation on an
applicable row is reported as a violation finding, labeled `executed` when
the constraint status is accepted and `hypothesized` when proposed — one
kind, countable apart by label, never merged into one severity (per
`kernel-checks` and `findings`).

#### Scenario: Every violation is found and weighted correctly
- **WHEN** an instance contains a semantic equation violation on an
  applicable row under either constraint status
- **THEN** the completeness theorem yields a matching finding in the
  `check` result carrying the status-agreed label

### Requirement: Dangling discipline is proven
The Lean model SHALL prove the dangling contract: dangling cells are
representable values that are never repaired, never reported as violations,
never grouped into key collisions, and never used to decide coproduct
coverage; a typing-violated cell is accounted exactly once (violation
report, its `None` cell is a build report, not a dangling finding).

#### Scenario: A dangling cell produces exactly its dangling finding
- **WHEN** an arrow cell is unresolved while other cells on the same row
  satisfy their equations
- **THEN** the discipline lemmas yield one dangling finding naming the
  arrow and row, no violation finding, and no key-collision or coverage
  participation for that row

### Requirement: The stop gate records the assurance outcome
The system SHALL record the proof phase's outcome in the ledger at the
stop gate — extend (search/migration follow-up), hold (maintain at landed
scope), or retire (archive the Lean tree) — with what the proofs bought
(defects, holes) versus fixture-only assurance; the outcome SHALL be a
recorded decision, never a silent drift into unmaintained state.

#### Scenario: The gate decision is on the ledger
- **WHEN** phase-2 proofs land and the gate review runs
- **THEN** the ledger contains the extend/hold/retire decision with the
  cost/benefit accounting, linked from the Lean README
