# Kata — Builder as monoid (legacy accumulation, `catherine-lgl.2`)

Committed **before** the probe run. Nothing in this directory or in
`prototype/builder_monoid_baseline.py` is edited after the run: a correction is
a superseding ledger entry.

## Hypothesis (pre-registered)

A legacy Builder's `addStep`/`build` accumulation is a monoid fold: a product
is the fold of an ordered step chain, the empty chain is the identity, and the
built artifact's parts are exactly the parts emitted by the declared steps. If
that fold is modeled as a schema — `Product`, `Step`, `Part`, with the
emission and provenance arrows and the trace equation below — `catherine`
should make the accumulation's losses and cross-product splices loud, and it
should say so plainly where it cannot (step multiplicity, total order, and the
monoid identity/default law).

The pattern-equivalence mapping is the ploeh `Builder ≅ monoid` correspondence
(`docs/ref/ploeh-grounding.md` §2.1/§6 candidate 2 analog): `Step` rows are the
monoid's elements, `step_prev` is the ordered composition (each chain's root
step points at itself — the identity as an endomorphism), and a built
`Product` is the fold's result. **The schema does not enforce the monoid laws
themselves**; it enforces the accumulation's structural consequences, and the
laws that remain inexpressible are recorded as gaps, not papered over.

## Schema under test

Objects: `Product`, `Step`, `Part`.

| Arrow | Source → target | Meaning |
|---|---|---|
| `step_product` | `Step → Product` | the product whose fold contains the step |
| `step_prev` | `Step → Step` | the predecessor in the product-local chain; a root step points at itself |
| `emits` | `Step → Part` | the part the step accumulates |
| `part_product` | `Part → Product` | the built product the part belongs to |
| `part_step` | `Part → Step` | the declared accumulation step the part traces to |

Accepted equations:

1. `part_step ; step_product = part_product` — **every emitted product part
   traces to a declared accumulation step of the same product** (the trace law
   named by the assignment).
2. `step_prev ; step_product = step_product` — the chain stays inside one
   product (a cross-product splice is a violation, not an invisible splice).
3. `emits ; part_product = step_product` — the step's emitted part belongs to
   the step's product.

`fixtures/katas/builder-monoid/builder.ct.json` is the clean above-floor
fixture: two builders (`p1` with three steps, `p2` with two), every arrow
valued, zero findings.

## Fault matrix (pre-registered)

The above-floor fault set is `faults/*.ct.json`; the near-floor control is
`floor/`. Each fault is pre-registered in `faults.json` with its sampled trace,
baseline expectation, probe expectation, canonical element map, unique count,
and verdict.

| Fault | Injection | Surface | Pre-registered probe finding | Canonical element |
|---|---|---|---|---|
| `floor-dropped-step` (control) | the single step row is removed; the part survives with no `part_step` | `check` | `dangling:part_step@row0` | `dropped_step:Step#0` |
| `floor-duplicated-step` (control) | a second identical step is appended to the one-step builder | `check` | none — instance multiplicity is not a path equation | — |
| `dropped-step` | the migration map drops the `Step` object (the declared accumulation), all 5 rows and every valued cell of the `Step`-sourced/`part_step` arrows | `migrate` | 5 `lost_row` + 20 `lost_cell` | `dropped_step:Step#0..4`, `dropped_step_cell:<arrow>#0..4` |
| `duplicated-step` | the duplicated step row `s1_dup` is inserted at index 3 **and** the trace law is declared twice | `check`, `redundant` | `check`: none; `redundant`: the duplicated trace law (one witness, emitted twice) | `duplicate_step:Step#3` |
| `out-of-order-step` | `p2`'s root step `s2_0` is spliced into `p1`'s chain as `s1_2`'s predecessor (`step_prev`) | `check` | one `violation` on the chain equation (index 2) at `Step` row 2: lhs `p2`, rhs `p1` | `misordered_step:Step#2` |
| `optional-part-dropped` | the migration map drops `optional_part` (`Product → Part`); the optional part row survives but its cell is lost | `migrate` | one `lost_cell` on `optional_part` at `Product` row 0 | `dropped_optional_part:Part#3` |

Element vocabulary: a probe element is one named finding witness — `lost_row:<object>#<row>`,
`lost_cell:<arrow>#<row>`, `violation:eq<index>@row<row>`, `dangling:<arrow>@row<row>`,
or `redundant:<derivable path>`. Each probe witness is mapped to a canonical
fault element by the pre-registered `probe_element_map`; an unmapped finding
fails the baseline script loudly.

## Conventional baseline arm (frozen before the run)

`prototype/builder_monoid_baseline.py` models the review a team actually runs
on a legacy builder: a **scripted accumulation diff over sampled steps**. It
replays the recorded trace, and for each sampled step index asserts that the
step row exists and that its `emits` cell is the recorded part. It never
inspects the `schema_map` (a migration edit is not a runtime accumulation
event) and never runs the redundancy analysis (a conventional test does not
prove schema declarations redundant).

Frozen sampling and detections:

| Fault | Sample | Baseline detection | Elements identified |
|---|---|---|---|
| `floor-dropped-step` | `[0]` | `sample:step#0:missing` | `dropped_step:Step#0` |
| `floor-duplicated-step` | `[0,1]` | `sample:step#1:unexpected=s_dup` | `duplicate_step:Step#1` |
| `dropped-step` | `[0,1,2,3,4]` | none (source trace unchanged) | none |
| `duplicated-step` | `[0,1,2,3]` | `sample:step#3:unexpected=s1_dup` | `duplicate_step:Step#3` |
| `out-of-order-step` | `[0,1,2,3,4]` | none (step rows and emissions unchanged) | none |
| `optional-part-dropped` | `[0,1,2,3,4]` | none (required accumulation unchanged; the optional part is not asserted) | none |

The baseline is executed before the probe arm by the script itself:
`--assert` recomputes the detections and identified elements from the committed
fixtures, compares them with `baseline_expected`/`baseline_element_map`, and
only then runs the `catherine` surfaces. `--hash` prints the sha256 pinned in
the ledger entries.

## Pre-registered unique arithmetic

`unique_detections` = probe canonical elements minus baseline-identified
elements (arithmetic, not judgement). Pre-registered counts:

| Fault | Probe elements | Baseline identifies | Unique | Verdict |
|---|---|---|---|---|
| `floor-dropped-step` | 1 | 1 | **0** | `killed` |
| `floor-duplicated-step` | 0 | 1 | **0** | `killed` |
| `dropped-step` | 25 | 0 | **25** | `survived` |
| `duplicated-step` | 1 | 1 | **0** | `killed` |
| `out-of-order-step` | 1 | 0 | **1** | `survived` |
| `optional-part-dropped` | 1 | 0 | **1** | `survived` |

The near-floor control self-negates: one step is the whole accumulation, so the
sampled diff catches both the dropped and the duplicated step, and the probe's
unique set is empty (`killed`). The `duplicated-step` above-floor entry is also
pre-registered `killed`: the baseline catches the duplicated **row** when it
reaches the sampled index, while the probe's `redundant` finding names the
duplicated **law**; both arms identify the same fault, so the unique set is
empty. This is parity, not a detection-rate claim: the kernel does not detect
the instance-level duplicate by itself.

## Kill condition (pre-registered)

If the conventional sampled-accumulation diff catches the same element the
probe names, the entry records `unique_detections: []` and verdict `killed`;
no detection-rate win is ever claimed. If the sampled trace cannot express the
fault but the probe also emits nothing, the entry is `killed` too. If the
pattern's law cannot be expressed with the existing kernel surfaces, the
entry records `inconclusive` with the named gap rather than fabricating a
surface.

## Observed results (recorded after the executed run)

Executed 2026-11-13; the pre-registered counts held exactly. Observed, never
presented as predictions:

| Fault | Baseline | Probe findings | Unique | Verdict |
|---|---|---|---|---|
| `floor-dropped-step` | `sample:step#0:missing` | 1 `dangling` (`part_step@row0`) | **0** | `killed` |
| `floor-duplicated-step` | `sample:step#1:unexpected=s_dup` | 0 (`check`) | **0** | `killed` |
| `dropped-step` | none | 5 `lost_row` + 20 `lost_cell` | **25** | `survived` |
| `duplicated-step` | `sample:step#3:unexpected=s1_dup` | 0 (`check`); 2 identical `redundant` findings in `redundant` (one distinct witness) | **0** | `killed` |
| `out-of-order-step` | none | 1 `violation` (equation 2, `Step` row 2: lhs `p2`, rhs `p1`) | **1** | `survived` |
| `optional-part-dropped` | none | 1 `lost_cell` (`optional_part@row0`) | **1** | `survived` |

`builder.ct.json` and `floor/single-step-builder.ct.json` check clean (exit 0,
zero findings); `redundant` on the clean base is empty. The baseline script
reports 27 unique detections across the six faults and pins itself at
`sha256:9d7b1066b77e9feedef981a4e1711a1af691ad99958cedeea5c2ec0408ef9ff4`
(`--hash` recomputes). The near-floor control self-negates (`killed`); the
duplicated-step entry is parity (`killed`), because the scripted diff catches
the duplicated row the probe's redundancy proof cannot name. The two survivings
above-floor entries are the migration axis's loud per-element losses and one
cross-product splice — never a detection-rate claim against a competent suite.

### Recorded expressibility gaps (observed, not forced)

- **Instance multiplicity.** Path equations cannot state that a step appears
  exactly once in the accumulation; `check` is silent on the duplicated row and
  `redundant` only proves the duplicated *declaration* derivable. The fault is
  recorded as parity, and this gap is the reason.
- **Total order / monoid identity.** `step_prev` can only require that a
  chain stays product-local (the equation that caught the splice). It cannot
  impose a total order on the chain, and the empty-chain identity/default law
  (empty build = documented defaults) has no path-equation form over an
  instance. The one-step near-floor fixture is the control for the identity
  boundary, not a proof of the law.
- **No detection-rate claim.** The migration losses and the cross-product
  splice are unique to the modeled surfaces, exactly the E4 migration-axis
  license; the kata does not claim the kernel out-detects the conventional
  suite.

## Execution

```
python3 prototype/builder_monoid_baseline.py --assert
cargo run -q -p catherine-ledger --bin ledger-check -- ledger/katas/builder-monoid
cargo test -q -p catherine-ledger --test builder_monoid_kata
```