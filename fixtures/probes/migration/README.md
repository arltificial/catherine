# Probe 1 — migration Δ at pilot scale (pre-registration)

Committed **before** the probe run (`add-domain-probe-validation`
tasks 5-7, beads `catherine-1od`). Nothing here may be edited after the
run: a correction is a superseding ledger entry.

## Fault set

Three committed fixtures, each a total format-1.1 `schema_map` document
shaped like the bogart pilot corpus (documents → pages → captures, a
serving projection):

| Fixture | Fault (probe catalog row) | Injected loss | Row counts |
|---|---|---|---|
| `floor-visible-loss.ct.json` | `floor-visible-loss` (control) | `RawCapture` (6 rows) assigned `null`; its two arrows dropped and valueless | source 8/16/6/4 → target 8/16/4 |
| `lost-data-cells.ct.json` | `lost-data` | `capture_note` (4 valued cells) and `captured_page` (6 valued cells) dropped; **every row kept** | 8/16/6/4 unchanged |
| `non-invertible-collapse.ct.json` | `non-invertibility` | `RawPageCapture` (6 rows) collapses onto `RawPage` through `capture_of`; no row is lost, 6 elements are unrecoverable | source 8/4/16/6 → target 8/4/16 |

Every target row count above is the Δ engine's documented base-preimage
convention (`RawPage` is the sorted-first preimage of `ServingPage`).

## Comparison rule (fixed before the run)

`unique_detections` is arithmetic, not judgement: **probe findings
minus baseline-identified elements**, where an *element* is one lost row
(`lost_row:<object>#<row>`), one lost cell (`lost_cell:<arrow>#<row>`),
or one round-trip witness (`round_trip:<object>#<row>`).

An arm **identifies** an element only when its output names it:

- a dropped table identifies every row of that table (the table is
  absent, so its lost row set is exactly its row set) — this is the
  floor control's "fully visible" property;
- a dropped column (`DROP COLUMN` lint) names the column, never which
  rows carried values;
- a row-count/checksum delta names the table and the count, never the
  elements;
- no conventional arm expresses Δ∘Δ′ at all, so a round-trip witness is
  never identified by the baseline.

## Baseline arm (task 6)

`prototype/migration_baseline.py`, committed and version-pinned by
sha256 **before** the probe run, encodes exactly the conventional
practice the dispatch synthesis named: statement-level migration linting
(Atlas/Flyway class: `DROP TABLE`, `DROP COLUMN`, table merge,
non-transportable path) plus a row-count/checksum dry run over the
declared map. It is executed by `--assert` against the frozen
`baseline_expected` detections in `faults.json`; a mismatch fails
loudly.

Pre-registered baseline detections:

| Fixture | Baseline detections | Elements identified |
|---|---|---|
| floor control | `lint:drop-table:RawCapture`, `lint:drop-column:capture_note`, `lint:drop-column:captured_page`, `dry-run:row-count-delta:RawCapture:6->0` | all 6 `RawCapture` rows |
| lost-data | `lint:drop-column:capture_note`, `lint:drop-column:captured_page` | none (columns named, cells not) |
| non-invertibility | `lint:table-merge:ServingPage<-RawPage,RawPageCapture`, `dry-run:row-count-delta:ServingPage:22->16` | none |

## Pre-registered probe detections

- floor control: 6 `lost_row` findings, `RawCapture` rows 0–5; no
  other finding; unique set **empty** → verdict `killed` (the control
  must self-negate).
- lost-data: 4 `lost_cell` findings on `capture_note` (rows 0–3) and
  6 on `captured_page` (rows 0–5); unique set = all 10 named cells.
- non-invertibility: `catherine migrate` emits **no** finding (the
  collapse hides the loss); the round-trip arm proves
  `RawPageCapture` row 0 unrecoverable (forward image
  `ServingPage` row 0; the candidate inverse returns `RawPage` row 0);
  unique set = that one witness.

Unique counts: 0 / 10 / 1. Verdicts by the recorded procedure
(`harness::derive_probe_verdict`): control `killed`; lost-data and
non-invertibility `survived` (unique non-empty, baseline arm executed).

## Falsifiers

- Any probe finding outside the pre-registered sets — the fixture's
  expected rows move; record the discrepancy, never edit the catalog.
- Baseline `--assert` mismatch — the baseline arm is wrong or the
  fixture changed; re-pin only with a superseding entry.
- Round-trip returns no proof, or names a different element — the
  non-invertibility row is refuted (verdict `killed`), and the null
  result is the deliverable.

## Execution

```
python3 prototype/migration_baseline.py --assert                      # baseline arm + frozen expectations
cargo test -p catherine-ledger --test probe_migration_fixtures        # probe arm, frozen assertions
cargo run -q -p catherine-ledger --bin ledger-check -- ledger/probes/migration
```