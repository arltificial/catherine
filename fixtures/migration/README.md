# Δ migration RED fixtures (add-delta-migration tasks 1–3)

Each fixture is a `format_version` 1.1 document whose `schema_map` is
TOTAL (the parser's `partial_schema_map` gate accepts it as-is). The
migration ENGINE is phase B — these fixtures pin the documents the
engine must exhibit lost data over, with the expected outcome recorded
per fixture (never inferred after the fact):

| Fixture | Map shape | Expected phase-B outcome (kata-10 pattern) |
|---|---|---|
| `drop_object.ct.json` | `Scratch` → null (object dropped) | one lost-data finding per dropped `Scratch` row (3 rows → 3 findings), each naming the dropped row as witness |
| `field_narrowing.ct.json` | `annotation` → null (arrow dropped) | one lost-data finding per `annotation` cell that held a value (2 cells → 2 findings) |
| `non_invertible_round_trip.ct.json` | Δ: Wide→Narrow with two Wide rows collapsing onto one Narrow row | the round-trip Δ∘Δ′ analysis proves non-invertibility with the offending element: two Wide rows (`_0`, `_1`) map to one Narrow row — the second is not recoverable |

The round-trip counterexample companion (the candidate inverse map Δ′
whose composition provably fails) ships with the phase-B engine work;
this fixture is the forward half it must fail to invert.
