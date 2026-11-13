# Granularity rule — PROPOSED, pilot calibration, and the re-model protocol

Date: 2026-11-13 · Ticket: `catherine-kq1` · Status: rule PROPOSED, one
domain of evidence (bogart), second-domain test **not executed**.

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

## The second-domain re-model protocol (not executed here)

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

This protocol is filed as its own follow-up ticket (`catherine-...`), since
it is bogart-side work and the dossier lives in pinky.