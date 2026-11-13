# Tasks: add-schema-diff

One red→green→refactor cycle (beads `catherine-dvx`, P10). Builds on
P4 (`add-bounded-analysis`) and P7 (`add-delta-migration`); no new
finding kinds.

## RED

- [x] 1. Fixture pair: silent weakening — v1 enforces an accepted
      equation v2 drops (`fixtures/schema_diff/weakened/`); the edit
      must be caught with a witness, never silently accepted
- [x] 2. Fixture pair: refactor — an orientation-normalized equation
      (`f∘g = h` written `h = f∘g`), same instance space
      (`fixtures/schema_diff/refactor/`); equivalence must be proven by
      bounded distinguish, not asserted
- [x] 3. Fixture pair: cosmetic negative control — rationale/order-only
      edit (`fixtures/schema_diff/cosmetic/`); must be silent

## GREEN

- [x] 4. `schema_diff::diff`: enforced-equation multiset diff (accepted
      `(lhs, rhs)` pairs; rationale/order/proposed equations are
      presentation) and the classification outcome enum
- [x] 5. Weakening witness: bounded enumeration of fully resolved
      instances; `v2`-clean and `v1`-violating yields the existing
      `violation`/`executed` finding with the v1 equation index and
      composed values; `cosmetic`/`accreted`/`equivalent-in-scope` emit
      zero findings
- [x] 6. Shared `search::enumerate` primitive (found / exhausted-with-
      scope / unknown-with-scope); `search` re-expressed on it, so the
      two consumers share ordering and budget semantics
- [x] 7. CLI `catherine diff <v1> <v2>` (runner + renderer + dispatch):
      weakened exits 3 through the existing class table; cosmetic /
      accreted / equivalent / strengthened / unknown exit 0 with the
      outcome in the payload; incomparable is the labeled
      `schema_incomparable` error (exit 1)

## REFACTOR

- [x] 8. Deterministic-op list + docs: `docs/DESIGN.md` §4.2/§4.3,
      `crates/catherine-analysis/README.md` limits + layout,
      `docs/USING.md` Recipe C, CHANGELOG; the analysis module stays
      inside the ≤400-LOC kernel-delta ceiling with zero kernel change

## Validation (pre-registered)

The three katas below are fixed before implementation; the recorded
outcomes are the acceptance evidence, reported honestly.

- [x] 9. Silent-weakening kata (`fixtures/schema_diff/weakened/`,
      bound `Node=2`, 64 candidates): MUST classify `weakened` with
      exactly one `violation`/`executed` finding (`equation 0`, row 0,
      `lhs 0 ≠ rhs 1`), and the witness instance MUST re-check clean in
      v2 and rejected in v1. **Recorded: weakened; 1 finding; witness
      `next = [1, 0]` on `Node#0/#1`; v2 check empty, v1 check exactly
      the finding.**
- [x] 10. Refactor kata (`fixtures/schema_diff/refactor/`, bounds
      `a=1,b=1,c=2`, 128 candidates): MUST classify
      `equivalent-in-scope` after full exhaustion (18 candidates), with
      zero findings. **Recorded: equivalent-in-scope; scope rows
      `{a:1,b:1,c:2}`, 18 candidates; 0 findings.**
- [x] 11. Cosmetic negative control
      (`fixtures/schema_diff/cosmetic/`): MUST classify `cosmetic` with
      zero findings. **Recorded: cosmetic; 0 findings.**
- [x] 12. Bounded honesty: a 1-candidate budget on the weakening pair
      MUST classify `unknown` with scope `{Node:2}`, 1 candidate — never
      equivalence; a zero-size bound MUST be `unknown` with the empty
      scope. **Recorded: unknown @ 1 candidate; empty scope at zero-size
      bound.**
- [x] 13. Orchestrator-owned gates: `cargo fmt`, workspace clippy, full
      test suite, `openspec validate add-schema-diff --strict`, then
      `bd close catherine-dvx` + fresh export (this worker runs only
      scoped checks). **bd close pending orchestrator.**