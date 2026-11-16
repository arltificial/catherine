# Tasks: add-model-composition

One feature cycle (RED → GREEN) plus one refactor cycle. No format
change; single-document behavior is byte-identical to today.

## RED

- [ ] 1. Kernel gluing tests (failing): two documents sharing objects/
      arrows/rows glue by name; agreeing overlap is accepted; disagreeing
      arrow endpoints → `schema_conflict` at the offending node naming both
      documents; disagreeing row values → `row_conflict`; equations and
      keys from both sides are enforced on the merged instance
- [ ] 2. Cross-document equation tests (failing): an equation in doc A
      referencing doc B's names parses (syntax-only at document level) and
      resolves post-glue; a dangling cross-document reference → labeled
      `unknown_arrow` at glue time with document attribution; the
      single-document `unknown_arrow` envelope is byte-identical to
      today's parse-time error (one resolution path, spans preserved —
      design Decision 3)
- [ ] 3. CLI tests (failing): `check a.ct.json b.ct.json` runs the joint
      check; disjoint documents (zero shared names) → clean findings plus
      the `empty_composition_interface` warning naming both inputs; the billing/shipping fixture — each clean alone, joint run
      exits with the violation code and the finding names both documents;
      single-document invocations byte-identical to today; exit-code
      precedence unchanged; `--json` envelope carries `source` attribution
- [ ] 4. Underconstraint attribution tests (failing): a parallel-path
      question that exists only post-glue is reported with seam attribution
- [ ] 5. Contracts for the new scenarios (`ah scenario new`) and negative
      matrix rows for the conflict surfaces

## GREEN

- [ ] 6. Kernel: `glue(doc_a, doc_b)` — schema/instance union with the
      Decision-2 conflict checks; n-ary fold; post-glue name resolution
      for deferred equations
- [ ] 7. Findings: `source` attribution threaded through witness payloads
      (additive, canonical order stable)
- [ ] 8. CLI: variadic `check` path (>1 document → glue then check; 1
      document → today's path untouched)
- [ ] 9. Probe: commit `ledger/probes/composition/` catalog + fixtures
      (billing/shipping from the gap suite), floor control, and the
      executed hand-merged baseline arm; record the run

## REFACTOR

- [ ] 10. Gate caps: split any file/function the strict pretender gate
      rejects, without behavior change
- [ ] 11. Docs: CHANGELOG entry; README/USING note that `check` is
      variadic and the scope guardrail (findings only, no model algebra)

## Verification

- [ ] 12. `cargo test --workspace`; `just ci`; `openspec validate --strict
      --all`; `ah check --changes add-model-composition --run-tests`;
      gap-suite composition rows flipped to `closed` (requires
      `add-expressibility-gap-suite` archived first — see Impact)

## Boundary

- [ ] 13. Close note: no document emission (Decision 4), conflicts are
      never silent (Decision 2), probe verdict recorded whatever it is
      (Decision 5 — a `killed` probe is a deliverable)
