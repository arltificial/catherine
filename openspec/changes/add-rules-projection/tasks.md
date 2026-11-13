# Tasks: add-rules-projection

One red→green→refactor cycle (beads `catherine-x94`). No format change:
`model-document`, the hash, and every sealed ledger entry are untouched.

## RED

- [ ] 1. Hermetic fixture + failing CLI tests
      (`crates/catherine-cli/tests/cli_rules_projection.rs`):
      collision → exactly one `violation` with both row names in the witness;
      clean projection → exit 0, zero findings; declared-but-uncovered
      `--values` input → `gap` (`underconstraint`); `--action` projection;
      each refusal (Decision 5) with its owned code
- [ ] 2. Contract stubs for the new cli scenarios (`ah scenario new`), each
      pointing at its RED test

## GREEN

- [ ] 3. Projection module: `Model` instance rows → `RulePresentation`
      (row-order rules, conjunctive equalities, identity/`--action` actions,
      fixed `DeclaredDisjoint`) + observed-tuple domain unioned with
      `--values`
- [ ] 4. CLI wiring: `--project`, `--columns`, `--action` on `rules`;
      `data.projection` payload; help text stating the unsealed-claim boundary
      and the product-declaration caveat
- [ ] 5. Acceptance run on `fixtures/probes/capture-selector/` per its
      README: exactly 3 violations (three ambiguous selectors), no gap on the
      30 undeclared zero-match selectors, exit 3; declared
      `source=nacional/primera edition_date=2026-11-07` adds the gap

## REFACTOR

- [ ] 6. Negative matrix rows for the new surface (the repo rule: a new
      flag/surface without a matrix row is incomplete); help/README consistency
      pass; CHANGELOG entry

## Boundary

- [ ] 7. Record in the change's close note that the projection claim is not
      sealed evidence (no hash, no ledger), and that this change neither
      validates nor invalidates the granularity rule