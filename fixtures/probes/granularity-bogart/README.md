# Storage-granularity re-model — bogart FTS anchor (pre-registration)

Committed BEFORE the run. Ticket `catherine-8v9`; protocol in
`.wai/projects/catherine/research/2026-11-13-granularity-rule-calibration.md`,
step 2 ("re-model at the rule's granularity"), step 3 (real row slice +
`catherine check` + triage), step 4 (compare with the pilot's 2/69), step 5
(falsifier).

## The rule under test

Model one arrow per **stored, independently writable reference**; drop
relations that exist only in prose, plans, or derived views.

## Provenance (read-only)

| Artifact | Path | sha256 |
|---|---|---|
| Serving projection (slice + full instance rows) | `bogart/data/backfill/delivery-2026-11-11-november/serving.sqlite` | `recorded at run time` |
| Batch working corpus (selector census only) | `bogart/data/backfill/corpus.sqlite` | `recorded at run time` |
| Generator (harness) | `prototype/bogart_storage_granularity.py` | committed before the run |

DDL the model is held to (`backfill_read_documents` /
`backfill_read_pages`):

```sql
backfill_read_documents(document_id TEXT, version TEXT, source TEXT,
                        edition_date TEXT, metadata_json TEXT, total INTEGER,
                        selected INTEGER, PRIMARY KEY(document_id, version))
backfill_read_pages(id INTEGER PRIMARY KEY, document_id TEXT, version TEXT,
                    page INTEGER, status TEXT, text TEXT, text_hash TEXT,
                    result_id TEXT, error_code TEXT,
                    UNIQUE(document_id, version, page))
```

## Model (what the rule admits, and only that)

| Arrow | src → tgt | Storing column(s) |
|---|---|---|
| `document` | `read_pages` → `read_documents` | `backfill_read_pages.document_id` + `backfill_read_pages.version` (composite FK into the documents PK) |
| `edition_date` | `read_documents` → `edition_date` | `backfill_read_documents.edition_date` |

Dropped against pilot model 1 (`catherine-hc6`): `document_id` as a
single-column arrow (the real reference is composite), `hit_edition_date`
(derived in `read_service`; no per-hit edition column exists in serving),
and the equation `document_id ; edition_date = hit_edition_date` (both sides
collapse to the same stored column — a tautology, not a check).

## Pre-registered predictions (falsifiable)

| # | Prediction | Falsified by |
|---|---|---|
| P1 | Slice run (edition 2026-11-11: 3 documents, 289 pages) exits **0** with **zero findings** | any finding on the slice run |
| P2 | Full-corpus run (924 documents, 74,257 pages) exits **0** with **zero findings** | any dangling/violation on the full run |
| P3 | No non-tautological equation is writable against stored columns; the model carries `equations: []` | exhibiting a stored two-path invariant in this subdomain |
| P4 | The rule's model cannot produce the pilot's two real findings (`bogart-4wt`, selector ambiguity): `backfill_captures` / `backfill_partitions` store no selector→capture column | a storing column for the selector match existing in the DDL |

Decision rule, fixed in advance: a *better real/noise ratio than 2/69* requires
a defined ratio. Zero findings make it undefined (0 real, 0 noise). Protocol
step 5 then applies: the rule is **not validated** on this second domain, the
docs label stays **PROPOSED**, and the measurement — including the recall
cost P4 names — is attached as the negative result.

## Result (observed 2026-11-13, after the pre-registration commit `c25e507`)

Provenance at run time: serving projection sha256
`30ec194777efc912c4848e06faca1e7451d3fb192690856e51ef98e07a413c34`
(1,372,024,832 bytes, mtime 2026-11-11 19:50:42 -0300); batch corpus
9,315,840,000 bytes, mtime 2026-11-13 13:46:33 -0300 (live, concurrent
ingest — read by the census only); slice fixture sha256
`f8dd74ccee1669ae38645a9d0c7e5e4789e7041a6edab62f9478600dd849d11d`.

| Run | Instance | Findings | Real | Noise | Exit |
|---|---|---|---|---|---|
| slice | 3 documents / 289 pages / 1 edition | 0 | 0 | 0 | 0 |
| full | 924 documents / 74,257 pages / 210 editions | 0 | 0 | 0 | 0 |

P1, P2, P3 confirmed. P4 confirmed and strengthened: the live selector census
(1,377 captures / 1,441 selectors) finds **three** ≥2-match selectors —
`nacional/primera 2026-11-03` and `pba/JUDICIAL 2026-11-03` (the pilot's two
real `bogart-4wt` defects, still present) plus `nacional/segunda 2026-11-13` —
and 67 zero-match selectors. The storage model, run against the same system,
reports **zero** findings: neither class is expressible without a storing
column. `storing_column_for_selector_match` = none (DDL, not opinion).

### Triage and comparison with 2/69

There is nothing to triage: 0 findings, 0 real, 0 noise. The pilot's 69
findings were 2 real / 67 noise (2.9% real); the re-model's ratio is
**undefined**, not better. What the run does establish:

- the 67 pilot noise findings are *structurally excluded* — planner-intent
  rows (`backfill_partitions` state `planned`) are not modeled, because they
  store no reference into captures;
- the 2 pilot real findings are *structurally excluded too* — same cause. The
  rule buys precision by discarding the only real defects the pilot ever
  found on this system.

Per the pre-registered decision rule (step 5): the rule is **not validated**
on this second domain; the docs label stays **PROPOSED** and this measurement,
including the recall cost, is attached as the negative result. The rule's
defensible scope is narrower than "all arrows": it governs *structural*
(reference/totality) claims. Derivation and selection claims — the ones that
found `bogart-4wt` — must be declared as such and checked by the rule-lens /
policy machinery, not suppressed by the granularity rule. Reproduce with
`python3 prototype/bogart_storage_granularity.py --slice|--full|--census`.