# Arms — rule-lens selector-collision probe (catherine-8qm)

Pre-registered with the README (RED, before any run). The README owns the
domain decision, the rule, custody, expectations, decision rule and review;
this file owns the exact inputs, commands, and the ledger entry template.

## Arm A — conventional suite (the executed control)

Script: `prototype/bogart_selector_bench.py` (sha256
`df8106581e2ff3eec83861372f0862decebca8b3e9874e951161f50d7553c762`,
committed with kw9.11). Census match semantics verbatim:
`src==s and (ed==e or e in metadata 'edition-dates')`; detections = selectors
matched by ≥2 stored capture rows.

Commands (all outputs and exit codes recorded verbatim in the ledger):

```
python3 prototype/bogart_selector_bench.py --instance fixtures/probes/selector-state/slice.json
python3 prototype/bogart_selector_bench.py --floor --instance fixtures/probes/selector-state/floor-selectors.json
python3 prototype/bogart_selector_bench.py --seed s1 --instance fixtures/probes/selector-state/slice.json
python3 prototype/bogart_selector_bench.py --seed s2 --instance fixtures/probes/selector-state/slice.json
```

Registered outcomes: probe run 3/3 detections (E1 `nacional/primera
2026-11-03`, E2 `pba/JUDICIAL 2026-11-03`, E3 `nacional/segunda
2026-11-13`, each `match_count 2`); floor 0 detections over the 127
zero-match selectors; S1 → 4 collision selectors (3 real +
`bench-seed/s1 2026-11-03`); S2 → output identical to the unseeded run (the
census makes no referential claim — N/A, scope-gap).

No seeded or floor variant instance is written in place; the script's
`--seed`/`--floor` modes are in-memory variants over the frozen slice. The
frozen files' hashes are re-verified unchanged after the run.

## Arm B — catherine rule-lens/policy surface

Binary: `target/debug/catherine` (`cargo build --workspace`; the build is the
probe's execution, not a gate). Two sub-surfaces:

1. **Derived declared models** (`rules --values` path): the frozen model plus
   an additive `rule_presentation` section — `domain: capture`, one string
   column `capture_selector`, `policy: "DeclaredDisjoint"`, and one rule per
   frozen capture row whose predicate is `{"kind":"cmp","column":
   "capture_selector","op":"eq","value":"<frozen capture_selector cell>"}` and
   whose action is the frozen `capture_artifact` cell. Every predicate value
   and action is copied from the frozen instance tables — a transcription,
   never a computation. Rule names are the frozen capture row names
   (`cap_…`), so the witness names resolve to frozen rows.
2. **The projection bridge** (`rules --project` path): one rule per instance
   row synthesized from the frozen tables, `Policy::DeclaredDisjoint` fixed
   by the bridge (`crates/catherine-cli/src/projection/mod.rs`), exact-tuple
   domain = the observed tuples. Instance rows verbatim by construction.

Derivation commands (deterministic; recorded verbatim in the ledger):

```
python3 prototype/rule_lens_probe_derive.py \
  --from fixtures/probes/selector-state/selector-state.ct.json \
  --out fixtures/probes/rule-lens/selector-state-rules.ct.json
# and the same invocation with --from …selector-state-floor.ct.json → …rules-floor…
# and with --from fixtures/probes/bench/seeds/selector-state-seed-s1.ct.json → …rules-seed-s1…
# and with --from fixtures/probes/bench/seeds/selector-state-seed-s2.ct.json → …rules-seed-s2…
```

Commands:

```
target/debug/catherine rules fixtures/probes/rule-lens/selector-state-rules.ct.json \
  --values "capture_selector=sel_pba_JUDICIAL__2026_11_03,sel_nacional_primera__2026_11_03,sel_nacional_segunda__2026_11_13" --json
target/debug/catherine rules fixtures/probes/selector-state/selector-state.ct.json \
  --project capture --columns capture_selector --action capture_artifact --json
target/debug/catherine rules fixtures/probes/rule-lens/selector-state-rules-floor.ct.json \
  --values "capture_selector=<the 127 frozen floor selector names, comma-joined>" --json
target/debug/catherine rules fixtures/probes/rule-lens/selector-state-rules-seed-s1.ct.json \
  --values "capture_selector=<the 4 collided selector names incl. the seed>" --json
target/debug/catherine rules fixtures/probes/bench/seeds/selector-state-seed-s2.ct.json \
  --project capture --columns capture_selector --action capture_artifact --json
target/debug/catherine rules fixtures/probes/rule-lens/selector-state-rules-priority.ct.json \
  --values "capture_selector=<the 3 collided selector names>" --json
```

Registered outcomes:

- probe run (both sub-surfaces): exit 3, **three collision violations** —
  one per selector, each naming the two colliding rule rows and the shared
  input (E1–E3).
- floor: exit 6 (`underconstraint` hypotheses over the declared 127-selector
  domain — the empty capture table cannot cover the declared scope), **zero
  violation findings** — the floor control's arm-B leg; recorded, never
  scored (rubric: only exit-3 violations are detections).
- S1: exit 3, **four collision violations** (3 real + `bench-seed/s1`).
- S2: exit 1, labeled load error naming the dangling `capture_artifact` cell
  (the kw9.11 loader precedent) — fires, sanity pass.
- policy variant (`selector-state-rules-priority.ct.json`: the same
  transcription with `policy: "PriorityOrder"`): exit 0, silent — the pair is
  order-resolved, shadow-class, never a collision.

The 127 floor selector names and the collided-selector name lists are taken
verbatim from the frozen artifacts (`floor-selectors.json`; the slice's three
collided selectors; the seed model's four). The full command strings with the
names expanded are recorded in the ledger entry.

## Ledger entry template

Per the domain-probes spec, entries `0001` (floor control) and `0002` (probe)
under `ledger/probes/rule-lens/`, sealed content-addressed
(`NNNN-<8hex>.json`), each with:

- `seq`, `kind: "kata"`, `recorded` (date),
- `kata.kata_id: "probe-rule-lens-selector-collision"`, `name`, `control`,
  `series_report` (this protocol), `series_verdict` (the README decision
  rule, fixed in advance), `fixture` + `fixture_hash` (the frozen artifact
  the run consumed),
- `kata.expected[]` — the README's rows E1–E6/S1/S2, each with
  `series_class` and `catherine_class`,
- `kata.run.commands[]` — every command verbatim + its exit code and
  verdict-bearing output summary, `kata.run.findings[]` — the exit-3
  findings verbatim,
- `kata.comparison.conventional_baseline` — arm A's artifact sha256 +
  its executed detections (mode `executed`), `fired[]`, and
  `unique_detections[]` (the `TP_B ∖ TP_A` arithmetic),
- `kata.expressibility_gap[]` — why the check surface had no cardinality
  claim, citing `ledger/probes/selector-state/0002-5c22c00e.json` (kw9.5),
- `kata.discrepancies[]` — verbatim `D1…` strings,
- `kata.procedure[]` — the executed steps in order,
- `kata.verdict` — from the closed vocabulary,
- `model_hashes[]` — every model artifact consumed or produced,
- `rollup: null` (no rollup entry in this sequence, per the bench precedent).

Validation: `cargo run -p catherine-ledger --bin ledger-check -- ledger/probes/rule-lens`
must print `ok`; a neighbor sweep through the throwaway-validator precedent
(bench README, Ledger section) is recorded if run, and any throwaway is
deleted after the run.
