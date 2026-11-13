# Capture-selector slice — the `rules --project` acceptance fixture

Grounded fixture for step 1 of
`.wai/projects/catherine/research/2026-11-13-post-granularity-capability-gaps.md`
(`rules --project`: project instance rows onto a rule table, no format change).

## Provenance (read-only)

| Artifact | Path | Value |
|---|---|---|
| Batch working corpus | `bogart/data/backfill/corpus.sqlite` | 9,315,844,096 bytes, mtime 2026-11-13 14:32:32 -0300 |
| Generator | `prototype/bogart_capture_slice.py` | committed before the run |
| Fixture | `fixtures/probes/capture-selector/capture-slice.ct.json` | sha256 `85178ffe7e4790f06e0a7eb63bca8c65f6c355c19b2345fde9091c5f0cdc4771` |

Regenerate: `python3 prototype/bogart_capture_slice.py --slice <path>`.

## The slice

Window `edition_date >= 2026-11-01` of `backfill_captures`, one object per
stored table, two arrows to value objects:

| Arrow | src → tgt | Storing column |
|---|---|---|
| `capture_source` | `backfill_captures` → `source` | `backfill_captures.source` |
| `capture_edition_date` | `backfill_captures` → `edition_date` | `backfill_captures.edition_date` |

38 captures → 35 distinct `(source, edition_date)` pairs, 9 edition dates,
5 sources. Every cell resolves: `catherine check` exits 0 with zero findings.

## Census (executed 2026-11-13, `--census`)

| Set | Captures | Pairs | Selectors | Ambiguous (≥2 captures) | Zero-match |
|---|---|---|---|---|---|
| slice (`>= 2026-11-01`) | 38 | 35 | 65 | **3** | 30 |
| corpus | 1,377 | 1,374 | 1,441 | **3** | 67 |

The three ambiguous selectors are `nacional/primera 2026-11-03`,
`pba/JUDICIAL 2026-11-03` (the pilot's two `bogart-4wt` defects, still
present) and `nacional/segunda 2026-11-13`. Selector count exceeds capture
count because the universe is `DISTINCT (source, edition_date)` over
`backfill_captures` UNION `backfill_partitions` — the pilot's selection
semantics; partitions contribute the zero-match rows.

## Acceptance (pre-registered)

`catherine rules fixtures/probes/capture-selector/capture-slice.ct.json
--project backfill_captures --columns capture_source,capture_edition_date`

| # | Expectation | Falsified by |
|---|---|---|
| A1 | exactly **3** `violation` findings, one per ambiguous selector, each carrying both capture rows as the witness pair (overlap under `DeclaredDisjoint`) | any other count, or a witness pair that is not the two captures behind an ambiguous selector |
| A2 | **no** finding on the 30 slice zero-match selectors — the projection's domain is the observed tuple set, so a selector with no capture row is not an input | a `gap`/`underconstraint` finding on this run |
| A3 | declaring an extra input explicitly (`--values source=nacional/primera edition_date=2026-11-07`, a real zero-match selector) yields a `gap` finding naming it | no finding, or a finding for an undeclared input |
| A4 | exit code 3 on the A1 run (violation outranks any underconstraint) | any other exit code |

The claim lives outside the document hash and the ledger: this run is evidence
about a *projection*, not a sealed model claim. State that wherever the result
is cited.