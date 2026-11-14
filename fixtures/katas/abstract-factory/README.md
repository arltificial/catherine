# Kata — abstract-factory (pre-registration)

Committed **before** the recorded run (`catherine-r5k.1`). Nothing in the
pre-registration sections is edited after the run: a correction is a
superseding ledger entry.

Substrate: the GoF Abstract Factory pattern as a declared product-family
model. The disposition row (`docs/ref/pattern-kata-dispositions.md`) marks
the pattern **disposed — construction only; no invariant on a finite
instance**. This kata tests the one candidate law that row leaves open.

## Named law (written before any fixture)

**Law AF-T — declared-grid product-family totality.** A model that declares

- a product-kind vocabulary `Kind` (the abstract factory's creation-method
  set),
- concrete factories `Factory` with `factory_family : Factory → Family`,
- concrete products `Product` with `product_kind : Product → Kind` and
  `product_family : Product → Family`,
- a realization site `Cell` for every (concrete factory, product kind)
  pair the model claims to realize, with arrows
  `cell_factory : Cell → Factory`, `cell_kind : Cell → Kind`,
  `cell_product : Cell → Product`,
- a coverage witness `kind_witness : Kind → Cell`,

is *product-family total* iff every **declared** cell of that grid is
filled and agrees:

| | Law | Encoding |
|---|---|---|
| T1 | every declared realization site names a product | no `dangling` on `cell_product` |
| T2 | every declared kind is realized somewhere | no `dangling` on `kind_witness` |
| T3 | the product realizing a kind is of that kind | equation `cell_product ; product_kind = cell_kind` (`accepted`) |
| T4 | a factory's products belong to the factory's family | equation `cell_product ; product_family = cell_factory ; factory_family` (`accepted`) |
| T5 | the realization table is a function, not a bag | key `Cell[cell_factory, cell_kind]` (`accepted`) |

This is a **declared-cell totality check, not a functor law**. The
universal quantifier lives in the authored `Cell`/`Kind` rows; the kernel
evaluates every declared cell and reports an unfilled one as an executed
`dangling` finding. The law is deliberately *scoped*: T1/T2 are total over
the declared instance, never over `Factory × Kind` — see the named gap G1.

## Pattern-equivalence mapping (fixed before the run)

| Abstract Factory element | `.ct.json` encoding |
|---|---|
| abstract factory interface (creation-method set) | object `Kind` (dense rows = declared product kinds) |
| concrete factory | object `Factory` (dense rows) |
| product family / variant | object `Family`; arrows `factory_family`, `product_family` |
| concrete product | object `Product`; `product_kind` names its kind |
| "factory F realizes kind K with product P" | object `Cell` with `cell_factory`, `cell_kind`, `cell_product` |
| "factory F has no creation method for K" | a declared `Cell` with `cell_product = null` → executed `dangling` |
| "kind K has no realization anywhere" | `kind_witness[K] = null` → executed `dangling` |
| "the product realizing K is of kind K" | equation T3 |
| "a factory's products share the factory's family" | equation T4 |
| "one realization per (factory, kind)" | key T5 |
| functor composition (`F(g∘f) = F(g)∘F(f)`) | **not encoded** — named gap G2 |

The kernel's finding surface is the path-equation + totality surface: a
null cell is `dangling` (exit 4), a failed accepted equation or key is
`violation` (exit 3), and a clean model is exit 0. No new predicate is
introduced by this kata.

## Expressibility boundary (fixed before the run)

- **G1 — cross-product completeness.** A (factory, kind) pair with **no**
  `Cell` row is invisible: nothing dangles because no cell exists to
  dangle, and the kernel has no quantifier over `Factory × Kind`. The
  coverage direction that *is* expressible (T2) is per-kind existential
  ("some factory realizes K"), authored as `kind_witness`; the universal
  per-factory direction is not. `05-missing-combination.ct.json` is the
  pre-registered null result for this boundary.
- **G2 — functor composition.** The model carries no product-operation
  arrows, so "F maps composites to composites" has no truth value on this
  surface. This is the dictionary's abstraction, not this kata's law.
- **G3 — witness-kind agreement.** `kind_witness ; cell_kind = id_Kind`
  cannot be written: the kernel rejects an empty equation path, so there
  is no identity path form. A `kind_witness` pointing at a cell of another
  kind is therefore not caught. The fixtures keep every witness agreeing;
  the gap is named, not tested.
- **G4 — orphan product.** A declared `Product` no cell references has no
  coverage surface unless a `product_witness : Product → Cell` arrow is
  authored. `01-missing-cell.ct.json` and `02-wrong-kind.ct.json` contain
  such orphans (the product class exists in the codebase, the factory no
  longer returns it); the kernel is silent on them, as expected.

## Fault matrix (pre-registered)

Element vocabulary: one element is one declared realization site, spelled
`cell:<Factory>/<Kind>`, or one declared kind, spelled `kind:<Kind>` for
the T2 surface. The baseline identifies an element only when one of its
pre-registered sample assertions fails on that exact element.

| Entry | Fixture | Injected fault | Predicted kernel surface | Baseline sample |
|---|---|---|---|---|
| `near-floor` (control) | `near-floor.ct.json` + two variants | 2×2 grid; (a) `ModernFactory/chair` product null, (b) `VictorianFactory/sofa` realizes a modern sofa | (a) `dangling cell_product`; (b) `violation` equation T4 | all four pairs + both kinds → catches both |
| `missing-realization` | `01-missing-cell.ct.json` | declared cell `ModernFactory/table` with `cell_product = null` | `dangling cell_product` @ that cell | sample omits `ModernFactory/table` → miss |
| `unknown-product` (construction) | `06-unknown-product.ct.json` | `cell_product` names undeclared `GhostChair` | parse rejection `syntax` naming the `Cell` row | not run; not counted |
| `wrong-kind` | `02-wrong-kind.ct.json` | `ModernFactory/sofa` realizes `ModernChair` | `violation` equation T3 @ that cell | sample omits `ModernFactory/sofa` → miss |
| `duplicate-declaration` | `03-duplicate-cell.ct.json` | two `Cell` rows for `ModernFactory/sofa` | `violation` key T5 @ that cell | sample omits `ModernFactory/sofa` → miss |
| `unrealized-kind` | `04-unrealized-kind.ct.json` | kind `bench` declared, no cell realizes it, `kind_witness[bench] = null` | `dangling kind_witness` @ `bench` | frozen kind list omits `bench` → miss |
| `missing-combination` | `05-missing-combination.ct.json` | no `Cell` row for `ModernFactory/table`; `table` is still witnessed by `VictorianFactory/table` | **none** — G1 | sample omits the pair; both arms silent |

### Fault fixtures

- `near-floor.ct.json` — clean 2×2 grid (2 kinds × 2 factories), every
  cell filled, T3/T4/T5 hold, zero findings.
- `near-floor-fault-missing.ct.json` — `cell_product[cell_modern_chair] = null`.
- `near-floor-fault-family.ct.json` — `cell_product[cell_victorian_sofa] = ModernSofa`.
- `01-missing-cell.ct.json` — 3 kinds × 2 factories, `cell_modern_table`
  product null, `ModernTable` present but unreferenced.
- `06-unknown-product.ct.json` — `01` with `cell_product[cell_modern_chair] = GhostChair`.
- `02-wrong-kind.ct.json` — `cell_product[cell_modern_sofa] = ModernChair`.
- `03-duplicate-cell.ct.json` — extra `cell_modern_sofa_alt`, same
  (factory, kind) and product as `cell_modern_sofa`.
- `04-unrealized-kind.ct.json` — `Kind` gains `bench`; `kind_witness[bench] = null`;
  no bench cell exists.
- `05-missing-combination.ct.json` — 5 cells: every (factory, kind) pair
  except `ModernFactory/table`; `ModernTable` is an orphan product;
  `kind_witness[table] = cell_victorian_table`.

## Near-floor control

The control is a two-kind, two-factory grid with four cells. The frozen
sample covers all four pairs and both kinds, so both injected faults are
caught by the baseline; the control's unique set must be **empty** and
its verdict `killed`. If the control survives, the kata is measuring test
coverage, not modeling.

## Conventional baseline protocol (frozen before the run)

`prototype/abstract_factory_baseline.py` asserts, for each run:

1. **Per sampled pair** `(Factory, Kind)`: exactly one declared `Cell`
   resolves the pair (`duplicate-cell` otherwise, the conventional table
   is a map); the cell's `cell_product` resolves (`product-missing`); the
   product is a declared row (`product-undeclared`); `product_kind`
   equals the pair's kind (`kind-mismatch`); `product_family` equals
   `factory_family` of the factory (`family-mismatch`).
2. **Per sampled kind**: at least one declared cell with that kind has a
   resolved product (`sample:kind:<Kind>:unrealized`).

Pairs and kinds outside the frozen sample are never inspected. The script
runs the baseline arm **before** the probe arm inside the same process,
executes the probe arm as the real CLI subprocess (`catherine check
--json`), maps witnesses to the elements above, and compares everything
to `faults.json`. `--assert` exits 1 on any drift; `--hash` prints the
script sha256 (pinned in the ledger's `comparison.baseline.artifact`).

The construction run is executed too: the script requires the CLI to
exit 1 with `code: syntax` and maps the rejected `Cell` row to its
element; it is recorded in `run.commands` and never counted in
`unique_detections`.

## Comparison rule

`unique = executed probe elements − baseline-identified elements`, with
these fixed rules:

- a `dangling` witness on `cell_product` (or `cell_factory`/`cell_kind`)
  resolves to the `cell:<Factory>/<Kind>` of its `Cell` row; on
  `kind_witness` it resolves to `kind:<Kind>`;
- a `violation` equation witness names its equation and `Cell` row → that
  cell element; a key witness names two `Cell` rows that share
  (factory, kind) → the same single cell element;
- a hypothesized or `outcome: unknown` finding is a question, never a
  detection;
- an executed construction rejection is recorded, never counted;
- an empty unique set is `killed`; a non-empty unique set with an
  executed baseline arm is `survived`; a claim resting on a hypothesis or
  construction probe is `inconclusive` at best.

## Pre-registered predictions (hypotheses, not results)

| Entry | Baseline detections | Probe elements | Unique | Predicted verdict |
|---|---|---|---|---|
| `near-floor` clean | ∅ | ∅ | ∅ | `killed` |
| `near-floor` fault (a) | `sample:ModernFactory/chair:product-missing` | `cell:ModernFactory/chair` | ∅ | `killed` |
| `near-floor` fault (b) | `sample:VictorianFactory/sofa:family-mismatch` | `cell:VictorianFactory/sofa` | ∅ | `killed` |
| `missing-realization` | ∅ | `cell:ModernFactory/table` | 1 | `survived` |
| `unknown-product` | ∅ | ∅ (construction) | ∅ | `killed` |
| `wrong-kind` | ∅ | `cell:ModernFactory/sofa` | 1 | `survived` |
| `duplicate-declaration` | ∅ | `cell:ModernFactory/sofa` | 1 | `survived` |
| `unrealized-kind` | ∅ | `kind:bench` | 1 | `survived` |
| `missing-combination` | ∅ | ∅ | ∅ | `killed` |

Predicted verbatim witness shapes (indices are dense row ids over sorted
presentation names):

- fault (a): `{"label":"executed","kind":"dangling","witness":{"arrow":"cell_product","src":0}}` (`cell_modern_chair` is Cell row 0);
- fault (b): `{"label":"executed","kind":"violation","witness":{"equation":0,"row":3,"lhs":0,"rhs":1}}` (T4 on `cell_victorian_sofa`);
- `missing-realization`: `dangling cell_product`, `src` = the row id of `cell_modern_table`;
- `wrong-kind`: `violation` equation 1 on `cell_modern_sofa`;
- `duplicate-declaration`: `violation` key 0, rows of `cell_modern_sofa` and `cell_modern_sofa_alt`;
- `unrealized-kind`: `dangling kind_witness`, `src` = the row id of `bench` (Kind rows sort `bench` first);
- `missing-combination`: `findings: []`, exit 0.

## Honesty gate — what this kata does not claim

- It does not claim a detection-rate win. The conventional arm is a
  **frozen, partial sample**; a conventional test that enumerated the
  whole declared grid and asserted T1–T5 would reach parity. The
  near-floor control (complete sample) is the parity demonstration.
- The positive band is therefore scoped: *given* the declared grid, the
  kernel names the unfilled or disagreeing cell without being told which
  ones to look at; the baseline looks only where its frozen sample says.
- The universal-over-cross-product reading of "every declared product
  kind is realized by a concrete factory" is **not** proved: G1 is the
  named boundary and `05-missing-combination` is its null result.
- The functor laws are not tested (G2). The disposition row can only move
  from "disposed" to a scoped executed row, not to a functor-law claim.

## Kill condition

- The control must self-negate (empty unique, verdict `killed`).
- A positive entry survives only with a non-empty unique set derived from
  **executed** evidence and an executed baseline arm. It is killed if the
  probe arm adds nothing outside the sampled cells.
- If a hypothesized question or a construction rejection is the only
  "detection", the entry is `inconclusive` at best; it is never upgraded
  to an executed claim.
- If cross-product completeness cannot be expressed, it is recorded as a
  gap/null result (G1), never fabricated as a finding.

## Falsifiers

- Any probe finding outside the pre-registered element set.
- Any baseline detection outside the frozen sample.
- A different witness shape than predicted above.
- A fixture edit after the run: the canonical model hashes pinned in the
  ledger entries (`fixture_hash`, `model_hashes`) change, and the
  correction must be a superseding ledger entry.

## Observed results (recorded after the run)

**Baseline arm** (`python3 prototype/abstract_factory_baseline.py
--assert`, exit 0): sha256
`4f635702ad371a9c1155965763f18e2fda0d8a6289455fb708fe60e2dcb07c97`,
6 entries, 3 unique elements. Observed detections:

| Fixture | Baseline detection | Probe element | Unique |
|---|---|---|---|
| `near-floor.ct.json` | — | — | — |
| `near-floor-fault-missing.ct.json` | `sample:ModernFactory/chair:product-missing` | `cell:ModernFactory/chair` | ∅ |
| `near-floor-fault-family.ct.json` | `sample:VictorianFactory/sofa:family-mismatch` | `cell:VictorianFactory/sofa` | ∅ |
| `01-missing-cell.ct.json` | — | `cell:ModernFactory/table` | 1 |
| `06-unknown-product.ct.json` | — (construction; arm not run) | — | ∅ |
| `02-wrong-kind.ct.json` | — | `cell:ModernFactory/sofa` | 1 |
| `03-duplicate-cell.ct.json` | — | `cell:ModernFactory/sofa` | 1 |
| `04-unrealized-kind.ct.json` | — | `kind:bench` | 1 |
| `05-missing-combination.ct.json` | — | — | ∅ |

**Probe arm, executed findings verbatim.** Labels below are what the
kernel emitted, not assignments:

- control missing product:
  `{"label":"executed","kind":"dangling","witness":{"arrow":"cell_product","src":0}}`
- control family cross:
  `{"label":"executed","kind":"violation","witness":{"equation":0,"row":3,"lhs":0,"rhs":1}}`
- missing realization:
  `{"label":"executed","kind":"dangling","witness":{"arrow":"cell_product","src":2}}`
- wrong kind:
  `{"label":"executed","kind":"violation","witness":{"equation":1,"row":1,"lhs":0,"rhs":1}}`
- duplicate declaration:
  `{"label":"executed","kind":"violation","witness":{"key":0,"rows":[1,2]}}`
- unrealized kind:
  `{"label":"executed","kind":"dangling","witness":{"arrow":"kind_witness","src":0}}`
- missing combination: `check` exit 0, `findings: []`.
- undeclared product (`06-unknown-product.ct.json`): exit 1, envelope
  `code: syntax`, message
  ``table `cell_product` maps `cell_modern_chair` to `GhostChair`, which is not a row of `Product` (line 154, column 30)``
  — recorded as a construction probe, never counted.

All nine predicted witness/exit shapes matched the recorded run.

**Recorded discrepancy (construction-run baseline).** The first baseline
implementation ran its dict-level sample against the raw JSON of
`06-unknown-product.ct.json` and identified `cell:ModernFactory/chair`
(`sample:ModernFactory/chair:product-undeclared`). The stated protocol is
a conventional test suite that loads the document before calling a
factory; an unloadable document is therefore recorded as *arm not run*,
not as a detection. The script was corrected to that protocol rule and
the correction is recorded in the ledger entry's `discrepancies` field.
The unique set is ∅ under either reading, so no detection claim moved.

**Ledger.** `ledger-check ledger/katas/abstract-factory` exits 0:
`ok: 6 entries (6 kata), rollup verdict: (no rollup)`. Entries (in seq
order):

| seq | file | verdict |
|---|---|---|
| 1 | `0001-fb741fb3.json` (control) | `killed` |
| 2 | `0002-2f2b8c37.json` (missing realization) | `survived` |
| 3 | `0003-a78ab015.json` (wrong kind) | `survived` |
| 4 | `0004-81dbfb80.json` (duplicate declaration) | `survived` |
| 5 | `0005-862cf5c3.json` (unrealized kind) | `survived` |
| 6 | `0006-e9da5c4d.json` (missing combination) | `killed` (null result) |

**Verdict.** Law AF-T is executable **scoped to the declared grid**:
T1/T2 name unfilled declared cells, T3/T4/T5 name disagreement on
declared cells, and the near-floor control self-negates. The candidate
law's cross-product reading is **not** proved: G1 is the named boundary
and entry 6 records the null result. No functor-law claim is made (G2).