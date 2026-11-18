# Rule of 5 Review — add-kani-verification-lane (2026-11-18)

Universal Rule-of-5 review of the change proposal at filing time
(proposal.md, specs/kani-verification-lane/spec.md, design.md, tasks.md).
Advisory; findings below were fixed in the follow-up revision commit.
Verification: TypeSafe Stage B over CRITICAL/HIGH — CORR-001 `verified`
@ 0.87, EDGE-001 `verified` @ 0.93 (gate 0.8); measured FP rate 0/2 = 0%.

## Findings and disposition

| ID | Sev | Location | Summary | Fix applied |
|----|-----|----------|---------|-------------|
| CORR-001 | HIGH | proposal.md What Changes vs tasks.md phases | Stop-gate timing contradiction: proposal/spec gate "after the first two harness families" but tasks ran phase 2 before the gate | Gate moved to end of phase 1; phase 2 tasks (8–9) now gated on the extend verdict (tasks.md, design Decision 6, spec stop-gate requirement) |
| EDGE-001 | HIGH | spec.md lane-separation THEN clause | `just verify` exit contract covered only counterexample/harness error — lane could exit green with an UNDETERMINED harness | Undetermined-without-recorded-disposition now fails the lane (spec new scenario + THEN clause, design Decision 5, tasks task 1) |
| CORR-002 | MEDIUM | design Decision 2, tasks task 2 | `kani::any()` cannot generate `BTreeMap`-bearing domain types (`instance.rs` rows/tables); wording implied direct generation | Reworded: bounded scalars → deterministic insertion loops; `UNVERIFIED` (external-tool claim, below verify severities) |
| CLAR-001 | MEDIUM | spec.md panic-freedom requirement | "parse-product execution path" meaningless; pointed opposite from Decision 2's parse exclusion | Reworded to "post-parse execution path" |
| EDGE-002 | MEDIUM | spec stop-gate requirement, design Decision 6 | "retire" verdict had no disposition semantics (Lean change spelled out "tree archived, lessons kept") | Retire now archives harness modules + verify recipe, lessons kept in README + ledger |
| EDGE-003 | MEDIUM | spec pinned-toolchain requirement | No proof-of-record: nothing pinned which Kani version produced recorded proofs | Lane README records evidence provenance (Kani release per recorded proof result) |
| CLAR-002 | LOW | proposal.md What Changes | "a `kani` feature + `#[cfg(kani)]`" read as two mechanisms | Single mechanism: `#[cfg(kani)]` (checker sets the cfg; no feature knob) |
| CLAR-003 | LOW | tasks task 9 (old numbering) | "elan-style provisioning" imported Lean's toolchain-manager name as analogy | Reworded to "pinned Kani release install, provisioning check in the recipe" |
| EDGE-004 | LOW | spec panic-freedom requirement, tasks task 3 | Vacuous-proof risk: bounds could be too tight to exercise checks | Non-vacuous bounds clause: bounded space admits a document exercising each check family |
| EXCL-001 | LOW | proposal.md Why | "190 docs" presented as current; point-in-time number | Labeled "at the Lean change's parity run" |

## Stage quality at review time

Draft GOOD · Correctness GOOD · Clarity GOOD · Edge Cases FAIR (closed by
EDGE-001/002/003 fixes) · Overall: NEEDS_REVISION → revised.

## Convergence

0 CRITICAL throughout; new-issue rate stayed >10% per stage (no early
stop); escalation not triggered (measured FP 0%, no uncertain verdicts).
