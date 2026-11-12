# ro5u review — add-rule-lens (catherine-nq5, HYPOTHESIZED → survived-scoped)

Full-diff lead-side review across four phases. Diff: format 1.2 rule
presentation (`model.rs` types; `parse/rule_presentation.rs` +
`parse/predicate.rs`; KNOWN_MINOR 1→2; hash-neutral with the pinned
guard; ct-schema + regenerated TS), the effective-region engine
(`rule_lens/{domain,analysis,equivalence}.rs`) with sim parity, the
Decision-1 taxonomy producers (`catherine-analysis/src/rule_lens.rs`),
example-scoped discovery, the `catherine rules` CLI surface (envelope,
class exits, 9 negative-matrix rows), `prototype/rule_lens_sim.py`
(self-checking oracle) + `prototype/rule_lens_baseline.py` (independent
conventional baseline), the phase-D validation artifacts
(`fixtures/rule_lens/kata/**`, 12 fault fixtures), and the rule-lens
ledger (`ledger/rule-lens/` — control first, kata, derived rollup; a
separate ledger because the sealed P6 ledger's rollup must be last,
which is the validator's own rule working as designed) plus a durable
ledger test.

Orchestrator verification (live): `ledger-check ledger/rule-lens` →
"3 entries (2 kata), rollup verdict: survived"; P6 ledger unchanged
(survived); sim PASS (7 analysis + 4 equivalence oracle cases); `just
ci` EXIT=0; pretender green; openspec validate strict valid; CLI exits
verified across classes (clean 0 / gap 6 / shadow 7 / no-section 1 /
UNKNOWN 0).

The validation, honestly: 12 injected faults on a 240-input above-floor
kata — conventional 45-assertion baseline 7/12, lens 12/12, **unique
5/12 — all behavior-preserving structural redundancy** (shadowed rules,
derivable rules). Every behavior-changing fault was caught by BOTH; the
near-floor control self-negated (0/3 unique). Verdict per Decision 4:
survived-scoped, structural axis only. The report states this caveat in
the entry's discrepancies rather than rounding it into a behavioral win.

Five passes:

1. Correctness: semantics staged sim-first (the oracle pins the laws;
   the kernel ports them); per-DomainQuestion and kind mappings are
   Decision-1 exact; no-policy overlap stays diagnostic (no invented
   kind); bounded enumeration expires to UNKNOWN{scope}.
2. Contract drift (class B): the additive 1.2 section keeps every
   pre-rule document's model_hash pinned; KNOWN_MINOR raised with the
   remediation hint updated; the version-policy tests follow.
3. Tests: sim parity (10 cases), finding-class fixtures (4), analysis
   suites (9), discovery (3), CLI (7) + negative matrix (9 rows),
   ledger derivation recomputed durably.
4. Negative matrix: 9 rows for the new CLI surface; the 348 discipline
   held.
5. Regression risk: additive format (hash-neutral), new subcommand with
   an extended exit class (6 already existed; no new numerals), kernel
   genesis-free kept.

Findings + dispositions:

- [RECORDED, honest scope] the 5 unique detections are structural
  redundancies only — a competent conventional suite catches every
  behavior-changing fault; the ledger discrepancy field carries this.
  No claim of a behavioral detection-rate win anywhere.
- [NOTE] `ledger/rule-lens/` as a sibling ledger: mechanically correct
  (P6's sealed ledger forbids appends after its rollup) and per
  Decision 4's own requirement for a fresh control.
- [NOTE] my orchestrator probe with a restricted `--values` sample
  exits 7 (in-sample redundancy) — correct behavior; the full-domain
  run is the documented clean case.

max_critical = 0. Verdict: PASS.