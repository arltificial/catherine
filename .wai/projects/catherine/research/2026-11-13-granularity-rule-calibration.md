# Granularity rule — PROPOSED, pilot calibration, and the re-model protocol

Date: 2026-11-13 · Tickets: `catherine-kq1` (rule + pilot calibration),
`catherine-8v9` (second-domain re-model) · Status: rule PROPOSED and
**re-scoped** — second-domain test EXECUTED 2026-11-13, validating nothing;
see "Second-domain re-model" below.

## The rule

Model one arrow per relation that exists as a **stored, independently
writable reference**; never encode relations that exist only in prose,
plans, or derived views.

**Pre-build test:** for every arrow you intend to declare, name the storing
column (or table) it reads. If more than a third of the arrows have no
storing column, you are modeling the plan — stop and re-scope. The one-third
threshold is arbitrary (calibrated against the bogart pilot's 2/69 triage),
a smell threshold, never a validated cutoff.

## Why it matters

The pilot produced 69 findings, 2 of them real (`catherine-hc6`): the checks
worked, but most arrows described how the pipeline *derives* identities, not
what the database *stores*. A model that encodes derivation paths buys noise:
every derived join can disagree for legitimate reasons and the tool cannot
tell the domain's intent from the extraction's convenience.

## Retro-audit of the three committed pilot models (calibration)

Mappings read from `data/corpus.sqlite` (`documents`, `versions`, `captures`,
`pages`); "stored" means the value exists in a column that a client writes,
"derived" means it is a join/computed identity.

| Model | Arrow | Storing column | Verdict |
|---|---|---|---|
| model1 FTS anchor | `document_id` (pages→documents) | pages stores `version_id`; `document_id` is derived through `versions` | derived |
| model1 | `edition_date` (documents→edition_date) | `documents.edition_date` | stored |
| model1 | `hit_edition_date` (pages→edition_date) | derived via versions→documents | derived |
| model2 identity domains | `aviso_document_id` | `captures.requested_aviso_id` | stored |
| model2 | `document_id` | `documents.document_id` (backfill mirror) | stored |
| model2 | `public_version` | `versions.public_version` | stored |
| model3 capture selector | `capture_source` | source is derived from `requested_aviso_id` | derived |
| model3 | `capture_edition_date` | `captures.requested_edition_date` | stored |
| model3 | `selector_source` | selector is a query input, not a row | derived |
| model3 | `selector_edition_date` | selector is a query input, not a row | derived |
| model3 | `match` | resolution is a join result | derived |

Fail ratios: model1 **2/3** (67%), model2 **0/3** (0%), model3 **4/5**
(80%). The rule would have flagged model1 and model3 as plan-modeling and
cleared model2 — consistent with the pilot triage, and with model3 being a
deliberate selector (query-plan) model.

Limits, stated: the audit reads the DDL and the model headers by hand; no
SQL was executed to prove each mapping, and the pilot models were designed
for identity domains, not storage granularity. This is calibration, not
validation.

## The second-domain re-model protocol (executed — result in the last section)

1. Pick one bogart subdomain with a small, stable DDL surface (the FTS
   anchor: `pages`, `versions`, `documents`, `edition_date`).
2. Re-model at the rule's granularity: one arrow per stored reference
   (`pages.version_id`, `versions.document_id`, `documents.edition_date`);
   drop derived joins from the arrow set; declare equations only between
   stored paths if the invariant survives the drop.
3. Extract a real row slice (the grounded-instance harness pattern), run
   `catherine check`, and classify every finding as real vs noise by the
   pilot's written triage rule.
4. Compare signal density with the pilot's 2/69. Record the result in the
   `ct-modelable-domains` dossier; if it lands, record a probe entry under
   `ledger/probes/` with a committed baseline arm (per
   `add-domain-probe-validation`) — a doodle is not evidence.
5. Falsifier: if the re-model produces no better real/noise ratio than the
   pilot, the rule is refuted for this domain and the docs label stays
   PROPOSED with the negative result attached.

This protocol is filed as its own follow-up ticket (`catherine-8v9`), since
it is bogart-side work and the dossier lives in pinky.

## Second-domain re-model — EXECUTED 2026-11-13 (catherine-8v9)

Protocol step 2 executed literally, against bogart's serving projection
(`serving.sqlite`, sha256 `30ec1947…413c34`) — the real stored shape, not the
legacy DDL quoted above. Full pre-registration, provenance and triage:
`fixtures/probes/granularity-bogart/README.md`; harness:
`prototype/bogart_storage_granularity.py`; slice fixture:
`fixtures/probes/granularity-bogart/fts-anchor-slice.ct.json`.

| Run | Instance | Findings | Real | Noise | Exit |
|---|---|---|---|---|---|
| slice | 3 docs / 289 pages / 1 edition | 0 | 0 | 0 | 0 |
| full | 924 docs / 74,257 pages / 210 editions | 0 | 0 | 0 | 0 |

Arrows admitted by the rule: `document` (pages→documents, storing columns
`pages.document_id` + `pages.version`, a composite FK) and `edition_date`
(documents→edition, storing column `documents.edition_date`). Equations: none
— both sides of the pilot's equation collapse onto the same stored column, and
serving stores no per-hit edition anchor.

**Outcome: not validated; falsifier not met; label stays PROPOSED.** The ratio
is undefined (0 findings), not better than 2/69. Both pilot classes vanish
structurally: the 67 `planned`-partition noise findings *and* the 2 real
`bogart-4wt` ambiguity defects. The live selector census (1,441 selectors;
`>=2-match` 3 — the pilot's `nacional/primera 2026-11-03` and
`pba/JUDICIAL 2026-11-03` plus a new `nacional/segunda 2026-11-13`; 0-match 67)
shows the real defects persist while the storage model reports nothing: no
table stores a selector→capture column.

**What this changes.** The rule as written is rejected for global use and
re-scoped: it governs *structural* claims (references, totality) where a
storing column exists; derivation/selection claims must be declared as such
and checked by the rule-lens / policy machinery (P8), which is exactly where
the pilot's only real defects came from. The pre-build one-third test remains
a smell test, not a gate — passing it (0/2 here) says nothing about whether the
model can see intent-level defects. No probe entry recorded: a probe needs an
executed baseline arm and a landed outcome, and the outcome is a negative
result, which lives in this file and the fixture README per the falsifier
branch of step 5.