# Kata — Iterator traversal law over a declared sequence (pre-registration)

Ticket `catherine-r5k.4`. Disposition input: `docs/ref/pattern-kata-dispositions.md`
carries the Iterator row as **disposed** — "traversal state is behavioral,
not a doc invariant; nothing here is validated". This kata names a law
first, then tests whether it is executable on the current kernel surface.

Committed **before** the recorded run. Nothing above the observed-results
section is edited after the run; a correction is a superseding ledger
entry.

## Law (named before any fixture was written)

**Declared-sequence traversal law.** Let `E` be a declared finite sequence
(dense rows of `Element`, named so dense order is declared order) and `S` a
declared traversal of it (dense rows of `Step`, in emission order), with
emission map `yields : Step → Element` and the documented inverse
`back : Element → Step`. The law is the conjunction:

1. `yields` is **total** — every step emits a declared element (an
   unresolved cell is the kernel's `dangling` surface);
2. `yields` is **injective** — no element is emitted twice;
3. `yields ; back = id_Step` — the documented inverse of an emission is the
   step that emitted it;
4. `back ; yields = id_Element` — every declared element is emitted by the
   step its inverse names (this is the "no skipped element" half, encoded
   through the declared inverse because the kernel has no existential or
   coverage predicate).

1–4 together state: **the traversal is a bijection onto the declared
sequence — every declared element exactly once.** That is the candidate
law from the brief, with its two clauses (no duplication, no skipping)
made explicit.

**Order clause (separate, not implied).** The bijection is order-blind. The
order clause is

    succ_s ; yields = yields ; succ_e

over declared successor cells (`succ_s : Step → Step`, `succ_e : Element →
Element`). Whether it is expressible depends on modelling order as a cell;
the kata measures that boundary (entries 4 and 5).

**Explicitly not claimed.** The iterator's internal cursor state. The model
records the traversal's *emissions*, never the cursor machine; the law is a
law about the declared traversal table. The disposition row's objection
("traversal state is behavioral") is therefore narrowed, not refuted: what
is checked is the declared emission table, not the running iterator.

## Pattern-equivalence mapping (Iterator ≅ declared emission table)

| Iterator element | `.ct.json` encoding |
|---|---|
| the collection / declared sequence | object `Element`; dense rows in declared order (`e0_alpha`, `e1_beta`, … sort as declared) |
| the traversal's emissions (`next()` / `currentItem()` sequence) | object `Step`; dense rows in emission order, arrow `yields : Step → Element` |
| "the i-th emission is the i-th declared element" | entries 4–5 only: the successor equation over declared `succ_*` cells |
| "no element emitted twice" | declarative key `Step[yields]` (injectivity witness names the two colliding step rows) and `yields ; back = loop_step` |
| "no element skipped" | arrow totality plus `back ; yields = loop_elem`; an element no step emits is a null `back` cell → `dangling` |
| "the documented inverse is the emitting step" | the accepted equations 2–3 |
| identity (`id_Step`, `id_Element`) | declared self-tables `loop_step`, `loop_elem` — the kernel has no empty path, so identity is **data**, not a primitive (named gap) |

## Encoding decisions (stated up front, never silent)

- **Identity is a declared self-table.** The kernel's equation paths must
  have a non-empty arrow list, so `id` is encoded as `loop_step` /
  `loop_elem` mapping every row to itself. The equations are checked
  *relative to that declared table*; whether the table really is the
  identity is not verifiable (named gap 1).
- **Successor cells are declared total chains with sentinel terminals.**
  `succ_e`/`succ_s` use final self-loops on the sentinel rows `e9_end` /
  `s9_end` so the encoding manufactures no `dangling` at the end of a
  sequence. Sentinel rows are outside the element vocabulary and outside
  the declared sequence.
- **`back` is modelling data, not kernel logic.** It is the documented
  inverse a reviewer would write; the kernel checks it against `yields`.
- **Finite-instance scope.** The law is checked over the declared rows.
  It is not a universal statement over arbitrary collections; it is the
  same scoped claim every kata in this series records.
- Fixtures are format 1.3 (they declare a key).

## Fault matrix (pre-registered)

Element vocabulary: one element is one declared `Element` row, spelled
`element:<row name>` (e.g. `element:e1_beta`). Six entries, seven fixtures.

| Entry | Fixture(s) | Injected fault | Kernel surface (predicted) |
|---|---|---|---|
| `floor-control` (control) | `floor-control.ct.json` + `clean-traversal.ct.json` | three-element traversal emits `alpha, beta, beta`: `beta` duplicated at the tail, `gamma` never emitted | key collision step rows 1,2; eq violation step row 2; `dangling back@2` |
| `duplicate-emission` | `duplicate-emission.ct.json` | replay: six steps over five elements, the last element emitted twice (no skip) | key collision step rows 4,5; eq violation step row 5 |
| `skipped-element` | `skipped-element.ct.json` | traversal omits the middle element `gamma`; the rest stay in order | `dangling back@2` (`e2_gamma`) |
| `reorder-order-blind` | `reorder-order-blind.ct.json` | `gamma`/`delta` swapped, bijection intact, no successor cells declared | **none** — the encoded law is order-blind |
| `reorder-modeled-swap` | `reorder-modeled-swap.ct.json` + `reorder-modeled-clean.ct.json` | the same swap with declared successor cells | order-equation violations at step rows 1, 2, 3 |
| `duplicate-covered` | `duplicate-emission.ct.json` | the same replay fault, baseline sample covering the whole traversal | key collision + eq violation (parity ceiling) |

## Conventional baseline protocol (frozen before the run)

`prototype/iterator_baseline.py` is the conventional arm: a **scripted
smoke walk** of the traversal over a pre-registered position sample. For
each sampled position `i` the walk asserts

    emitted[i] == declared[i]

and names **both sides** when it fails (a failing assertion prints the
expected and the actual value). A sampled position past the declared
sequence names the unexpected emission. The walk does **not** assert total
length, termination, or the unreached tail. It reads the fixture's declared
sequence and emission table, so it tests the same evidence the kernel reads;
what it does not do is enumerate the whole traversal.

Pre-registered position samples:

| Fixture | Sample |
|---|---|
| `floor-control.ct.json` | `[0, 1, 2]` (complete three-position window) |
| `clean-traversal.ct.json` | `[0, 1, 2, 3]` (complete) |
| `duplicate-emission.ct.json` | `[0, 1, 2]` (of six steps) |
| `skipped-element.ct.json` | `[0, 1]` (of five steps) |
| `reorder-order-blind.ct.json` | `[0, 1]` |
| `reorder-modeled-clean.ct.json` | `[0, 1]` |
| `reorder-modeled-swap.ct.json` | `[0, 1]` |
| `duplicate-emission.ct.json` (parity entry) | `[0, 1, 2, 3, 4, 5]` (complete) |

Sampling is the point: this is a bounded conventional test, not an
enumeration. The control's sample is the **complete** window, so the
control measures what a complete window catches, and entry 6 repeats the
duplicate fault with a complete sample to record the parity ceiling
explicitly.

The baseline arm runs **before** the probe arm inside the script; the
ledger records the script hash and the executed mode.

## Comparison rule (fixed before the run)

`unique = probe elements − baseline-identified elements`, arithmetically.
Witness-to-element mapping:

- **key** collision `{key, rows}`: both rows are `Step` rows; the element
  is the shared emission `yields[rows[0]]` (the element emitted twice);
- **dangling** on arrow `back`: the witness row is an `Element` row, so the
  element is that row (the element no step emitted);
- **violation** `{equation, row}`: the equation index is the position in the
  parser's canonical equation order (sorted by `(lhs, rhs, rationale,
  status)`); the lead arrow's source decides the domain. A `Step`-domain
  equation maps the row through `yields` (the element the step actually
  emitted); an `Element`-domain equation names the element row itself.
- the baseline identifies exactly the elements its failed assertions name.

A finding that maps to no element is recorded verbatim and never counted.
Verdict follows the recorded rule: empty unique = `killed`; unique with an
executed baseline arm = `survived`.

## Pre-registered predictions (hypotheses, not results)

| Entry | Baseline detections | Probe elements | Unique | Predicted verdict |
|---|---|---|---|---|
| `floor-control` | `sample:pos2:expected-e2_gamma-got-e1_beta` | `element:e1_beta`, `element:e2_gamma` | ∅ | `killed` |
| `floor-control` clean run | ∅ | ∅ | ∅ | — |
| `duplicate-emission` | ∅ (sample is a prefix of the six emissions) | `element:e4_epsilon` | `element:e4_epsilon` | `survived` |
| `skipped-element` | ∅ (sample is a prefix) | `element:e2_gamma` | `element:e2_gamma` | `survived` |
| `reorder-order-blind` | ∅ | ∅ (kernel silent) | ∅ | `killed` |
| `reorder-modeled-swap` | ∅ | `element:e1_beta`, `element:e2_gamma`, `element:e3_delta` | same | `survived` |
| `duplicate-covered` | `sample:pos5:unexpected-emission` naming `e4_epsilon` | `element:e4_epsilon` | ∅ | `killed` |

Predicted verbatim findings are pinned in `faults.json` (`findings_expected`
per run), including exit codes: heap violations exit 3, dangling-only exits
4, silence exits 0.

## Kill condition

- The control must self-negate (empty unique, verdict `killed`) and the
  clean fixture must emit zero findings (no false positive).
- A positive entry survives only with a non-empty unique set derived from
  **executed** evidence and an executed baseline arm.
- `duplicate-covered` must be `killed`: with the sample covering the fault,
  the probe must buy nothing. If it bought something, the comparison rule is
  wrong.
- `reorder-order-blind` must be `killed` with an empty probe set: the
  encoded law is order-blind by construction; a finding there would mean the
  encoding smuggles order in.
- If the order clause turns out unexpressible even with declared successor
  cells, it is recorded as a named gap, not fabricated as a finding.

## Falsifiers

- Any probe finding outside the pre-registered set, by kind, witness, or
  element mapping.
- Any baseline detection outside the frozen sample.
- A fixture edit after the run: the fixture hashes pinned in the ledger
  entries change and the entry ids no longer recompute.
- The clean fixtures emitting a finding (the encoding manufactures a false
  positive).

## Named expressibility gaps (pre-registered, to be confirmed by the run)

1. **Identity is not a primitive.** The kernel has no empty path; `id_Step`
   and `id_Element` are declared self-tables, and the kernel cannot verify a
   self-table is the identity. Equations 3–4 are checked only relative to
   that declared data.
2. **Order needs a modeled cell.** Dense row ids follow the parser's sorted
   row names, and the kernel has no index or successor arithmetic, so
   emission order is not expressible from `yields` alone. Order is
   expressible only by declaring `succ_*` cells — and even then the
   successor relation itself is unverified data.
3. **Coverage is encoded, not quantified.** "Every declared element is
   emitted" is expressed as a declared inverse whose cell is `null` for an
   unemitted element (`dangling`), not as an existential/coverage
   constraint. The model author must fill `back`; the kernel never states
   "the image of `yields` is all of `Element`" as a universal.
4. **Runtime cursor state is out of scope.** No cursor, no `hasNext`, no
   mutation order is modeled: the law validates the declared emission table.
5. **The unique band is coverage, not modelling.** A conventional *complete*
   sequence comparison catches every seeded multiplicity fault (entry 6 is
   the parity ceiling). The unique band exists only where the pre-registered
   sample does not look — a scoped claim, never a detection-rate claim.

## Execution

```
python3 prototype/iterator_baseline.py --assert
cargo run -q -p catherine-cli -- check fixtures/katas/iterator/<fixture> --json
cargo run -q -p catherine-ledger --bin ledger-check ledger/katas/iterator
```

## Observed results (recorded after the executed run)

Everything below this line is observed, never predicted.
**Baseline arm, executed first.** `python3 prototype/iterator_baseline.py
--assert` exited 0 with `baseline_sha256 =
9011179b321d92d855a2199dbfffb6746325fad488a0068dc2a3dc0221092a62`,
8 runs over 6 entries, and five unique detections across the three
positive entries (four distinct elements).

| Entry | Run | Baseline detections | Probe findings | Probe elements | Unique | Verdict |
|---|---|---|---|---|---|---|
| `floor-control` | `floor-control.ct.json` | `sample:pos2:expected-e2_gamma-got-e1_beta` | 3 | `e1_beta`, `e2_gamma` | ∅ | `killed` |
| `floor-control` | `clean-traversal.ct.json` | ∅ | 0 | ∅ | ∅ | (no finding) |
| `duplicate-emission` | `duplicate-emission.ct.json` | ∅ | 2 | `e4_epsilon` | `e4_epsilon` | `survived` |
| `skipped-element` | `skipped-element.ct.json` | ∅ | 1 | `e2_gamma` | `e2_gamma` | `survived` |
| `reorder-order-blind` | `reorder-order-blind.ct.json` | ∅ | 0 | ∅ | ∅ | `killed` |
| `reorder-modeled-swap` | `reorder-modeled-swap.ct.json` | ∅ | 3 | `e1_beta`, `e2_gamma`, `e3_delta` | same | `survived` |
| `reorder-modeled-swap` | `reorder-modeled-clean.ct.json` | ∅ | 0 | ∅ | ∅ | (no finding) |
| `duplicate-covered` | `duplicate-emission.ct.json` | `sample:pos5:unexpected-emission` | 2 | `e4_epsilon` | ∅ | `killed` |

**Probe arm, findings verbatim** (`catherine check --json`, exit codes
observed). Labels are what the kernel emitted, not assignments.

- `floor-control.ct.json`, exit 3:
  `{"label":"executed","kind":"violation","witness":{"equation":1,"row":2,"lhs":1,"rhs":2}}`,
  `{"label":"executed","kind":"violation","witness":{"key":0,"rows":[1,2]}}`,
  `{"label":"executed","kind":"dangling","witness":{"arrow":"back","src":2}}`
- `clean-traversal.ct.json`, exit 0: `[]`
- `duplicate-emission.ct.json`, exit 3:
  `{"label":"executed","kind":"violation","witness":{"equation":1,"row":5,"lhs":4,"rhs":5}}`,
  `{"label":"executed","kind":"violation","witness":{"key":0,"rows":[4,5]}}`
- `skipped-element.ct.json`, exit 4:
  `{"label":"executed","kind":"dangling","witness":{"arrow":"back","src":2}}`
- `reorder-order-blind.ct.json`, exit 0: `[]`
- `reorder-modeled-clean.ct.json`, exit 0: `[]`
- `reorder-modeled-swap.ct.json`, exit 3:
  `{"label":"executed","kind":"violation","witness":{"equation":1,"row":1,"lhs":3,"rhs":2}}`,
  `{"label":"executed","kind":"violation","witness":{"equation":1,"row":2,"lhs":2,"rhs":4}}`,
  `{"label":"executed","kind":"violation","witness":{"equation":1,"row":3,"lhs":4,"rhs":3}}`

**What the witnesses mean, observed.** The control's key collision names
`e1_beta` (emitted by both `s1` and `s2`) and its `yields;back` violation
fires at `s2` (the step's emission returns `s1`); the dangling `back@2`
names `e2_gamma`. The duplicate replay names `e4_epsilon` twice — once as
the shared target of the colliding steps `s4`/`s5`, once through the
round trip at `s5`. The skipped element is a single null inverse cell. The
order-modeled swap fires the successor equation at three consecutive step
rows (1, 2, 3) because the relation is local: mapping those rows through
`yields` names `e1_beta`, `e3_delta` and `e2_gamma` — the two swapped
elements plus their predecessor, which is the equation's honest local
scope, not a claim that beta is misplaced.

**Named gaps, confirmed by the run.**

1. Identity is not a primitive: the accepted equations are stated against
   the declared self-tables `loop_step`/`loop_elem`, whose self-ness the
   kernel cannot verify.
2. Order is not expressible from the bijection: `reorder-order-blind`
   emitted nothing on a real swap. It became expressible only with
   declared successor cells (`reorder-modeled-swap`), which are themselves
   unverified data. This is the boundary the brief's candidate law pointed
   at: order must be a modeled cell.
3. Coverage is encoded, not quantified: a skipped element is a null
   declared inverse (`dangling`), not an existential/coverage constraint.
4. Runtime cursor state is outside the model: the law holds over the
   declared emission table, not the iterator's internal state.
5. The non-empty unique sets are sampling coverage, not modelling
   advantage: `duplicate-covered` shows the conventional walk catching the
   same element once its sample covers the fault. No detection-rate claim
   is made anywhere in this kata.

**Predictions vs observations.** All eight runs matched the pre-registered
findings, exit codes, baseline detections, element sets and unique sets
exactly; no discrepancy was observed and nothing above was edited after
the run.

**Ledger.** Six entries in `ledger/katas/iterator/` (control first):
`seq 1` `killed`, `seq 2` `survived`, `seq 3` `survived`, `seq 4` `killed`
(named gap), `seq 5` `survived`, `seq 6` `killed` (parity ceiling).
`ledger-check ledger/katas/iterator` exits 0:
`ok: 6 entries (6 kata), rollup verdict: (no rollup)`.

**Outcome, stated plainly.** The law from the brief is expressible on the
current kernel surface, but only through two encodings that must be named
(identity as declared self-tables; coverage as a declared inverse), and
its order clause needs order modelled as successor cells — without them
the encoding is order-blind and silent on a real swap. Against a complete
conventional sequence comparison the law buys nothing (entry 6 is the
parity ceiling); the positive band exists only where the pre-registered
smoke walk does not look. The disposition row's "disposed" should become a
scoped executed record with that boundary, not a claim of advantage.
