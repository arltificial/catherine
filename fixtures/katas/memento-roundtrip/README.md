# Kata — memento round trip as a Δ schema-map pair (pre-registration)

Ticket `catherine-lgl.5`. Committed **before** any run. Nothing here is
edited after the run except the observed-results section, which is
labeled observed; a correction is a superseding ledger entry.

## Hypothesis

A legacy Memento implementation (save a snapshot, restore a document) is
a pair of Δ schema maps: `save: Document -> Snapshot` and
`restore: Snapshot -> Document`. The Memento contract is
`restore ∘ save = id` on the document. Catherine's Δ engine expresses
the two failure classes the pattern's law names:

- an arrow or object assigned `null` in either map is **lost data**,
  enumerated per lost row or per valued dropped cell (`catherine
  migrate`, label `executed`), not a silent drop;
- a map pair that is not the identity is **non-invertibility**, proven
  with the offending element named (`catherine_core::migrate::round_trip`,
  asserted by the frozen test; the CLI's `migrate` surface is used for
  the lost-data arm).

Prediction: the conventional arm (a field-by-field save/restore
checklist over sampled cells) catches only the losses its sample lands
on; the Δ arms name every loss and the collapse's unrecoverable element.
A parity outcome is a legitimate, recorded result.

## Pattern-equivalence mapping

| Memento element | Catherine surface |
|---|---|
| `Save()` (create memento) | `schema_map` in the save direction: every document object/arrow carries an explicit target or `null` |
| `Restore()` (set state from memento) | `schema_map` in the restore direction: `Snapshot -> Document` |
| memento is a snapshot of the originator | source instance rows/values are the saved cells; no field is invented |
| undocumented loss in `Save`/`Restore` | `lost_row` / `lost_cell` findings, one witness per element |
| `Restore(Save(doc)) == doc` | `round_trip(source, forward, inverse)` returns `None`; otherwise the smallest `(object,row)` that does not return to itself is the proof |

Lowering limits, stated: catherine has no `Document`/`Snapshot` *type*
constructor and no runtime object identity; a Memento here is a schema
pair plus instances. The kata therefore validates the Δ law, not the
OOP encapsulation. Arrow values are row references (the model format's
value shape), so a "field" is a self-arrow and "field dropped" is that
arrow assigned `null`.

## Fault matrix (pinned in `faults.json`)

All fixtures are total format-1.1 documents. v1 snapshot shape:
`Snapshot`, `SnapshotPage`, `SnapshotTag` with `snap_title`,
`snap_subtitle`, `snap_word_count`, `page_text`, `page_note`,
`page_next`, `page_owner`, `tag_label`, `tag_owner`.

| Fault | Fixture | Injection | Expected probe elements |
|---|---|---|---|
| near-floor control | `restore-identity.ct.json` | none — full-fidelity restore map, every cell carried | none |
| (a) dropped field on restore | `restore-dropped-field.ct.json` | restore map assigns `snap_word_count` `null`; the field is required and valued on both snapshots | `lost_cell:snap_word_count#0`, `#1` |
| (b) lossy serialization of an optional field | `restore-lossy-optional.ct.json` | restore map assigns `page_note` `null`; the field is optional, valued on pages 1 and 3 only | `lost_cell:page_note#1`, `#3` |
| (c) version-skew restore | `restore-version-skew.ct.json` | old snapshot shape (per-page rows) restored into the new document shape (no block object): `SnapshotPage` collapses onto `Document`; `page_next`, `page_note`, `page_text` have no place in the new shape and are `null` | `lost_cell:page_next#0`, `lost_cell:page_note#1`, `lost_cell:page_text#0..#2`, `round_trip:SnapshotPage#0` |

The control is the near-floor case: a faithful identity snapshot. The
kernel must emit zero findings (no false positive) and
`round_trip(control, forward, reverse)` must be `None`; any unique
detection kills the control.

## Conventional baseline arm (committed before the run)

`prototype/memento_roundtrip_baseline.py` is the review a team performs
on a legacy save/restore: a field-by-field migration checklist over a
**sampled** set of cells. For each sampled `(arrow, row)` it asserts the
restore map does not assign the field to `null`. It executes no Δ, has
no row-count or object-collapse model, and cannot express
`restore ∘ save = id`.

Pre-registered sample (`faults.json`):

| Arrow | Rows |
|---|---|
| `snap_title` | 0, 1 |
| `page_text` | 0, 1 |
| `tag_label` | 0, 1 |
| `snap_subtitle` | 0 |
| `page_note` | 0 |
| `page_owner` | 0 |

Sampling is the point: the checklist is not an enumeration of the
migration. Pre-registered baseline behaviour: no detections on the
control, on (a), and on (b); on (c) it catches only the sampled
`page_text` cells `#0` and `#1`, because those are the only sampled
valued cells in a dropped field. It never names the collapse or the
lost optional cells.

## Kill condition

If the conventional checklist catches the same losses as the probe, the
entry is recorded `killed` and said so, not reframed. The control is
`killed` by construction when it self-negates (zero unique). A null or
parity result is the deliverable.

## Comparison rule (fixed before the run)

An element is `lost_row:<object>#<row>`, `lost_cell:<arrow>#<row>`, or
`round_trip:<object>#<row>`. An arm identifies an element only by naming
it. `unique_detections` = probe elements minus baseline-identified
elements, derived arithmetically by the baseline script and re-derived
in the frozen test. Verdicts follow `harness::derive_probe_verdict`:
empty unique = `killed`; unique plus an executed baseline arm =
`survived`.

## Falsifiers

- Any probe finding outside the pre-registered sets — record the
  discrepancy; never edit `faults.json` after the run.
- Baseline `--assert` mismatch — the baseline or a fixture drifted;
  re-pin only with a superseding entry.
- A control finding, or `round_trip` returning a proof on the control —
  the near-floor control is refuted and the kata is `killed`.
- The version-skew `round_trip` naming a different element or returning
  `None` — the non-invertibility claim is refuted; record the null
  result.

## Execution

```
python3 prototype/memento_roundtrip_baseline.py --assert        # baseline arm, frozen
cargo run -q -p catherine-cli -- migrate --json fixtures/katas/memento-roundtrip/<fixture>
cargo test -q -p catherine-ledger --test memento_roundtrip_kata
cargo run -q -p catherine-ledger --bin ledger-check -- ledger/katas/memento-roundtrip
```

## Observed results (recorded after the executed run)

Everything below the line is observed, never predicted.

Baseline arm, executed first: `python3
prototype/memento_roundtrip_baseline.py --assert` exited 0 with
`baseline_sha256 =
f67f8df3786076fdf850cc0aa8c4e237d9f75c7376cfcb5da45c3dc44f487c8f`,
unique `[0, 2, 2, 4]`.

| Fault | Baseline detections | Probe findings | Unique | Verdict |
|---|---|---|---|---|
| identity (control) | none | none (`lost: []`, round trip `None`) | 0 | `killed` (self-negation) |
| dropped-field | none | 2 `lost_cell` (`snap_word_count` rows 0, 1) | 2 | `survived` |
| lossy-optional | none | 2 `lost_cell` (`page_note` rows 1, 3) | 2 | `survived` |
| version-skew | `field-drop:page_text#0`, `field-drop:page_text#1` | 5 `lost_cell` (`page_next#0`, `page_note#1`, `page_text#0..#2`) plus the round-trip proof `round_trip:SnapshotPage#0` | 4 | `survived` |

The version-skew proof, observed: `row 0 of SnapshotPage` shares its
forward image `Document row 0` with row 1; the candidate inverse
returns `Snapshot row 0`, so the page element is not recoverable. The
frozen test asserts the witness, not a copy of the string.

Ledger: `ledger/katas/memento-roundtrip/` holds four entries — control
first (`killed`, empty unique), then the three positives with executed
baseline arms; `ledger-check` reports `ok: 4 entries (4 kata), rollup
verdict: (no rollup)`.

Outcome honestly stated: the probe buys unique detections on all three
faults (2 / 2 / 4) and the near-floor control self-negates. The
version-skew unique set includes one sampled loss the checklist missed
(`page_text#2`), one unsampled field (`page_next#0`), one sampled field
at an unsampled valued row (`page_note#1`), and the non-invertibility
witness no conventional diff expresses. Parity was possible and did
not occur here; the control did.