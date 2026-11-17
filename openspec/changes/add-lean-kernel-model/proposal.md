# Change: Add the Lean 4 kernel model — external assurance twin of the semantic kernel
> IDs: E1–E10 = docs/DESIGN.md §0 evidence base · kimi `1a1e453e` = cotorreo `raw/_conversations/kimi/1a1e453e-ddf2-8ddb-8000-099928ab74f3/` (2026-11-16 "Decision on Lean Modeling")

## Why

The kernel's contracts (witness-backed findings, dangling discipline, honest
UNKNOWN) are currently pinned operationally by fixtures and property tests —
there is no artifact that *states* them. A prior recommendation (kimi
`1a1e453e`, grounded in this repo's tree) concluded: yes to a **thin Lean 4
twin of the semantic kernel**, no to modeling CLI/WASM/UI, and no to a runtime
dependency — DESIGN §2 keeps "general theorem prover" a non-goal, so Lean is
an *external assurance artifact*, not a new engine. This change lands the
phase the recommendation scoped as weeks 1–2: formalize the kernel semantics
and prove the two headline properties (checker soundness, equation +
dangling completeness), gated by a parity oracle over the existing fixture
corpus. Weeks 3 (bounded search, migration) are an explicit follow-up
decision, not hidden scope. Beads ticket: `catherine-xvh`.

## What Changes

- New `lean/` tree (Lake project, Lean 4 **std only — no Mathlib**): a
  reference executable semantics mirroring the Rust kernel's data shapes —
  `Schema.lean`, `Instance.lean` (dense row spaces, partial `Vec<Option>`
  morphism tables, typing-violation/dangling build reports), `EvalPath.lean`
  (`compose` with `None` short-circuit), `Findings.lean` (the taxonomy's
  kinds, witnesses, and the three constructible labels), `Check.lean` (the
  `check` walk: dangling, equation, key, coproduct findings, sorted).
- Theorems proved in Lean, stated against the *semantic* notions, not the
  walk:
  1. **Soundness** — every finding `check` emits carries a semantic witness
     (a real violated equation on an applicable row, a real unresolved cell
     accounting, a real duplicate key tuple, a real coproduct failure).
  2. **Equation completeness** — every semantic equation violation on an
     applicable row is reported, labeled `executed` for accepted and
     `hypothesized` for proposed constraints, never merged.
  3. **Dangling discipline** — dangling cells are representable values:
     never repaired, never reported as violations, never grouped into key
     collisions or coverage decisions; a typing-violated cell is accounted
     exactly once (violation report, not dangling).
- **Parity oracle**: a harness runs Lean `check` and Rust `check` over the
  fixture corpus (`fixtures/`) and compares normalized findings — divergence
  is always resolved as one of two named beads (Lean-model defect or Rust
  kernel defect), never silenced.
- **Stop gate**: after the phase-2 proofs, continue only if the proof burden
  is buying confidence beyond what the fixture corpus already pins; the
  ledger records the decision either way.
- Openspec capability `lean-kernel-model` (new; consumes the contracts of
  `kernel-checks`, `findings`, `analysis`, `migration` as its theorem
  statements).

## Impact

- Affected specs: `lean-kernel-model` (new capability). No existing
  requirement is modified — if parity surfaces a Rust defect, that is a
  defect fix through its own beads ticket, not a spec edit.
- Affected code: new `lean/` tree, plus a `catherine-ledger` schema
  extension forced by the stop-gate requirement: a new terminal `decision`
  entry kind, its validator rules, and the `ledger-record` sealing path
  (the existing schema had no decision kind). The Rust kernel is untouched;
  the Cargo workspace gains no dependency on Lean and vice versa. Kernel
  changes happen only if parity finds kernel defects (filed separately, out
  of this change's scope).
- Beads: implements `catherine-xvh`.
