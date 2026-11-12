# ro5u review — add-kata-validation-ledger (catherine-9vv, PROJECT STOP CONDITION)

Full-diff lead-side review. Diff: `crates/catherine-ledger/` (validator,
entry shapes + sha256 content addresses, recorded-procedure rollup,
pre-registered catalog, fixture harness, `ledger-check`/`ledger-record`
bins, 270-line RED test file — 14+3 tests green), `fixtures/katas/` (10
documents: 4 kata models + fault variants + load-rejection probes),
`ledger/` (entries #1–#4 + rollup #5, sealed), `web/` (kata preset
select + fetch wiring through the existing load→check→render path),
ticked tasks.md (11/11). Implemented across three dispatch phases plus
orchestrator stabilization (the credit-dead first implementor's TDD-RED
scaffold completed inline: stubbed validate_dir/derive made real, the
missing catalog/harness modules and bins written — disclosed, not
silent; credits made corrective dispatch impossible twice).

The stop condition, executed in order: FizzBuzz negative control ran
FIRST (entry #1, seq 1, killed, zero unique detections — a stop-check
assertion gated the recorder); only then Bowling (#2, survived-scoped
with three named gaps), Gilded Rose (#3, both pre-registered structural
catches fired), Vending (#4, F2/F5 fired, F3 construction-time, F6 gap
+ proposal discrepancy recorded). Rollup #5 derived strictly via the
committed `rollup::derive` over the four entries: **verdict survived
(scoped)**, control gate passed — the project continues; P7/P8 unblock
on survived (bead flip orchestrator-reserved, executed after this
review).

Orchestrator verification (live): `ledger-check` → `ok: 5 entries
(4 kata), rollup verdict: survived`; kata runs re-executed (FizzBuzz
exit 0 clean — the control finding nothing is the self-negation;
Bowling miswired fault exit 3; full Vending reproduction 69→n/a
(hc6); gilded-rose fault exits recorded in entries); headless-browser
smoke: kata preset select loads Gilded Rose → check → findings panel +
5-node schema graph rendered (screenshot archived); `just ci` EXIT=0;
`openspec validate` 10/10 strict.

Five passes:

1. Correctness: verdicts derived, never asserted — the rollup embeds
   PROCEDURE verbatim and derive() implements exactly those steps; the
   validator re-computes ids from canonical bytes and refuses partial
   ledgers.
2. Contract drift (class B): expectations pre-registered in
   `catalog.rs` from the series ledger BEFORE the runs; the proposal's
   Vending "6/6" vs the series' F2/F3/F5+F6-refuted discrepancy is
   recorded in catalog + entry #4 — the series record is ground truth.
3. Tests: validator RED file (14 tests) pins every named rejection;
   entry id stability tests; harness executes the same kernel path as
   CLI/wasm. No snapshots (structural assertions; findings deserialize
   into contract types).
4. Negative matrix: no new CLI flag/surface; the ledger bins are new
   operator surfaces whose failure modes are the named Violations
   (exit 1) — covered by the RED tests.
5. Regression risk: workspace gains one crate (genesis-free, consumes
   core's public artifacts); web/ gains a preset select through the
   existing path; kernel untouched.

Findings + dispositions:

- [FIXED during verification] pretender caps forced decomposition
  (validate_dir 104→phase helpers; derive 65→control_gate/per_kata
  helpers; expectations 77→per-kata fns); clippy -D warnings fixed
  (manual_contains, identical blocks, test-file vec!/Path lints).
- [DEFERRED, LOW, written reason] task 10's "shared with P8/P9" is
  satisfied by the crate boundary (harness module) rather than a
  fixture-format abstraction — P8/P9 adoption may require a second
  iteration; the boundary is the right seam to iterate from.
- [NOTE] entries #2–#4 record honest zeros: `unique_detections: []`
  everywhere (parity with conventional baselines, parity never
  claimed) — the series' own honesty discipline applied to the tool.

max_critical = 0. Verdict: PASS.
