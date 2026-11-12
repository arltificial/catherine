# Δ migration fixtures (add-delta-migration tasks 1–3 RED + task 8 rehearsal)

Each fixture is a `format_version` 1.1 document whose `schema_map` is
TOTAL (the parser's `partial_schema_map` gate accepts it as-is). The
migration ENGINE is phase B — these fixtures pin the documents the
engine must exhibit lost data over, with the expected outcome recorded
per fixture (never inferred after the fact):

| Fixture | Map shape | Expected phase-B outcome (kata-10 pattern) |
|---|---|---|
| `drop_object.ct.json` | `Scratch` → null (object dropped) + `scratch_note` → null (its arrow dropped) | every dropped element named loudly: one lost-row finding per dropped `Scratch` row (3) AND one lost-cell finding per valued `scratch_note` cell (3) — 6 findings total |
| `field_narrowing.ct.json` | `annotation` → null (arrow dropped) | one lost-data finding per `annotation` cell that held a value (2 cells → 2 findings) |
| `non_invertible_round_trip.ct.json` | Δ: Wide→Narrow with two Wide rows collapsing onto one Narrow row | the round-trip Δ∘Δ′ analysis proves non-invertibility with the offending element: two Wide rows (`_0`, `_1`) map to one Narrow row — the second is not recoverable |
| `bogart_rehearsal.ct.json` (task 8, P9 dry run) | raw corpus → serving projection: `RawDocument`/`RawPage` transfer to `ServingDocument`/`ServingPage` with `page_of` → `["serving_page_of"]`; deliberate drops: `RawCapture` → null (captures are backfill inputs, not served) + `prev_page` → null + `captured_page` → null | 10 findings total, in the engine's deterministic (sorted) order: one lost-row per dropped `RawCapture` row (4: rows 0–3), then one lost-cell per valued dropped-arrow cell — `captured_page` at `RawCapture` rows 0–3 (4), `prev_page` at `RawPage` rows 1–2 (2, `_0` held no value). Target rows: `ServingDocument` 2, `ServingPage` 3 — migrated via `catherine migrate`, exit 8 |

The round-trip counterexample companion (the candidate inverse map Δ′
whose composition provably fails) ships with the phase-B engine work;
this fixture is the forward half it must fail to invert.

`catherine migrate <doc>` (task 7) runs these through the phase-B
engine: `--json` emits the success envelope with `data.rows` (target
rows per object), `data.lost` (the `MigrationFinding` encoding
verbatim), and `data.row_map`; lost data exits 8, a lossless migration
exits 0, and a document without a `schema_map` is the labeled
`no_schema_map` error (exit 1).
