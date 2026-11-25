# verus-verification-lane delta

## Purpose
Functional-correctness assurance over a Verus proof model of the kernel
checks: `requires`/`ensures` contracts discharged by the pinned Verus
release, with fixture parity pinning the model to the shipped Rust kernel
and the Verus guide corpus as a toolchain tripwire.

## ADDED Requirements

### Requirement: A Verus verification lane exists, separate from the CI gate
The system SHALL provide a `verify-verus` lane that runs the Verus proof
units over the kernel proof model, invocable as `just verify-verus`. The
standard CI gate (`just ci`) SHALL NOT require the Verus release or run
any proof unit — the lane mirrors the Lean and Kani gates' separation: a
toolchain a fresh checkout cannot assume must not gate every commit.

#### Scenario: verify-verus runs the proof units without touching the ci gate
- **WHEN** `just verify-verus` runs on a machine with the pinned Verus
  release and its required rustup toolchain provisioned
- **THEN** every declared proof unit executes and the recipe exits nonzero
  if any proof unit returns an error, or ends resource-exhausted without a
  recorded disposition

#### Scenario: ci stays Verus-free
- **WHEN** `just ci` runs on a checkout without Verus installed
- **THEN** it succeeds or fails exactly as before this change — no recipe
  in its transitive closure invokes the Verus driver

### Requirement: The Verus toolchain is pinned and reprovisionable
The lane SHALL pin the Verus release (tag and zip asset recorded in the
repo) and the rustup toolchain name the Verus driver requires, with
provisioning instructions, so a fresh machine or CI runner reproduces the
exact checker. The workspace's Rust toolchain pin SHALL remain untouched —
the Verus toolchain is installed alongside it under the exact name the
driver's shim requires.

#### Scenario: Fresh-machine provisioning reproduces the lane
- **WHEN** a fresh machine follows the documented provisioning steps
- **THEN** `just verify-verus` runs against the pinned Verus release
  without modifying the workspace toolchain, the Cargo build, or any
  runtime dependency

#### Scenario: Recorded proofs name the Verus release that produced them
- **WHEN** the lane README records a proof result
- **THEN** the record names the Verus release that produced it, so
  recorded proofs stay reproducible across later pin bumps

### Requirement: The kernel walk's functional correctness is proven on the model
Over the proof model, the lane SHALL establish: an output-equality
invariant tying the walk to the spec mirror (**out-eq**); every
applicable emission position appears in the mirror output
(**complete**); every mirrored position carries its emission
(**faithful**); positions ordered by emission key order (**sorted**).
Proof obligations SHALL be non-vacuous — preconditions admit at least
one instance exercising each clause.

#### Scenario: The quantifier contract holds within the model
- **WHEN** the walk-correctness proof family runs under the pinned Verus
  release
- **THEN** each proof unit reports verified against its stated contract,
  or the lane surfaces an error or resource-exhaustion under the outcome
  protocol

### Requirement: Model parity with the shipped Rust kernel is enforced by fixtures
Because Verus proves the proof model and not the kernel sources, the lane
SHALL keep the model pinned to the kernel: a Verus-built executable runs
the model's executable walk over the shared fixture corpus, and the lane
SHALL diff its emitted findings against the kernel's `check` output for
the same corpus. A divergence SHALL be treated under the counterexample
protocol, never silently reconciled by editing either side to match.

#### Scenario: Model drift is caught, not absorbed
- **WHEN** the kernel walk's semantics change and the proof model is not
  updated
- **THEN** the parity diff fails the lane, naming the fixture and the
  diverging finding

### Requirement: The guide tripwire corpus stays green
The lane SHALL pin the Verus guide example corpus as fixtures with their
expected per-file outcomes (verified, or intentionally-failing with the
documented error), so the lane detects a Verus release or toolchain bump
that changes guide-example behavior.

#### Scenario: A bump that breaks guide examples fails the lane
- **WHEN** the pinned guide corpus is re-verified under a new pinned
  release and a file's outcome differs from its recorded expectation
- **THEN** the lane reports the file as failed and exits nonzero; the pin
  bump proceeds only after the expectations are deliberately re-recorded

### Requirement: Every proof unit reports a three-valued outcome
Each proof unit SHALL report exactly one of: **verified**, **error**
(concrete counterexample or compile failure), or **resource-exhausted**
(solver budget). A run SHALL NOT drop, skip, or silently pass a proof
unit, and the lane's aggregate result SHALL name any unit that did not
verify.

#### Scenario: Resource exhaustion is loud, never green
- **WHEN** a proof unit cannot complete within its solver budget
- **THEN** the lane reports that unit as resource-exhausted with its
  budget, and `just verify-verus` does not present the family as proven

#### Scenario: Exhausted without a disposition fails the lane
- **WHEN** a proof unit ends resource-exhausted and no disposition (a
  raised budget or a split obligation, recorded in the lane README)
  exists for it
- **THEN** the recipe exits nonzero, naming the unit and its budget

### Requirement: Counterexamples are attributed, never silenced
A parity divergence or a failing proof obligation SHALL be resolved as
exactly one of: a **model defect** (the proof model misstates the
kernel's behavior or its own contract — fix the model or its proof) or a
**Rust kernel defect** (a real bug — fixed with a beads ticket and a
regression fixture through its own change, not by editing this lane's
scope). The attribution and the resolution ticket SHALL be recorded,
mirroring the Lean parity oracle's divergence protocol.

#### Scenario: A kernel defect found by the lane has a home
- **WHEN** a parity fixture shows the model faithfully describing behavior
  the kernel does not have, or a proof exposes a genuine contract
  violation in the modeled logic
- **THEN** the failure is filed as a kernel-defect beads ticket with the
  diverging fixture attached, and neither the model nor the obligation is
  weakened to make the lane green

### Requirement: The lane records a stop-gate decision in the ledger
After the walk-correctness and parity families land (end of phase 1), the
lane SHALL record one of **extend** / **hold** / **retire** in the ledger
as a terminal `decision` entry (the existing kind validated by
`catherine-ledger`), stating what the proofs bought (contracts
established, defects found, proof-engineering cost) versus
fixture-and-property assurance alone. The verdict gates the remaining
proof spend.

#### Scenario: The verdict gates the remaining proof spend
- **WHEN** the gate review runs after the first two families land
- **THEN** extend funds the phase-2 proof families and hold maintains at
  phase-1 scope — neither runs proof work the verdict did not fund

#### Scenario: Retire archives the lane with lessons kept
- **WHEN** the gate verdict is retire
- **THEN** the proof model, the guide corpus, the verify recipe, and the
  workflow job are archived, with the proof conventions and lessons kept
  in the lane README and the ledger entry

#### Scenario: The decision is sealed either way
- **WHEN** the stop-gate review runs
- **THEN** a `decision` ledger entry exists for the Verus lane and
  `ledger-check ledger/` accepts it — extend, hold, and retire are all
  valid outcomes; an unrecorded drift into unmaintained proofs is not
