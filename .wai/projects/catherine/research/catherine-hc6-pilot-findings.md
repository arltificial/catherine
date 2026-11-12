# catherine-hc6 pilot findings — bogart pilot (models 1–3)

Completed 2026-11-12. Salvage + completion of the run the previous
implementor died on (API-credit failure at ~40%: it had reported a RED
fixture that fired and a live census, but nothing was verified, reported, or
committed).

## Salvage report (prior partial state, verified not trusted)

- `test-resources/catherine-pilot/red_drifted_anchor.ct.json` — sound; RED
  re-verified (see below). Kept.
- `model1_fts_anchor.ct.json` / `model2_identity_domains.ct.json` /
  `model3_capture_selector.ct.json` — sound and grounded (schema cross-checked
  against the real sqlite tables; invariants traced to bogart
  `corpus.storage.fts_text_layer` / `corpus.storage.stable_ids` /
  `backfill.inspection` selection semantics). Kept as-is.
- `scripts/pilot_ct_instances.py` — kept; one real defect fixed: the
  full-run model 1 page query was join-closed
  (`WHERE document_id IN (docs)`), silently excluding exactly the rows the
  totality claim must cover. Full run now enumerates all 74,257 pages and
  emits null cells (dangling witnesses) for unresolvable `document_id`.
  Regenerating the committed slice models after the fix is byte-identical
  (md5-verified), so the fix only affects `--full-m1`.
- Live-run artifacts the dead run left in `/tmp/hc6-live/` — regenerated
  fresh; outputs byte-identical (md5 + diff verified).
- Missing and now delivered: pilot report, evidence JSON, CI workflow,
  defect filing, mirror file, bogart commits.

## RED evidence (task 1)

`catherine check test-resources/catherine-pilot/red_drifted_anchor.ct.json`
(run from the catherine repo; binary = `catherine` 0.1.0):

```
violation: eq-0 document_id∘edition_date ≠ hit_edition_date
  at row backfill_read_pages#2: lhs=edition_date#0 rhs=edition_date#1
dangling: arrow document_id
  at row backfill_read_pages#3
exit=3
```

Exit 3 = violation class wins the P3 precedence (violation > dangling);
the JSON envelope carries both findings with witnesses. Verified under
catherine main `7f11d48` AND pinned `268c142` — identical.

## Models 1–3

All `.ct.json` in `../bogart/test-resources/catherine-pilot/`, rows generated
from the frozen corpora by `scripts/pilot_ct_instances.py` (row-name mapping:
non-alnum → `_`, prefixed `pg_/doc_/ed_/ver_/src_/cap_/sel_/aviso_/ldoc_`).

- M1 FTS-anchor (`corpus.storage.fts_text_layer`): slice exit 0 (289 pages);
  live full exit 0 over 74,257 pages / 924 docs / 210 editions — totality
  holds (0 null, 0 unresolvable `document_id`). Drift half is enforced
  structurally by today's join — exercised by the RED fixture, recorded as a
  grounding note, not a defect.
- M2 identity domains (`corpus.storage.stable_ids`): exit 0, **vacuous by
  design** — disjointness is inexpressible in v1 (needs rule-lens P8);
  recorded as expressibility gap, not papered over.
- M3 capture selector (`backfill.inspection/export!` selection semantics):
  slice exit 0 (3 unique-match selectors); live full exit 4 with 69 dangling.

## Live run + triage (task 8)

Census: 230 captures, 295 selectors; histogram {0-match: 67, 1-match: 226,
2-match: 2} — matches the dead implementor's reported census (2 ambiguous +
67 gap), now witness-verified.

- **Real: 2 findings → bogart-4wt** (filed in bogart bd, P2 bug, open).
  `nacional/primera 2026-11-03` and `pba/JUDICIAL 2026-11-03` each match TWO
  captures: a proper revision capture plus a pickup-lane
  `pickup-unknown-observation` capture under the same (source, edition-date).
  bogart's `inspection.clj` date selection fails `capture_ambiguous` → those
  selectors cannot export/inspect by date today. Model mirrors the selection
  SQL exactly → faithful witnesses.
- **Noise: 67 findings.** All are `backfill_partitions` state `planned`
  selectors with zero matches (dates 2026-10-23…2026-11-11, five sources).
  Cause: finding granularity — the model treats the whole planner census as a
  totality claim; a planned partition is declared work intent and
  `capture_unavailable` is its expected pre-capture state. Historical
  completion is explicit backlog (bogart-cte); the recent tail reflects the
  frozen snapshot's acquisition window. v1 cannot scope checks to "selectors
  that must resolve" — that is the rule-lens totality/policy machinery.
- Accounting: 69/69 classified, none suppressed. Full witness list (selector,
  partition state, match count, cause, matching capture ids) in bogart
  `docs/evidence/catherine-pilot-2026-11-11.json`.

## Expressibility gaps (per add-rule-lens design.md Decision 1)

1. M2 disjointness: `overlap` under `DeclaredDisjoint` → violation; `gap` →
   underconstraint (hypothesized). Both P8; model 2 stays spec-only.
2. M3 match multiplicity: preimage > 1 (ambiguous) and preimage = 0
   (unavailable) both surface as v1 **dangling** on the null `match` cell;
   Decision 1 would map them to violation / underconstraint respectively once
   a policy/totality is declarable.
3. M3 alias widening (metadata `edition-dates`) is modeled but unobservable
   in this corpus — model-precision limit.
4. M1 drift half: no independent per-hit edition column exists in serving
   today; equation holds by construction (see salvage note on the generator
   fix).

## CI wiring (task 7)

`../bogart/.github/workflows/ci.yml` (new): bogart checkout → catherine
checkout pinned `268c142` (needs `CATHERINE_PAT` secret, contents:read) →
actions/cache on `catherine/target` keyed `hashFiles('catherine/Cargo.lock')`
→ `cargo build -p catherine-cli` → `catherine check` per committed model
(fails on any finding-class exit; no text parsing) → RED self-test asserting
exit 3. No docker/services. Validation: YAML parses (pyyaml); actionlint
unavailable on this host (release download returned non-gzip); not run live
(push is gated).

## Bogart commits

See bogart `git log` (lowercase-prefix convention). Nothing pushed; catherine
repo untouched except this research file (and the brief, both under
`.wai/projects/catherine/`).

## Defects filed in bogart bd

- `bogart-4wt` — inspection/export! date-selector fails `capture_ambiguous`
  when a `pickup-unknown-observation` capture shares (source, edition-date)
  with a revision capture (P2, bug, open; not fixed by the pilot).
