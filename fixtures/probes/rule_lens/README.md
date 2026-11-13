# Probe 2 — rule lens on an EXTRACTED table (pre-registration + provenance)

Committed for `add-domain-probe-validation` tasks 8-9 (beads
`catherine-1od`). The expectation classes were pre-registered in the
probe catalog (`crates/catherine-ledger/src/catalog.rs`, committed with
tasks 3-4); this directory holds the substrate, the fault set, and the
frozen baseline protocol. Observed counts are marked as observations,
never presented as predictions.

## Provenance (real source, not authored DMN)

The substrate is a real rule table extracted from an official Boletín
Oficial annex — the DTRT indemnity caps for CCT 288/97:

| Field | Value |
|---|---|
| Annex | Disposición 75/2026, annex 1 (`DI-2026-75-APN-DTRT#MCH`) |
| Source notice | `recon/sources/probe-anexos/html/348735-20261008.html` (bogart repo, edition 2026-10-08) |
| PDF text layer | `recon/sources/probe-anexos/pdfs/anexo_7796103_1.pdf` (gitignored in bogart) |
| Committed extraction | `recon/sources/probe-anexos/pdfs/anexo_7796103_1.txt`, sha256 `7f01bdcb903f9f02c94ffde396a001a7114d9774323b8c77e86af2bc15d1c645` |

The extracted table has two dimensions — `alcance` ∈ {`general`,
`zona_desfavorable`} and one effective date from three — and six rows,
each a printed `Promedio de las Remuneraciones` → `Tope indemnizatorio`
pair. `Tope = 3 × Promedio` holds on every row; `Zona Desfavorable =
1.2 × General` holds per date (rounded).

**Extraction hazard (recorded, not smoothed):** the PDF text layer emits
`Alcance: Zona Desfavorable` *after* its three rows, so a naive
row-order parser assigns all six rows to `General`. `anexo_7796103_1.txt:62`
is that label. The extraction here maps the label backwards from the
block boundary; the hazard is exactly why a DMN analyzer cannot ingest
this source, and it motivates one of the injected faults (a duplicate
row, the naive-extraction artifact).

**Scope caveat:** the annex is not in the bogart corpus read surface
(`corpus_attachments` is empty; the exact values appear on 0 serving
pages). Its grounded artifact today is a recon text extraction, so the
probe is *extracted-table*, not *corpus-document*, evidence. Recorded in
the ledger entry's discrepancies.

## Substrate files

| Fixture | Role |
|---|---|
| `extracted/cct-288-97.ct.json` | the extracted table (6 rules, one per (alcance, fecha) cell, action = printed cap) |
| `extracted/faults/01-gap-remove-general-2023-01.ct.json` | a row dropped at a SAMPLED cell |
| `extracted/faults/02-gap-remove-zona-2022-12.ct.json` | a row dropped at an UNSAMPLED cell |
| `extracted/faults/03-overlap-declared-disjoint.ct.json` | one ZD rule widened to both scopes + `DeclaredDisjoint` |
| `extracted/faults/04-shadow-duplicate-general-2022-10.ct.json` | naive-extraction duplicate (dead under `PriorityOrder`) |
| `extracted/faults/05-shadow-widen-zona-2022-10.ct.json` | ZD rule widened to every date; two later ZD rules become dead |
| `floor/cct-1492-15.ct.json` | near-floor control: one-row extracted table (Disposición 45/2026, CCT 1492/15, text sha256 `ae5bea61…49fb`) |
| `floor/faults/01-gap-remove-row.ct.json` | near-floor fault: the only row dropped |
| `floor/faults/02-overlap-different-cap.ct.json` | near-floor fault: a second rule with a different cap + `DeclaredDisjoint` |

## Frozen baseline protocol (committed BEFORE the run)

`prototype/rule_lens_extracted_baseline.py` — the review a team actually
performs on an extracted table: a **sampled** value check, four cells for
the above-floor table (`General` at all three dates, `Zona Desfavorable`
at 2022-10-01) and the single cell for the near-floor table, each
resolved under the document's declared policy and compared to the
printed cap. Missing, ambiguous, and wrong-cap resolutions are its
detections. Sampling is the point: a spot check does not enumerate the
rule space.

Pre-registered baseline behaviour: catches `01` (sampled gap), `03`
(overlap at a sampled cell), and both floor faults; misses `02`
(unsampled gap), `04` (dead duplicate), `05` (dead successors of the
widened rule).

## Observed results (recorded after the executed run)

| Fault | Class | Baseline | Lens findings | Unique |
|---|---|---|---|---|
| 01-gap-remove-general-2023-01 | gap | caught | 1 `underconstraint` | no |
| 02-gap-remove-zona-2022-12 | gap | missed | 1 `underconstraint` | **yes** |
| 03-overlap-declared-disjoint | overlap | caught | 1 `violation` | no |
| 04-shadow-duplicate-general-2022-10 | shadow | missed | 1 `redundant` | **yes** |
| 05-shadow-widen-zona-2022-10 | shadow | missed | 2 `redundant` | **yes** |
| floor-gap-remove-row | floor-near-trivial | caught | 1 `underconstraint` | no |
| floor-overlap-different-cap | floor-near-trivial | caught | 1 `violation` | no |

Unique detections: **4** above-floor (1 gap + 3 shadow), **0** at the
near-floor control — the control self-negates (verdict `killed`), so the
P8 unique band survives restriction to an extracted table, scoped to
the classes actually instantiated.

## Unreached expectation (recorded as a gap, never forced)

`derivable` — a derivable rule's region must equal the intersection of
two OTHER rules' regions; the extracted table's rules are disjoint
singleton cells, so every pairwise intersection is empty and no
non-duplicating derivable insertion exists. The probe records the gap
instead of fabricating broad axis rules the annex does not print.

## Execution

```
python3 prototype/rule_lens_extracted_baseline.py --assert
cargo run -q -p catherine-ledger --bin ledger-check -- ledger/probes/rule-lens-extracted
```