# Kata — factory-method dispatch agreement (pre-registration)

Committed **before** the recorded run. The pre-registration sections are
not edited after the run: a correction is a superseding ledger entry.

Ticket: `catherine-r5k.2` (P3, lane:design). Disposition row under test:
Factory Method — *disposed (construction dispatch only; no invariant on a
finite instance)*. This kata exists to test that disposition, not to
assume it.

## Law named first (before any fixture)

**Dispatch agreement (Factory Method).** A creator's dispatch is a finite
relational structure, not only a construction act. Let `K` be the
declared product-kind inventory, `Creator` the declared concrete creators,
and let the legacy creator's table be `route : K → Route` (the switch's
arm per kind) with `handler : Route → Creator` (the arm's body). Let
`creator : K → Creator` be the inventory's declared per-kind creator — the
product each kind is defined to be built by. The law has two faces:

1. **Agreement:** `route ; handler = creator`. Every declared kind's
   dispatch resolves to *that kind's own* declared creator. This is the
   executable content of "no silent default": a `default:` arm is silent
   exactly when it resolves a declared kind to a handler that is not the
   kind's declared creator.
2. **Totality of the route:** every declared kind has a dispatch cell; a
   missing cell is a `dangling` finding. This face applies to a creator
   with no default arm.

Both faces are one declared equation plus the existing path/dangling
surface; the kernel enumerates **every** declared kind, never a sample.

**Honest boundary.** The declared creator map is authored specification —
it encodes exactly the expectation a conventional dispatch test asserts
for the kinds it samples. This kata measures the transfer into the
declared finite relation and its exhaustive evaluation, not a
detection-rate win: a conventional test that enumerates every declared
kind and asserts the concrete class reaches parity. The document is a
transcription of the legacy table; nothing here proves the running
binary's switch is the transcribed one.

## Pattern-equivalence mapping (fixed before the run)

| Factory Method element | `.ct.json` encoding |
|---|---|
| declared product-kind inventory (the switch's case labels) | object `ProductKind`, one row per kind |
| concrete creator / factory method per product | object `Creator`, one row per creator |
| the switch's arm for a kind | object `Route`, one row per arm; arrows `route : ProductKind → Route`, `handler : Route → Creator` |
| the inventory's intent "kind `k` is built by creator `h`" | arrow `creator : ProductKind → Creator` (authored spec) |
| "every kind routes to its own creator, no silent default" | accepted equation `route ; handler = creator` (`violation` on the failing kind's row) |
| "every kind has an arm (no default arm to hide behind)" | a `route` cell that is `null` → `dangling route@ProductKind#<row>` |
| a default arm that serves a kind with no specific case | **no predicate** — see the gap entries |

The kernel owns the enumeration; the fixtures only declare the relations.

## Substrate — `dispatch.ct.json` (clean)

`ProductKind` rows (alphabetical, so a witness row index is also the
declaration index): `json` 0, `text` 1, `xml` 2, `yaml` 3.
`Creator` rows: `create_default` 0, `create_json` 1, `create_text` 2,
`create_xml` 3, `create_yaml` 4. `Route` rows: `r_json` 0, `r_text` 1,
`r_xml` 2, `r_yaml` 3.

| kind | route | handler | creator |
|---|---|---|---|
| json | r_json | create_json | create_json |
| text | r_text | create_text | create_text |
| xml | r_xml | create_xml | create_xml |
| yaml | r_yaml | create_yaml | create_yaml |

`create_default` is declared and referenced by nothing in the clean table:
an unused fallback creator is clean (the inventory may keep one for a
future kind). The clean table has zero findings.

## Near-floor control — `floor/dispatch.ct.json`

Two kinds (`json` 0, `text` 1), two routes, every kind sampled by the
conventional dispatch test. Injected floor faults:

| Fault | Fixture | Defect | Pre-registered kernel class |
|---|---|---|---|
| NF1 | `floor/fault-missing-route.ct.json` | `route(text) = null` | `dangling route@ProductKind#1` |
| NF2 | `floor/fault-silent-default.ct.json` | `handler(r_text) = create_default` while `creator(text) = create_text` | `violation` eq0 row 1 |

The floor sample is **both** kinds, so the scripted test catches both: NF1
as an unrouted kind, NF2 as a wrong concrete class. Unique detections must
be empty → control verdict `killed`.

## Above-floor fault matrix (fixed before the run)

Frozen sample: **`json`, `text`** — the two formats the legacy suite
exercised; `xml` and `yaml` were added later and are unsampled.

| Fault | Fixture | Injected defect | Element | Pre-registered class | Baseline |
|---|---|---|---|---|---|
| F1 | `fault-silent-default.ct.json` | new kind `yaml`; its arm still falls through to `create_default` | `kind:yaml` | `violation` eq0 row 3 (executed) | unsampled → missed |
| F2 | `fault-missing-route.ct.json` | new kind `xml` added with no arm (`route(xml) = null`) | `kind:xml` | `dangling route@ProductKind#2` (executed) | unsampled → missed |
| G1 | `gap-declared-fallback.ct.json` | `yaml`'s arm is the fallback *and* `creator(yaml) = create_default` (the spec itself names the fallback) | `kind:yaml` | none — kernel silent | unsampled → missed (a *complete* conventional test would catch it; the kernel cannot, see below) |
| G2 | `gap-dead-creator.ct.json` | clean dispatch plus a `Creator` row `create_toml` referenced by no kind and no handler | — | none — kernel silent | — |
| P1 | `fault-undeclared-kind.ct.json` | `creator` table keys `toml`, not a `ProductKind` row | — | `syntax` rejection at parse (construction probe, never counted) | — |

G1 is the honest limit of the law: "handler is a fallback" is not a
predicate. When the authored spec names the fallback as a kind's creator,
the agreement equation is satisfied and the kernel is silent; the fault is
indistinguishable from an intentional default on the current surface.
G2 is the second limit: "every declared creator is used by at least one
kind" is an image-coverage/existential claim with no equation or
dangling form.

## Probe arm — the rule lens on the same dispatch (exploratory, recorded)

The visitor-style rule surface was the nearest existing encoding:
`rule_probe/rule-silent-default.ct.json` declares one rule per kind with a
specific arm plus a trailing catch-all `create_default` under
`PriorityOrder`, analysed over `kind=json,text,xml,yaml`. Predicted:
**zero findings** — `yaml` is covered by the catch-all, so the lens sees
no gap, and the catch-all is live, so it sees no redundancy. This is
recorded as a probe: the rule surface cannot carry the agreement law.

## Conventional baseline protocol (frozen before the run)

`prototype/factory_method_baseline.py` models the scripted factory test
the legacy team ran: for each **sampled** kind, call the creator's
dispatch and assert the returned concrete creator/class against the frozen
expectation. Resolution reads the fixture's own `route`/`handler` tables
(the legacy switch), while the expected class is the frozen sample in
`faults.json` (the test's own assertions — not read back from the
document's `creator` arrow). Kinds outside the sample are never inspected.

Detection strings:

- `sample:<kind>:unrouted` — the switch has no arm (route cell null);
- `sample:<kind>:wrong-creator:<actual>` — the arm resolves to a creator
  other than the frozen expected class.

The script supports `--assert` (every fixture's detections equal its
frozen `baseline_expected`; exit 1 on any drift) and `--hash` (sha256 of
the script, pinned into each positive entry's baseline artifact). The
baseline arm runs **before** the probe arm.

## Comparison rule

An element is `kind:<k>`. `unique = executed probe elements −
baseline-identified elements`. A `violation`/`dangling` witness is mapped
to its kind by the witness row (the equation's lead `route` is sourced at
`ProductKind`; rows are in declared alphabetical order, pinned in the
substrate). A parse rejection is a construction probe: recorded under
`run.probes`, never counted. A silent run on a gap fixture is a null
result, never a detection.

## Pre-registered predictions (hypotheses, not results)

| Entry | Baseline detections | Probe elements | Unique | Predicted verdict |
|---|---|---|---|---|
| control (floor) | NF1, NF2 | both floor kinds | ∅ | `killed` |
| agreement | ∅ (yaml unsampled) | `kind:yaml` | 1 | `survived` (coverage-only scope) |
| totality | ∅ (xml unsampled) | `kind:xml` | 1 | `survived` (coverage-only scope) |
| residual gaps | ∅ | ∅ (G1, G2 run silent; P1 refused at parse) | ∅ | `killed` (named missing predicates) |

## Second hypothesis (H2) — role-annotated fallback exclusion

**Pre-registered before H2's own fixtures were built or run.** H1's G1
(`gap-declared-fallback.ct.json`) was predicted to be silent and was
observed silent; that null stands as recorded and is never re-encoded.
H2 is a *separately scoped* hypothesis about the same pre-named clause —
"no silent default" — and it is opened here, before its fixtures exist,
because leaving it untested would license a false claim that the clause
has no predicate at all.

**Observation that motivates H2 (already recorded, not a result of H2):**
the canonical model hash of `gap-declared-fallback.ct.json` equals that of
`dispatch.ct.json` (`sha256:d638d420…`). The two documents express the
same canonical model under a different row label: `create_default` and
`create_yaml` are interchangeable presentation names when both are the
target of the same two cells. The *name* "default" is presentation; the
structure carries no fallback role. So no structure-level predicate can
distinguish "yaml is built by the fallback" from "yaml's dedicated
creator is that row".

**H2 encoding.** Declare the classification the structure lacks, and a
policy, then state the exclusion as an equation:

| element | encoding |
|---|---|
| creator role | object `Role`, rows `concrete` 0, `fallback` 1; arrow `role : Creator → Role` |
| policy "every declared kind is served by a concrete creator" | arrow `expected_role : ProductKind → Role`, every cell `concrete` |
| "no declared kind routes to the fallback" | accepted equation `route ; handler ; role = expected_role` (eq1) |

The price is explicit: the role classification is authored (someone must
say which creator is the fallback), and `expected_role` is a policy
constant, not per-kind expectation data. The two fixtures:

| Fixture | Contents | Pre-registered finding |
|---|---|---|
| `role/dispatch.ct.json` | four kinds, all creators concrete, `expected_role` all `concrete`; H1's eq0 also present | zero findings |
| `role/fault-silent-default.ct.json` | `yaml`'s arm resolves to `create_default` **and** `creator(yaml) = create_default`, so H1's eq0 is satisfied; `role(create_default) = fallback`, `expected_role(yaml) = concrete` | `violation` eq1 row 3 (executed) |

| `role/floor/dispatch.ct.json` | two-kind floor, clean | zero findings |
| `role/floor/fault-silent-default.ct.json` | floor `text` served by the fallback, spec agrees (eq0 satisfied) | `violation` eq1 row 1 (executed) |

**H2 predictions.** Frozen sample `{json, text}`: the above-floor fault on
`yaml` is unsampled, so the baseline misses it and the role clause buys
one unique element (`kind:yaml`) → `survived`, scoped to coverage. The
two-kind floor samples **both** kinds, so the scripted test catches the
floor fault and the role clause must record zero unique detections →
control `killed`.

**H2 falsifiers.** Silence on `role/fault-silent-default.ct.json`; a
finding on the clean role fixture; a witness row other than `yaml`
(row 3) / floor `text` (row 1); an eq0 firing in a fault fixture (that
would mean the creator map agrees differently than the fixture declares).
A falsified H2 is recorded as a killed entry with the observed refusal —
the encoding is never adjusted after the run.

## Kill conditions and falsifiers

- The control must self-negate (empty unique, `killed`).
- A positive entry survives only with a non-empty unique set from executed
  findings and an executed baseline arm. It is killed if the probe adds
  nothing outside the sampled kinds.
- If a fault yields no finding on the surface that was predicted to carry
  it, record the null result and the missing predicate; never re-encode it
  after seeing the result.
- Falsifiers: any finding outside the pre-registered class sets; any
  baseline/assert mismatch; any silent clean fixture; any witness row that
  does not resolve to the pre-registered kind.

## Execution order

```
python3 prototype/factory_method_baseline.py --assert          # conventional arm first
cargo run -q -p catherine-cli -- check <fixture> --json          # probe arm
cargo run -q -p catherine-cli -- rules rule_probe/rule-silent-default.ct.json \
    --values kind=json,text,xml,yaml --json                     # exploratory rule arm
cargo run -q -p catherine-ledger --bin ledger-check -- ledger/katas/factory-method
```
## Observed results (appended after the runs)

**Baseline arm** (`python3 prototype/factory_method_baseline.py --assert`,
exit 0): sha256
`62553030b06c98d87d43bed081b1d857b2d41fb9d243d63bf17ee93e0cbf3848`,
13 fixtures, 3 detections. Detections: `sample:text:unrouted` (NF1),
`sample:text:wrong-creator:create_default` (NF2), and the same
`sample:text:wrong-creator:create_default` on RNF1. Every clean, gap, and
unsampled-fault fixture records zero detections, exactly as frozen.

**Probe arm, executed findings verbatim** (`catherine check --json`; row
indices are the declared alphabetical rows):

| Fixture | Exit | Findings |
|---|---|---|
| `dispatch.ct.json` | 0 | `[]` |
| `fault-silent-default.ct.json` | 3 | `violation` eq0 row3 lhs0 rhs4 — `yaml` resolves to `create_default`, its declared creator is `create_yaml` |
| `fault-missing-route.ct.json` | 4 | `dangling route@src 2` — `xml` has no arm |
| `gap-declared-fallback.ct.json` | 0 | `[]` |
| `gap-dead-creator.ct.json` | 0 | `[]` |
| `fault-undeclared-kind.ct.json` | 1 | `syntax`: ``table `creator` keys row `toml`, which is not a row of `ProductKind` (line 41, column 9)`` |
| `floor/dispatch.ct.json` | 0 | `[]` |
| `floor/fault-missing-route.ct.json` | 4 | `dangling route@src 1` |
| `floor/fault-silent-default.ct.json` | 3 | `violation` eq0 row1 lhs0 rhs2 |
| `role/dispatch.ct.json` | 0 | `[]` |
| `role/fault-silent-default.ct.json` | 3 | `violation` eq1 row3 lhs1 rhs0 (eq0 satisfied) |
| `role/floor/dispatch.ct.json` | 0 | `[]` |
| `role/floor/fault-silent-default.ct.json` | 3 | `violation` eq0 row1 lhs0 rhs2 **and** `violation` eq1 row1 lhs1 rhs0 |

**Rule-lens probe arm** (`catherine rules
rule_probe/rule-silent-default.ct.json --values
kind=json,text,xml,yaml --json`): exit 0, `findings: []` — predicted
silence confirmed. The live catch-all covers `yaml`, so the rule surface
sees no gap; and because it is live, it is not redundant either. For
contrast, the same table with a specific rule for every declared kind
makes the catch-all `redundant` (exit 7); a *dead* default is visible, a
*live* default that masks a missing arm is not.

**The G1 hash observation.** Model identity (canonical model hash):
`model_hash(dispatch.ct.json) = model_hash(gap-declared-fallback.ct.json)
= sha256:d638d42020e1d090675b4351c0167acfd29e03507fb8a990ba30f0b4e9c2aaae`.
The two documents are the same canonical model under a different row
label; the label `create_default` carries no structure. This is the
executed proof that "handler is a fallback" is not a structure-level
predicate — it is the reason H2 declares a role arrow.

**H2 deviation, recorded not smoothed.** The H2 pre-registration predicted
one finding (eq1) on `role/floor/fault-silent-default.ct.json`. The
committed floor fixture's `creator(text)` cell names `create_text` while
its arm resolves to `create_default`, so eq0 fires as well as eq1 (two
`violation` findings, both on row 1). The fixture was not edited after the
run: the floor's control role (baseline parity, zero unique) is unaffected,
and the above-floor H2 claim — where `creator(yaml)` *is* `create_default`
so eq0 is satisfied and only eq1 fires — behaved exactly as predicted.

**Outcome against the law.** The candidate law IS expressible on the
existing surface, with a named price:

1. **Agreement** (`route ; handler = creator`) catches a new kind falling
   through to the default when the inventory declares the kind's own
   creator — executed `violation` on the unsampled kind.
2. **Totality** of `route` catches a new kind with no arm — executed
   `dangling`.
3. **Fallback exclusion** (`route ; handler ; role = expected_role`)
   catches the same fall-through even when the authored creator map also
   names the default — but only after the fallback role is declared; the
   canonical structure cannot derive it (G1's hash equality).
4. **Not expressible:** "every declared creator is used by at least one
   kind" (image coverage / existential negation) — G2 is silent; and the
   code's own switch is not modeled (the fixture is a transcription).

Verdicts recorded in `ledger/katas/factory-method/`: floor control
`killed`, agreement `survived` (scoped, coverage-only), totality
`survived` (scoped, coverage-only), residual gaps `killed` (named missing
predicates), role floor control `killed`, role-annotated fallback
exclusion `survived` (scoped: authored role classification).
