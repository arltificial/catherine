# Selector→capture state re-model — bogart selector subdomain (pre-registration)

Committed BEFORE the run (RED). Ticket `catherine-kw9.5`; fresh-domain
precision probe, U1-U2 (≥50% precision). Domain decision (recorded in the
ticket, not re-litigated here): candidate D — the bogart selector→capture
state subdomain. Census semantics are the prototype's, verbatim:
`prototype/bogart_storage_granularity.py --census` (a capture `(src, ed)`
matches selector `(s, e)` iff `src == s` and (`ed == e` or `e` is in the
capture's `metadata_json` `edition-dates` alias set)).

## The rule under test

Model one arrow per **stored, independently writable reference**; the
selector/capture queue state is what the DDL actually stores: each
`backfill_captures` row stores a composite reference to an implicit
selector (`source` + `edition_date`, alias-resolved) and a reference to the
captured artifact (`sha256`); each `backfill_partitions` row stores the
same composite selector reference; the serving slice stores
`read_pages → read_documents` (composite) and
`read_documents → edition_date`.

## Provenance (read-only; recomputed at extraction, 2026-11-25)

| Artifact | Path | sha256 |
|---|---|---|
| Serving projection (slice rows for the collision editions) | `bogart/data/backfill/delivery-2026-11-11-november/serving.sqlite` | `30ec194777efc912c4848e06faca1e7451d3fb192690856e51ef98e07a413c34` (matches the value recorded 2026-11-13) |
| Batch working corpus (selector census + captures; LIVE, opened `mode=ro`) | `bogart/data/backfill/corpus.sqlite` | `1ac681dbe8874643c4c40d4a5bce2a4f0cefbe9bbf22deaa33c6448774f000b5` (at extraction 2026-11-25; the 2026-11-13 run did not record one) |
| Census semantics generator | `prototype/bogart_storage_granularity.py` | committed before the run |
| Frozen census | `fixtures/probes/selector-state/census.json` | `154913904e7544930b28dda7e8bca3ae903a0bfb06342e937fd67b8967e4be3d` |
| Frozen slice rows | `fixtures/probes/selector-state/slice.json` | `840e81c843ce371fbdb73c071dda4a9080f94bbb14da016d2f72dc065a5280c5` |
| Frozen model + instance | `fixtures/probes/selector-state/selector-state.ct.json` | `3cc5314d614d57e4eb8fa33420a2b1ee582c4e548e2268ce97bb23b72143163a` |

### Snapshot rule and census reconciliation

The corpus is live (concurrent ingest): a raw census today does **not**
reproduce the recorded ground truth. The frozen census applies a snapshot
cutoff `captured_at <= 1794586142000` (ms epoch), the unique cutoff that
reproduces the recorded capture count exactly:

| Count | Recorded 2026-11-13 | Frozen snapshot | Reconciled? |
|---|---|---|---|
| captures | 1,377 | 1,377 | yes — exact |
| ≥2-match selectors | 3 (`nacional/primera 2026-11-03`, `pba/JUDICIAL 2026-11-03`, `nacional/segunda 2026-11-13`) | the same 3, each with exactly 2 matches | yes — exact |
| selectors | 1,441 | 1,501 | **no — +60** |
| 0-match selectors | 67 | 127 | **no — +60** |

The +60 is fully explained and disclosed: `backfill_partitions` carries no
timestamp column, and 60 new distinct partition selectors (all 0-match)
have been added by the running backfill since the recording. They are not
reconstructable from the live database; the frozen census keeps all 127
zero-match selectors and the drift note lives in `census.json`. The probe's
ground truth (the 3 collisions) reproduces exactly.

## The frozen slice (custody)

- The 3 collision selectors' captures: 6 rows (`backfill_captures`), two
  per collision. Each pair shares the **same `sha256`** and differs only in
  `job_id`/`revision` (e.g. `revision = pickup-unknown-observation` vs
  `3`) — two stored capture rows for one artifact. This is the concrete
  shape of the known-real defect.
- Their `backfill_partitions` rows: 3 (all `state = planned`).
- Serving rows for the collision editions: 5 `backfill_read_documents`
  rows + 454 `backfill_read_pages` rows, all `edition_date = 2026-11-03`.
  **Slice boundary:** the delivery holds no `2026-11-13` edition rows; the
  `nacional/segunda 2026-11-13` collision lives corpus-side only (its 2
  captures are frozen above).
- The 127 frozen zero-match selectors as bare selector rows (P2's
  denominator).

## Model (what the rule admits, and only that — from actual DDL columns)

Objects: `capture`, `partition`, `selector`, `artifact`, `read_page`,
`read_document`, `edition_date`. Instance: 602 rows (selector 130,
capture 6, partition 3, artifact 3, read_page 454, read_document 5,
edition_date 1).

| Arrow | src → tgt | Storing column(s) |
|---|---|---|
| `capture_selector` | `capture` → `selector` | `backfill_captures.source` + `backfill_captures.edition_date` (composite; alias resolution via `metadata_json` `edition-dates`) |
| `capture_artifact` | `capture` → `artifact` | `backfill_captures.sha256` |
| `partition_selector` | `partition` → `selector` | `backfill_partitions.source` + `backfill_partitions.edition_date` |
| `page_document` | `read_page` → `read_document` | `backfill_read_pages.document_id` + `backfill_read_pages.version` (composite FK into the documents PK) |
| `document_edition` | `read_document` → `edition_date` | `backfill_read_documents.edition_date` |

Dropped: match-count rollups, edition-date alias sets as separate selector
rows beyond the alias-resolution rule, any selector→capture arrow (no
stored column points from a selector at a capture — the direction is
capture stores selector), any per-hit selector column in serving (none
exists).

## Pre-registered predictions (falsifiable)

| # | Prediction | Falsified by |
|---|---|---|
| P1 | The 3 collision selectors produce **exactly 3 real findings** (one per collision) | any other count |
| P2 | The 127 zero-match selectors produce **zero findings** | any finding on a zero-match selector |
| P3 | Negative control (FizzBuzz-class): rule-based derived relations — the computed match-count histogram and derived edition rollups — are modeled as **prose intent only** (`equations: []`); the tool must stay **silent** on them | any finding attributable to a rule-based expectation (a computed/derived relation reported as stored-state violation) |
| P4 | Total findings **N** with precision = 3/N scored against the census ground truth | — (scoring rule, not a prediction) |

## Scoring rubric and decision rule (fixed in advance)

A finding is **REAL** iff it names a row or key involved in a known
collision — one of the 3 selectors `nacional/primera 2026-11-03`,
`pba/JUDICIAL 2026-11-03`, `nacional/segunda 2026-11-13`, or one of their 6
capture rows / 3 artifact rows. Everything else is **noise**.

Precision = real / total findings (0/0 = undefined). U1-U2 is answered
**yes** iff precision ≥ 50% (i.e. N ≤ 6 with 3 real; or N < 6 with real/N ≥
½ — concretely: 3 real findings among at most 5 total, since 3/6 = 50%
meets the bound).

Verdict vocabulary, mapped BEFORE the run:

| Observed outcome | Verdict |
|---|---|
| P1 confirmed (exactly 3 real findings), P2 confirmed, P3 confirmed (silent on derived relations) | `survived` |
| Zero findings on the slice, or real findings < half of total (precision < 50%), or findings attributable to derived relations | `killed` |
| Run does not complete (parse/load failure, engine error), or outcomes mix confirmed and falsified predictions such that neither row above applies | `inconclusive` (or `invalid-run` for a tool failure, per the ledger validator) |

Floor control: a self-negating negative-control entry (`killed`) precedes
the probe entry in the probe ledger sequence (`NegativeControlMissing`).
GREEN runs:

```
catherine check fixtures/probes/selector-state/selector-state.ct.json
```

Input construction only happened in RED (this fixture is frozen data, not
a check run). Exit class 8 = the granularity lint questioned the model's
shape; kernel findings carry their usual classes. Findings are scored with
the rubric above and recorded in the ledger with this catalog hash.
