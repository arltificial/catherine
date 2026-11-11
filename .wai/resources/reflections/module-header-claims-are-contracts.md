# Practice: module-header absolute claims are contracts

**Rule:** any module-header sentence of the form "X never/only/always Y"
gets a declared scenario (spec `#### Scenario:` + contract `.toml` bound to
a real test) or is downgraded to a documented-limit note — docstrings are
contracts.

## Why

Module headers state behavior in epistemic vocabulary (`never`, `only`,
`always`). Nothing forced that vocabulary to stay true: a refactor can
quietly break the promise while the prose keeps asserting it, and reviewers
trust the header because it *sounds* like a spec.

## Evidence

- `catherine-u3k` (fixed 18c37a4): `crates/catherine-analysis/src/search.rs`
  header promised "only accepted equations constrain the search", but
  `is_clean` counted proposed-equation violations as dirty — the prose was
  the spec, the code disagreed, and no scenario noticed.
- `catherine-mrq` (P2, open): `crates/catherine-analysis/src/rewrite.rs`
  header claims leftmost-innermost reduction; the implementation is
  leftmost-outermost — same drift class, no scenario noticing.

## Gate

`catherine-za6` adds the recurrence gate: the search accepted/proposed
split is now a declared scenario in
`openspec/changes/add-bounded-analysis/specs/analysis/spec.md` with a
contract at
`.espectacular/changes/add-bounded-analysis/analysis/accepted-equations-constrain-proposed-equations-question.toml`
bound to the regression tests in `search.rs`. The pre-push
`ah check --run-tests` hook keeps declared scenarios executable.

## Decision procedure

1. Grep the module header (`//!` block) for `never|only|always`.
2. Is the sentence a behavioral claim (the code could disagree)?
   - Process/history notes ("extending the enum is a follow-up ticket",
     "split out verbatim, no behavior change") are not behavioral — leave.
   - Type-enforced facts ("a finding without a witness is
     unrepresentable") cannot drift — leave.
3. Backed by a declared scenario or an existing test? → keep the wording,
   name the test/scenario in the sentence where practical (see
   `query.rs`, which cites its parity tests inline).
4. Cheap and clearly testable, and the claim is load-bearing? → declare a
   scenario (spec heading + `ah scenario new` + `[tests]` binding).
5. Otherwise → downgrade: rewrite as a documented-limit note without the
   absolute ("reads as `None`, not a separate report", "has no producer
   here"), so the doc records the limit instead of promising it.
