# ro5u review — add-schema-diff (catherine-dvx, P10 — final ticket)

Lead-side review. Diff: scaffolded `openspec/changes/add-schema-diff/`
(proposal + tasks with the pre-registered validation outcomes +
specs/analysis deltas — three ADDED requirements; strict-valid),
`crates/catherine-analysis/src/schema_diff.rs` (234 LOC: multiset diff
of ENFORCED equations — accepted `(lhs, rhs)`; rationale/order/proposed
are presentation — + classification Cosmetic | Accreted | Weakened |
StrengthenedInScope | EquivalentInScope | Unknown | Incomparable, with
one existing-taxonomy finding for weakening: `violation`/`executed`
carrying the v1 equation index and composed values), the shared bounded
engine extraction in `search.rs` (`enumerate` + `is_clean_typed`, so
candidate order/budget semantics cannot drift between search and diff),
the `catherine diff` CLI (exit 3 weakened via the existing class table,
0 cosmetic/accreted/equivalent/strengthened/unknown, 1 labeled
`schema_incomparable`), three fixture pairs, tests (analysis 3 + cli 3
+ negative-matrix rows; negative_matrix.rs now 396 LOC — under cap),
docs (analysis README limits/layout, DESIGN §4.2/§4.3 deterministic-op
list, USING Recipe C, root README row, CHANGELOG).

Orchestrator verification (live): the three pairs verbatim — weakened
exit 3 with the witness (`rows Node [0,1]`, `next [1,0]`) re-checking
valid in v2 and rejected in v1; refactor → `equivalent-in-scope` with
FULL exhaustion (scope 2×3×3, 18 candidates) and zero findings; cosmetic
→ `cosmetic`, silent, exit 0; `--max-candidates 1` → `unknown` with the
exact scope; zero-size bound → empty-scope unknown. `just ci` EXIT=0;
pretender green; all 11 changes strict-valid.

Five passes:

1. Correctness: classification is multiset-based over ENFORCED equations
   only (presentation edits are cosmetic by construction — the
   negative-control silence case); weakening demands a witnessed
   instance re-checked against BOTH versions; equivalence demands
   exhaustion within bounds (no silent equivalence claims).
2. Contract drift (class B): no kernel or format change (LOC delta: 234
   analysis + 45 shared, kernel 0 — the ≤400 ceiling is a falsifiable
   claim, honored); no new finding kinds; the deterministic-op list and
   docs updated together.
3. Tests: three fixture pairs + bounded-honesty cases + the
   strengthened-not-rounded case; negative-matrix rows for the new
   surface.
4. Negative matrix: rows added; 348 discipline held.
5. Regression risk: the search-engine extraction is behavior-preserving
   by construction (search runs on the same engine); existing suites
   re-run green.

Findings + dispositions:

- [NOTE] `schema_incomparable` is a labeled ERROR (exit 1), not a
  finding — incomparable schemas are a tool limitation surfaced loudly,
  not a taxonomy claim. Consistent with the closed-error discipline.
- [NOTE] the ticket's deterministic-op list entry landed in DESIGN
  §4.2/§4.3 (docs-as-contracts).

max_critical = 0. Verdict: PASS.