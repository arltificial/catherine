# Tasks: add-expressibility-gap-suite

One feature cycle (RED → GREEN) plus one refactor cycle. Test and docs
infrastructure only; no product behavior change, so no sealed artifact
moves.

## RED

- [ ] 1. Verify-at-HEAD pass: run every candidate limitation claim from
      the chat investigations against the current binary and record the
      observed behavior (identity/empty-path equations, optional
      references, unknown top-level fields, cones, non-functorial
      `schema_map`, dropped equations on migrate, vacuous clean,
      `distinguish` observation scope, associativity scaffolding,
      composition absence, naturality absence, coproduct absence)
- [ ] 2. Disposition table (failing docs gate): draft
      `docs/ref/expressibility-dispositions.md` with one row per case —
      id, claim, source conversation (provider + conv id), observed HEAD
      behavior, disposition `gap`/`closed`, sealing evidence
- [ ] 3. Harness tests (failing): a `gap` row whose case contradicts the
      claim fails; a `closed` row whose case regresses to the limitation
      fails; a case with no table row fails; the suite passes when table
      and observations agree
- [ ] 4. Fixture documents for every registered case, each asserting the
      *claimed* behavior structurally (exit code, finding kinds, parse
      outcome — never byte-compare)

## GREEN

- [ ] 5. Harness: the disposition runner (table → cases → verdicts) under
      `crates/catherine-cli/tests/expressibility_gaps/`; `just ci` wiring
- [ ] 6. Docs gate: the grep gate over limitation claims ↔ row ids (new
      work — no existing docs gate to mirror; design Decision 4)
- [ ] 7. Register the initial rows with their verified dispositions;
      `closed` rows cite fixing commits (pattern lens, additive-section
      rework, whichever the verification pass confirms)

## REFACTOR

- [ ] 8. Gate caps: split any file/function the strict pretender gate
      rejects, without behavior change
- [ ] 9. Docs: CHANGELOG entry; README note pointing at the table as the
      registry of known expressibility limitations

## Verification

- [ ] 10. `cargo test --workspace`; `just ci`; `openspec validate --strict
      --all`; `ah check --changes add-expressibility-gap-suite
      --run-tests`; deliberately flipping one row's disposition fails the
      suite (negative check), restoring it passes

## Boundary

- [ ] 11. Close note: expressibility claims only (Decision 5); rows cite
      chat provenance but assert HEAD truth (Decision 2); flips go
      through the table in the same PR as the behavior change
      (Decision 1)
