# kani-verification-lane delta

## ADDED Requirements

### Requirement: A Kani verification lane exists, separate from the CI gate
The system SHALL provide a `verify` lane that runs the Kani proof
harnesses over the kernel crates, invocable as `just verify`. The standard
CI gate (`just ci`) SHALL NOT require the Kani toolchain or run any proof
harness — the lane mirrors the Lean gate's separation: a toolchain a fresh
checkout cannot assume must not gate every commit.

#### Scenario: verify runs the harnesses without touching the ci gate
- **WHEN** `just verify` runs on a machine with the pinned Kani release
  provisioned
- **THEN** every declared harness executes and the recipe exits nonzero if
  any harness returns a confirmed counterexample, a harness error, or an
  undetermined outcome without a recorded disposition

#### Scenario: ci stays Kani-free
- **WHEN** `just ci` runs on a checkout without Kani installed
- **THEN** it succeeds or fails exactly as before this change — no recipe
  in its transitive closure invokes `cargo kani`

### Requirement: The Kani toolchain is pinned and reprovisionable
The verification lane SHALL pin the Kani release (version recorded in the
repo, same discipline as `rust-toolchain.toml` pins the workspace
toolchain) with provisioning instructions, so a fresh machine or CI runner
reproduces the exact checker. The workspace's Rust toolchain pin SHALL
remain untouched — Kani runs under its own bundled toolchain.

#### Scenario: Fresh-machine provisioning reproduces the lane
- **WHEN** a fresh machine follows the documented provisioning step
- **THEN** `just verify` runs against the pinned Kani release without
  modifying the workspace toolchain or adding a runtime dependency

#### Scenario: Recorded proofs name the Kani release that produced them
- **WHEN** the lane README records a proof result
- **THEN** the record names the Kani release that produced it, so
  recorded proofs stay reproducible across later pin bumps

### Requirement: Kernel execution paths are proven panic-free within bounds
The lane SHALL include harnesses proving that the kernel's post-parse
execution path — typed-instance build, path composition, and the `check`
walk including key, coproduct, and pullback checks — performs no panic,
arithmetic overflow, or out-of-bounds access over nondeterministic bounded
model documents constructed from typed domain values. Declared bounds
SHALL be non-vacuous: the bounded space admits at least one document that
exercises each check family.

#### Scenario: Adversarial bounded models cannot crash the kernel
- **WHEN** a panic-freedom harness explores the bounded space of
  nondeterministic `Schema`/`RawInstance` values within the declared
  bounds
- **THEN** the harness proves the absence of panics and overflow within
  those bounds, or produces a concrete counterexample input under the
  counterexample protocol

### Requirement: Witness soundness is proven on the shipped Rust
The lane SHALL include a harness family proving that every finding the
Rust `check` emits carries a genuine witness: re-evaluating the witness
against the model with an independent re-check fails in the way the
finding claims — the Rust counterpart of the Lean twin's soundness
theorem, verified on the artifact agents invoke rather than the Lean
mirror.

#### Scenario: Every emitted finding's witness re-fails on re-check
- **WHEN** the witness-soundness harness explores bounded model documents
  whose `check` results emit findings
- **THEN** each emitted finding's witness, independently re-evaluated,
  exhibits the claimed violation, and the harness proves this universally
  within bounds or produces a counterexample under the protocol

### Requirement: Every harness reports a three-valued outcome
Each harness SHALL report exactly one of: **proved** (within the declared
bounds), **counterexample** (with a concrete failing input), or
**undetermined** (with the unwinding/bound reason). A run SHALL NOT drop,
skip, or silently pass a harness, and the lane's aggregate result SHALL
name any harness that did not prove.

#### Scenario: Undetermined is loud, never green
- **WHEN** a harness cannot complete within its unwinding bounds
- **THEN** the lane reports that harness as undetermined with the bound
  reason, and `just verify` does not present the family as proven

#### Scenario: Undetermined without a disposition fails the lane
- **WHEN** a harness ends undetermined and no disposition (a raised bound
  or a split obligation, recorded in the lane README) exists for it
- **THEN** the recipe exits nonzero, naming the harness and its bound
  reason

### Requirement: Counterexamples are attributed, never silenced
A confirmed counterexample SHALL be resolved as exactly one of: a **harness
defect** (the harness misstates the obligation) or a **Rust kernel defect**
(a real bug — fixed with a beads ticket and a regression fixture through
its own change, not by editing this lane's scope). The attribution and the
resolution ticket SHALL be recorded, mirroring the Lean parity oracle's
divergence protocol.

#### Scenario: A kernel defect found by a harness has a home
- **WHEN** a harness produces a counterexample whose input genuinely
  triggers a kernel panic or a witnessless finding
- **THEN** the failure is filed as a kernel-defect beads ticket with the
  counterexample input attached, and the harness obligation is not weakened
  to make the run green

### Requirement: The lane records a stop-gate decision in the ledger
After the first two harness families land (end of phase 1), the lane
SHALL record one of **extend** / **hold** / **retire** in the ledger as a
terminal `decision` entry (the existing kind validated by
`catherine-ledger`), stating what the harnesses bought (defects found,
bounds reached) versus fixture-and-property assurance alone. The verdict
gates the remaining harness spend.

#### Scenario: The verdict gates the remaining harness spend
- **WHEN** the gate review runs after the first two harness families land
- **THEN** extend funds the phase-2 harness families and hold maintains
  at phase-1 scope — neither runs harness work the verdict did not fund

#### Scenario: Retire archives the lane with lessons kept
- **WHEN** the gate verdict is retire
- **THEN** the harness modules and the verify recipe are archived, with
  the lessons kept in the lane README and the ledger entry

#### Scenario: The decision is sealed either way
- **WHEN** the stop-gate review runs
- **THEN** a `decision` ledger entry exists for the Kani lane and
  `ledger-check ledger/` accepts it — extend, hold, and retire are all
  valid outcomes; an unrecorded drift into unmaintained harnesses is not
