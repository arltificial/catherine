# Brief — catherine-mx1.4: Null Object pattern lens

Beads ticket: `catherine-mx1.4` (P2, lane:afk). Parent epic: `catherine-mx1`. Blocked by `catherine-mx1.2`.

## Why

Delegation rationale: one lens is a bounded module + test slice with a fixed seam contract,
dispatched in parallel with the other eleven lenses; the lead owns registry wiring and replay
verification. Independent files per ticket make the parallel batch conflict-free.

## Desired outcome

Implement the `Null Object` lens in `crates/catherine-analysis/src/pattern/null_object.rs`:

- `pub struct Lens;` implementing `PatternLens` with metadata:
  id `null_object`, name `Null Object`, abstraction `monoid identity element`, law per the seam enum,
  scope `identity/totality for the nil element`.
- Law: nil-identity equations.
- Bind the declared domain from `LensInput::values` via
  `catherine_core::rule_lens::RuleDomain::from_presentation` and reuse
  `catherine_analysis::rule_lens::rule_lens`; return its findings, or
  `LensOutcome::Unknown { scope }` on budget exhaustion. Findings use the existing
  P2 taxonomy kinds and witnesses only.
- Parity rows stay silent: the killed/parity cases recorded in
  `ledger/katas/composite-null-object/` must produce no finding.
- Evidence to read first: `ledger/katas/composite-null-object/` (every entry's `kata.series_verdict`,
  `run.commands`, `run.findings`) and `fixtures/katas/composite-null-object/`.
- File header: `//!` Purpose / Responsibilities / Rationale, per the crate convention.

## Out of scope

- Do not edit `crates/catherine-analysis/src/pattern/mod.rs` (the orchestrator wires the
  registry), `ledger/katas/`, `docs/ref/pattern-kata-dispositions.md`, any other lens module,
  or the replay harness.
- No new finding kinds, no new heuristics beyond the recorded law, no detection-rate claim.

## Exact files

- `crates/catherine-analysis/src/pattern/null_object.rs`
- `crates/catherine-analysis/tests/pattern_null_object.rs`

## Red → green → refactor

1. RED: `crates/catherine-analysis/tests/pattern_null_object.rs` calls `Lens.run(...)` directly
   (not the registry) and asserts the recorded class per fault against the fixture; write the
   failing assertions first.
2. GREEN: implement the module until the test passes.
3. REFACTOR: within the module only.

## Completion criteria (runnable; exit 0 = done)

```bash
cargo test -p catherine-analysis --test pattern_null_object
cargo test -p catherine-analysis --lib
```

Do not run repo-wide gates or formatters; the orchestrator runs them once after the batch.
The registry/replay Meter is executed by the orchestrator after wiring:
`cargo test -p catherine-analysis --test pattern_replay null_object`.

## Spawn model

Agent: `task` (harness default model). Context ceiling: ≤15% of the model window.

## Follow-up threshold

If the recorded law cannot be expressed with the seam signature without changing `rule_lens`
semantics, stop and report the exact conflict instead of widening the seam.
