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
- [x] 7. Equivalence checking + discovery support (examples in, gap
      findings out). Evidence: `core::rule_lens::equivalence` proves
      equivalence by witness; `analysis::rule_lens::discover` checks a
      candidate table against declared input→output examples, mapping
      uncovered examples to `underconstraint`/`UncoveredInputs` and
      differing actions to `violation`/`EquationWitness`, scoped over the
      sample (`tests/rule_lens_discovery.rs`: covered-clean, uncovered-gap,
      differing-violation).
- [x] 8. CLI surface for rule-lens analysis. Evidence: `catherine rules
      <doc> --values column=v1,v2,... [--max-candidates N]` emits the
      findings envelope (`data.findings` verbatim; budget expiry is
      `outcome: "unknown"` + the exact scope) and one human line per
      finding. `tests/cli_rules.rs` pins clean (0), gap (6), shadow (7),
      short-budget UNKNOWN (0), and `no_rule_presentation` (1); the
      negative matrix carries the new rules rows.

## VALIDATION (pre-registered)

- [x] 9. Above-floor kata: 10–20 compound rules + priority policy;
      unique-detection audit vs conventional suite. Evidence:
      `fixtures/rule_lens/kata/pricing-gold.ct.json` (16 compound rules,
      PriorityOrder, 240-input domain, clean: 0 findings) and the frozen
      conventional baseline `prototype/rule_lens_baseline.py` (45
      behavior-only assertions). Twelve injected faults under
      `fixtures/rule_lens/kata/faults/` (3 gap, 3 shadow, 4 derivable,
      2 overlap-under-DeclaredDisjoint): conventional catches 7/12,
      `catherine rules` catches 12/12, unique detections 5/12
      (04, 05, 07, 09, 10 — all behavior-preserving structural
      redundancy, witnessed). Ledger entry
      `ledger/rule-lens/0002-4ef797a1.json`.
- [x] 10. Near-floor negative control; ledger entries per protocol.
      Evidence: `fixtures/rule_lens/kata/negative-control/near-floor.ct.json`
      (3 rules, 4 inputs) plus 3 behavior-changing faults; the 4-row
      conventional check catches 3/3 and the lens catches 3/3 — zero
      unique detections. Control entry
      `ledger/rule-lens/0001-65162a79.json` (verdict `killed`, empty
      `unique_detections`). Entries live in `ledger/rule-lens/` because
      the P6 ledger is sealed by its seq-5 rollup (the validator requires
      a rollup to be last; appending #6+ after it would be refused).
- [x] 11. Killed ⇒ document negative result, stop rule-lens scope;
      survived ⇒ record evidence. The committed procedure derives
      `survived` (scoped): five unique detections, all structural
      redundancy; every behavior-changing injected fault was caught by
      the conventional suite. Rollup `ledger/rule-lens/0003-0eee8ef3.json`;
      `ledger-check ledger/rule-lens` → `ok: 3 entries (2 kata),
      rollup verdict: survived`. `crates/catherine-ledger/tests/rule_lens_ledger.rs`
      recomputes the fixture hashes and the derived rollup. The bead flip
      is orchestrator-reserved.

## REFACTOR + close

- [x] 12. Dedupe predicate evaluation; `openspec validate add-rule-lens
      --strict`; `bd close catherine-nq5` + fresh export
