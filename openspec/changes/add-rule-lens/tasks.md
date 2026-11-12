# Tasks: add-rule-lens

One red→green→refactor cycle (beads `catherine-nq5`). HYPOTHESIZED —
killable per design.md Decision 4.

## RED (sim-first per Decision 5)

- [x] 1. Python sim: effective regions + equivalence laws vs
      hand-computed tables (the oracle); commit as prototype
- [x] 2. Golden fixtures: gap, overlap-under-DeclaredDisjoint, shadow,
      derivable — each asserting the mapped taxonomy kind from
      design.md Decision 1

## GREEN

- [x] 3. Attribute columns + predicate expression language (additive
      format extension + JSON Schema update)
- [x] 4. `RulePresentation` types + policy semantics (total, no implicit
      precedence). Evidence: `core::rule_lens::analysis` computes
      effective regions per policy; `rule_lens_parity` pins priority,
      declared-disjoint, and no-policy behavior.
- [x] 5. Effective-region computation in Rust; sim/kernel parity on all
      sim tables. Evidence: `tests/rule_lens_parity.rs` (7 analysis
      cases) + `tests/rule_lens_equivalence.rs` (4 equivalence cases)
      reproduce `prototype/rule_lens_sim.py`; `tests/rule_lens_fixtures.rs`
      analyses all four golden fixtures.
- [x] 6. Findings as taxonomy producers (no new variants); enumeration
      via P4 bounded search. Evidence: `analysis::rule_lens` maps the
      four classes (gap → underconstraint/hypothesized with
      `UncoveredInputs`; overlap under DeclaredDisjoint → violation;
      shadow/derivable → redundant), `tests/rule_lens_findings.rs`
      pins all four plus `UNKNOWN { scope }` on budget expiry.
- [ ] 7. Equivalence checking + discovery support (examples in, gap
      findings out)
- [ ] 8. CLI surface for rule-lens analysis

## VALIDATION (pre-registered)

- [ ] 9. Above-floor kata: 10–20 compound rules + priority policy;
      unique-detection audit vs conventional suite
- [ ] 10. Near-floor negative control; ledger entries per protocol
- [ ] 11. Killed ⇒ document negative result, stop rule-lens scope;
      survived ⇒ record evidence

## REFACTOR + close

- [ ] 12. Dedupe predicate evaluation; `openspec validate add-rule-lens
      --strict`; `bd close catherine-nq5` + fresh export
