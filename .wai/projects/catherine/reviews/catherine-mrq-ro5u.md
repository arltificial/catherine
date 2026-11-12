# ro5u review — catherine-mrq (docs-only wording fix)

Scope of diff: `crates/catherine-analysis/src/rewrite.rs` — three docstring
spots corrected: "leftmost-innermost" → "leftmost-outermost" (lines 7, 19, 68).
No code change. Adjudication per ticket: the scan is positions-then-rules
(first position wins, canonical rule order) = leftmost-outermost; deterministic
either way; docstring is the contract in this codebase (za6 discipline).
Recorded here because the ticket has no openspec change dir (deviation from
the per-change artifact convention, noted in the orchestrator report).

Five passes over the full diff:

1. Correctness: wording now matches `rewrite_step` (outer `position` loop,
   inner canonical `rule` loop). MATCH.
2. Contract drift (class B): claim and implementation now agree; no scenario
   pins scan order — deferral reason: scan order is unit-testable but the
   deterministic-order guarantee is already pinned by determinism tests on
   normal forms; a dedicated order-pinning scenario is LOW, deferred.
3. Tests: no behavior change → no test required (ro5u checklist condition
   "behavior-impacting fix" does not trigger).
4. Negative matrix (class A/348): no flag/surface touched. N/A.
5. Regression/regression-risk: none — comments only.

Findings: none ≥ MEDIUM. max_critical = 0. LOW deferred: scenario pinning
rewrite scan order (written reason above).

Verdict: PASS.
