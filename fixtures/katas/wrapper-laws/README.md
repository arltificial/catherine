# Kata — wrapper laws (Adapter / Proxy / Decorator): pre-registration

Committed **before** the executed run (beads `catherine-lgl.6`). Nothing in
the pre-registered sections may be edited after the run: a correction is a
superseding ledger entry, never an edit.

## Provenance (honest scope, no prior series run)

`docs/ref/ploeh-grounding.md` §6 proposes four reverse-engineering kata
candidates (rule lens, composite-as-monoid, visitor-as-sum-type, state
discovery). **Adapter/Proxy/Decorator is not among them**, so this kata
carries no series row to map a verdict against and no ct-catlab kata
number. It is HYPOTHESIZED from the same corpus — the ploeh series hub
(*From design patterns to category theory*,
<https://blog.ploeh.dk/2017/10/04/from-design-patterns-to-category-theory/>),
whose Adapter / Proxy / Decorator articles state delegation laws informally
— and it inherits the grounding's operating principle (§3): a declaration
does not enforce, a check does.

The kata therefore makes **no series claim**. Its deliverable is the
pre-registered comparison below: does a declared path-equation battery on
wrapper boundaries find anything a sampled delegation suite does not, and
does it self-negate at the identity-wrapper floor?

## The laws and their executable form

`wrapper.call ≡ target.call` is a relation between rows of *two* different
objects. Catherine equations are parallel paths from **one** source
object (`parse/schema.rs::check_equations` types both sides against the
same src/tgt pair), so the cross-object form is not expressible. The
closest executable law introduces the wrapper's declared link arrow and
reads:

| Law | Executable path equation | Parsed index | Meaning |
|---|---|---|---|
| L1 adapter delegation | `adapts ∘ result = adapter_result` | 2 | the adapted call returns the wrapped store's result |
| L2 adapter identity | `adapts ∘ key = adapter_key` | 0 | identity survives translation, not just the payload |
| L3 adapter field | `adapts ∘ note = adapter_note` | 1 | a translation may not drop a declared field |
| L4 proxy delegation | `proxies ∘ result = proxy_result` | 5 | the proxy boundary is transparent for the result |
| L5 proxy identity | `proxies ∘ key = proxy_key` | 4 | a proxy may not rewire identity |
| L6 decorator transparency | `decorates ∘ result = decorator_result` | 3 | a decorator adds behaviour, never changes the answer |
| L7 decorator stacking | `stack_of ∘ decorator_result = stack_result` | 6 | a stack returns the underlying result; two orders over one store must agree |

**Parsed index, not declaration order.** The parser sorts equations by
path key (`check_equations`), so the ledger's `violation:eq<n>` witnesses
cite the parsed index: adapter laws are `0/1/2`, decorator transparency
`3`, proxy identity `4`, proxy delegation `5`, stacking `6`. The floor
control's three laws parse as `0` (key), `1` (note), `2` (result).

## Fixtures

| Path | Role |
|---|---|
| `wrapper-laws.ct.json` | the clean above-floor model: 3 stores, adapter / proxy / decorator wrappers, 2 decorator stack orders (d1 log-over-metrics, d0 metrics-over-log), 7 laws |
| `faults/01-adapter-drops-field.ct.json` | W1: `adapter_note` cells dropped on all 3 rows (rows kept, values gone — the migration lost-cell shape) |
| `faults/02-proxy-rewires-identity.ct.json` | W2: proxy row 1 reports `k_acct` while its store's key is `k_cart` |
| `faults/03-decorator-order-swaps-result.ct.json` | W3: the reversed stack order (`st_ml`, row 1) returns `r_beta` instead of `r_alpha` |
| `faults/04-wrapper-loses-optional.ct.json` | W4: `decorator_note` cell dropped on row 1; no law mentions the field |
| `floor/identity-wrapper.ct.json` | near-floor control: one store, one identity wrapper, three observables, three laws |
| `floor/faults/01-identity-wrapper-swaps-result.ct.json` | C1: the wrapper returns a different result |
| `floor/faults/02-identity-wrapper-rewires-key.ct.json` | C2: the wrapper reports a different key |
| `floor/faults/03-identity-wrapper-loses-note.ct.json` | C3: the wrapper's auxiliary cell is dropped |

The clean model must produce **zero** findings: a law battery that fires on
its own clean model would invalidate the kata.

## Fault matrix (pre-registered)

Elements are the kernel's witness ids: `dangling:<arrow>#<row>` (dense row
ids are the parser's sorted-name order) or
`violation:eq<index>@row<row>`.

| Fault | Control | Class | Injected change | Probe elements | Baseline | Unique | Verdict |
|---|---|---|---|---|---|---|---|
| C1 identity-wrapper-swaps-result | yes | behavioral | wrapper result `r_beta` vs store `r_alpha` | `violation:eq2@row0` | catches (1 assertion) | 0 | killed |
| C2 identity-wrapper-rewires-key | yes | structural | wrapper key `k_cart` vs store `k_acct` | `violation:eq0@row0` | catches | 0 | killed |
| C3 identity-wrapper-loses-note | yes | structural | wrapper note cell dropped | `dangling:wrapper_note#0` | catches | 0 | killed |
| W1 adapter-drops-field | no | structural | all three `adapter_note` cells dropped | `dangling:adapter_note#0,#1,#2` | catches row 0 only | 2 | survived |
| W2 proxy-rewires-identity | no | structural | proxy row 1 key rewired | `violation:eq4@row1` | misses (identity sampled only at row 0) | 1 | survived |
| W3 decorator-order-swaps-result | no | behavioral | reversed stack order returns `r_beta` | `violation:eq6@row1` | misses (one stack order sampled) | 1 | survived |
| W4 wrapper-loses-optional | no | structural | `decorator_note` dropped at row 1 | `dangling:decorator_note#1` | misses (field unsampled) | 1 | survived |

Pre-registered unique total: **5** above the floor, **0** at the floor.
The control must self-negate (verdict `killed`); the positive kata is
`survived` only under the frozen sample, which the baseline section
deliberately restricts.

## Conventional baseline (frozen before the run)

`prototype/wrapper_laws_baseline.py` — the scripted delegation assertions a
team writes for a wrapper boundary: for each sampled (wrapper, field, row)
cell, assert `wrapper.field == target.field` through the wrapper's declared
link. The sample is frozen in `faults.json` (`sample`, `sample_size`), not
inferred after the run:

- **above floor, 13 assertions**: results for every adapter / proxy /
  decorator row and the sampled stack order (3+3+3+1); keys spot-checked
  at one row per wrapper kind (adapter row 0, proxy row 0); auxiliary
  notes spot-checked at adapter row 0 only; decorator notes not sampled;
  the second stacking order not sampled.
- **floor, 3 assertions**: every observable of the single wrapper — the
  floor's surface is small enough that the sample is exhaustive.

A baseline assertion identifies an element **only** when it disagrees and
the cell maps to a pre-registered law: a dangling wrapper cell identifies
`dangling:<field>#<row>`; a wrong value identifies
`violation:eq<law>@lawrow` (`faults.json::surface`). Nothing else counts —
no near-miss credit.

**Density caveat, stated up front.** The suite is a sample, not the
cross-product. A complete delegation suite asserting all 23 declared
above-floor cells would identify all five elements in the pre-registered
unique set, because every one is a cell (or a stack row) the sample skips.
The kata therefore tests coverage of unsampled cells, not a defect class a
complete conventional suite cannot reach. This is the parity ceiling and it
is recorded as such.

## Near-floor control

`floor/identity-wrapper.ct.json`: the smallest wrapper that still carries
the three laws (result, identity, auxiliary field). Three observables, three
laws, three conventional assertions — the sample is exhaustive, so every
injected fault is caught and the probe buys **zero** unique detections. The
control must be `killed`; if it records any unique detection, the floor
claim is refuted.

## Expressibility gaps (pre-registered, never forced)

1. **Cross-object equation form.** `wrapper.call ≡ target.call` relates two
   objects' rows; catherine's equations are parallel paths from one source
   object. The kata states the law through a functional wrap link, which
   assumes exactly one wrapped target per wrapper. A wrapper delegating to
   several targets, or a law quantifying over a call argument, has no form.
2. **Optionality.** Every declared arrow is total: an absent cell is
   `dangling`. The model cannot declare a field *optional*, so a deliberate
   absence and a silent loss are the same finding (W4 is reported by
   totality, not by a law; the "optional" naming is the library's, not the
   model's).
3. **Runtime behaviour.** The model compares declared result rows. Whether
   the wrapper actually dispatches at runtime — that the proxy *calls
   through* rather than fabricating a row — is not observable; L1/L4/L6 are
   the closest executable statements, not execution traces.
4. **Object identity.** "The proxy is the same entity as its target" has no
   surface: there is no object-identity or subobject relation. Identity is
   modelled as agreement of a declared `key` arrow (L2/L5), which is a
   weaker claim.

## Verdict rule (pre-registered)

- **Control entry**: `killed` required. Any unique detection at the floor
  means the "conventional testing suffices here" claim is refuted, and the
  run is a falsifier, not a result.
- **Positive entry**: `survived` iff at least one unique detection against
  the executed baseline arm; `killed` iff zero (parity — the null result is
  the deliverable); `inconclusive` if the baseline could only be cited,
  never executed. The verdict is derived by the recorded procedure, never
  asserted, and the unique total is arithmetic from the frozen matrix.

## Falsifiers

- Any probe finding outside the pre-registered sets — the fixture no longer
  instantiates the declared law; record the discrepancy, never edit the
  matrix.
- Any baseline detection drift or `--assert` mismatch — the frozen sample
  changed or the fixture did; re-pin only with a superseding entry.
- A clean fixture that fires — the law battery is inconsistent with its own
  model.
- Any unique detection at the floor control — the floor claim is refuted.

## Execution

```
python3 prototype/wrapper_laws_baseline.py --assert        # both arms + frozen expectations
python3 prototype/wrapper_laws_baseline.py --hash          # the pin carried in the ledger
cargo run -q -p catherine-ledger --bin ledger-check -- ledger/katas/wrapper-laws
cargo test -q -p catherine-ledger --test wrapper_laws_kata
```

## Observed results (recorded after the executed run)

`python3 prototype/wrapper_laws_baseline.py --assert` exited 0 against the
frozen matrix (script sha256
`08a61ed2061fc1c962d3b3523abf0fef639d54bffcc263df3c47d7727192772e`).
Both clean fixtures produced zero findings and zero baseline detections —
the law battery holds on its own model.

The arm was re-executed after a gate-mandated structural refactor of one
function (nesting, no behaviour change); the two runs' records are
byte-identical, and the ledger pins the re-executed script.

| Fault | Baseline detections | Probe elements | Unique | Verdict |
|---|---|---|---|---|
| C1 identity-wrapper-swaps-result | `delegation:Wrapper:wrapper_result#0:r_beta!=r_alpha` | `violation:eq2@row0` | 0 | killed |
| C2 identity-wrapper-rewires-key | `delegation:Wrapper:wrapper_key#0:k_cart!=k_acct` | `violation:eq0@row0` | 0 | killed |
| C3 identity-wrapper-loses-note | `delegation:Wrapper:wrapper_note#0:missing` | `dangling:wrapper_note#0` | 0 | killed |
| W1 adapter-drops-field | `delegation:Adapter:adapter_note#0:missing` | 3 `dangling` cells (`adapter_note` rows 0–2) | **2** | survived |
| W2 proxy-rewires-identity | — | `violation:eq4@row1` | **1** | survived |
| W3 decorator-order-swaps-result | — | `violation:eq6@row1` | **1** | survived |
| W4 wrapper-loses-optional | — | `dangling:decorator_note#1` | **1** | survived |

Unique detections: **5** above the floor, **0** at the near-floor control —
the control self-negates (verdict `killed`), exactly as pre-registered.

**What the five are.** One identity rewiring on an unsampled proxy row, one
reversed stacking order the sample never exercises, one dropped auxiliary
cell no law mentions (W4, caught by totality alone), and two adapter rows
whose dropped cells the single spot-checked note row does not cover. None
of them is a defect class a *complete* delegation suite misses: the frozen
sample is 13 of the model's 23 declared cells, and a cross-product suite
asserting all 23 would identify all five elements. The kata's edge here is
**coverage of cells the sampled suite skips**, not detection of a new
failure mode. The positive verdict is therefore scoped to sampled suites;
against a dense conventional suite the result is parity (unique 0).

**The near-floor control is the symmetric case**: at one wrapper and three
observables the sample is exhaustive, so the same three fault classes are
all caught conventionally and the probe buys nothing. The floor claim
("the tool must say conventional testing suffices") survives, and the
above-floor band is the coverage delta between the two widths.

**Faults exercised exactly one pre-registered gap each** — the cross-object
law form (every law is stated as `link ∘ target_field = wrapper_field`) and
optionality (W4 exists only as a totality finding). Runtime behaviour and
object identity were not exercised; they remain named, unexercised gaps.