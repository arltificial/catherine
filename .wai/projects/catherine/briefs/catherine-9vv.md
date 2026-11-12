# Brief: catherine-9vv — P6 kata validation ledger (PROJECT STOP CONDITION)

Ticket: P1. This is the stop condition: a `killed` verdict STOPS the project (negative result written, ledger kept; P7 `catherine-alr` / P8 `catherine-nq5` do not start). Verdicts are derived from tool findings via the recorded procedure in `openspec/changes/add-kata-validation-ledger/design.md` — no goalpost moves, negative control FIRST.

## Source material (validated expectations — read before anything)

- `/var/home/sasha/para/areas/dev/gh/ak/matadero/series/ct-catlab/SERIES.md` — the verdict ledger: kata-1 Bowling (survived-scoped: conventional 5/5 behavioral vs categorical 3/5 structural + construction-time inexpressibility), kata-2 Gilded Rose (categorical 2/6 structural: Sulfuras frozen-quality + post-concert pass), kata-3 Vending (structural F2 bounds/F3 command-vocabulary/F5 collision; F6 REFUTED by executed counterexample), kata-6 FizzBuzz (KILLED negative control: conventional 3/3, categorical zero unique detections).
- Per-kata reports: `runs/2026-11-02-kata0{1,2,3,6}-local-attempt001/REPORT.md` in that series dir.
- NOTE: the proposal text says Vending is "strongest validated structural result, 6/6" — the series ledger says otherwise (3 structural catches, F6 refuted). The LEDGER is ground truth (record discipline applies to us on our own tool): port what the series actually validated, record the discrepancy in the ledger entry, never inflate.

## Target paths

- `fixtures/katas/` — the four kata fixtures (FizzBuzz, Bowling, Gilded Rose, Vending) as browser-loadable `.ct.json` models
- `ledger/` — the validation ledger (entries content-addressed via the P0 hash port: `catherine_core::hash::{canonical_bytes, sha256_digest, model_hash}`; append-only; supersession supported)
- `web/` — task 6: kata fixtures wired into the web view (load, check, findings panel per kata)
- ledger validator + harness: follow the kernel-crate layout the implementor of 9b5 established; a small `crates/catherine-ledger/` crate is acceptable if the validator needs a home (kernel stays genesis-free)
- `openspec/changes/add-kata-validation-ledger/tasks.md` — tick boxes

## Order (tasks.md, enforced)

1. RED task 1: ledger validator rejects — entry without prior negative control; verdict outside `killed|survived|inconclusive|invalid-run`; entry without model-hash references.
2. GREEN task 2 FIRST OF ALL LEDGER WRITES: FizzBuzz negative control → expected verdict "no categorical advantage" (series says killed) → ledger entry #1. If catherine's executed checks find a UNIQUE detection on FizzBuzz, that REFUTES the negative control — STOP and report before touching any other kata; do not proceed to Bowling.
3. Tasks 3–5: Bowling (temporal semantics — expected PARTIAL, record honestly per series: history-dependence is a validated CT limit), Gilded Rose (state-shape invariants), Vending (ground in the real series result, see NOTE).
4. Task 6: wire fixtures into web view.
5. Task 7: content-addressed append-only ledger w/ supersession.
6. Task 8: project-verdict rollup entry; verdict vocabulary per procedure; P7/P8 gates wired (beads stay blocked unless `survived`).
7. Task 9: expressibility-gap field per kata (injectivity-flavored invariants named, not papered over).
8. Task 10: extract kata-fixture harness shared with P8/P9.

## Discipline

Edit only; `cargo check`/targeted `cargo test` of your own new test targets allowed. NEVER: fmt/clippy/full workspace gates, `just ci`, ah/pretender/bd/lefthook, git commits, bd mutations (bd close/export + the P7/P8 gate flip are orchestrator-reserved). If any acceptance command is impossible as specified, STOP and report — do not substitute.

## Acceptance (report with evidence)

1. Validator RED tests (task 1) pass against the validator.
2. Ledger entries #1–#4 present in order: FizzBuzz (negative control) → Bowling → Gilded Rose → Vending; each with model-hash refs, verdict from the closed vocabulary, expressibility-gap field.
3. Project rollup entry derived strictly via the recorded procedure from the four entries.
4. FizzBuzz executed BEFORE the other three (evidence: entry ids/order + run log lines).
5. Web view loads + checks each kata fixture (parity with CLI findings encoding).
6. tasks.md boxes ticked; exact orchestrator commands listed; "bd close pending orchestrator" noted.
