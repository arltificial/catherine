# Tasks: add-rules-projection

One red→green→refactor cycle (beads `catherine-x94`). No format change:
`model-document`, the hash, and every sealed ledger entry are untouched.

## RED

- [x] 1. Hermetic fixture + failing CLI tests
      (`crates/catherine-cli/tests/cli_rules_projection.rs`):
      collision → exactly one `violation` with both row names in the witness;
      clean projection → exit 0, zero findings; declared-but-uncovered
      `--values` input → `gap` (`underconstraint`); `--action` projection;
      each refusal (Decision 5) with its owned code
- [x] 2. Contract stubs for the new cli scenarios (`ah scenario new`), each
      pointing at its RED test

## GREEN

- [x] 3. Kernel seams (hash-neutral, additive): `parse::parse_with_rows` +
      `RowNames` (names outside `Model`, so equality/canonical bytes/hash are
      untouched); `RuleDomain::from_tuples` exact-input construction
- [x] 4. Projection module: `Model` rows + `RowNames` → `RulePresentation`
      (row-order rules, conjunctive equalities, identity/`--action` actions,
      fixed `DeclaredDisjoint`) + observed-tuple domain unioned with
      `--values`; CLI wiring of `--project`, `--columns`, `--action`; help
      text stating the unsealed-claim boundary and the product-declaration
      caveat (envelope shape unchanged)
- [x] 5. Acceptance run on `fixtures/probes/capture-selector/` per its
      README: exactly 3 violations (three ambiguous selectors), no gap on the
      30 undeclared zero-match selectors, exit 3; declared
      `source=nacional/primera edition_date=2026-11-07` adds the gap

## REFACTOR

- [x] 6. Negative matrix rows for the new surface (the repo rule: a new
      flag/surface without a matrix row is incomplete); help/README consistency
      pass; CHANGELOG entry

## Boundary

- [x] 7. Record in the change's close note that the projection claim is not
      sealed evidence (no hash, no ledger), and that this change neither
      validates nor invalidates the granularity rule
## Close note

Landed with `catherine-x94`. Evidence: `cargo test --workspace` green (incl. 10
projection tests and the seven new negative-matrix rows); acceptance A1–A4 on
`fixtures/probes/capture-selector/` reproduced from the committed slice — three
violations whose witness pairs are exactly the two captures behind each
ambiguous selector, no gap on the 30 undeclared zero-match selectors, exit 3,
and the declared `ed_2026_11_07` input adds the expected gap.

Boundary: the projection claim is outside the document hash and the ledger —
evidence about a projection, never sealed model evidence. This change neither
validates nor invalidates the granularity rule (`catherine-8v9` stays closed and
the rule stays PROPOSED for structural claims only).
