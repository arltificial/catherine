# Kata — facade (call-trace law): pre-registration and recorded run

Committed **before** the recorded run (`beads catherine-r5k.3`, gap kata under
epic `catherine-r5k`). Nothing in the pre-registration sections is edited after
the run: a correction is a superseding ledger entry. The observed section at the
bottom was appended after the executed run.

## Provenance and what is being re-opened

`docs/ref/pattern-kata-dispositions.md` lists Facade as **disposed** — "convenience
delegation; no invariant" — with no executable law and no ploeh §6 series row.
That row is a gap statement, not a theorem, and this kata re-opens it with a
named law rather than with a new fixture in search of one.

The law is named first (next section). It is a **trace law**, not the wrapper
transparency law: a facade's defining structure is *one entry point fronting
several subsystem calls*, and the only truth-valued claim that structure makes
is that the implementation realizes the documented trace. The law is
expressible on the current kernel surface (parallel path equations plus one
declarative key); the facade-defining clauses that are **not** expressible are
named, with executed evidence, at the end.

## The law named first (pre-registered)

Objects `Module`, `Action`, `FacadeOp`, `SpecCall`, `Call`, `Outcome`. A
`SpecCall` row is one *documented* call (entry point, subsystem action); a `Call`
row is one *implemented* call (entry point, subsystem action, observed outcome,
and the documented call it realizes). The law has five clauses:

| Clause | Executable form | Parsed index |
|---|---|---|
| L1 call transparency | `call_action ; action_outcome = call_outcome` | 0 |
| L2 attribution (implementation side) | `call_spec ; spec_action = call_action` | 1 |
| L3 attribution (entry-point side) | `call_spec ; spec_facade = call_facade` | 2 |
| L4 completeness (action side) | `spec_impl ; call_action = spec_action` | 3 |
| L5 completeness (entry-point side) | `spec_impl ; call_facade = spec_facade` | 4 |
| L6 trace multiplicity | key on `Call` over `(call_facade, call_action)` | key 0 |

**Parsed index, not declaration order.** The parser sorts equations by
`(lhs, rhs, rationale, status)`, so the ledger's `violation:eq<n>` witnesses cite
the parsed index above. The key is the single declared key, index 0.

L1–L5 are path equations over one source→target pair each; they state
completeness and attribution *row-wise* by pairing every documented call with an
implementation (and back). L6 is a declarative key (`format_version` 1.3,
`add-declarative-keys`): injectivity of the `(entry point, action)` tuple over
`Call` rows, so an entry point making the same subsystem call twice is a
collision.

The law is **documentation-relative by construction**: it is enforceable only
when the model declares the documented call set (`SpecCall`). A facade model
with only implemented calls cannot state composition at all — the earlier
probe for this ticket built exactly that model, and every composition fault was
probe-silent. That is a modeling obligation of the law, recorded here rather
than hidden: the kernel checks the implementation against the *declared* trace,
not against an external specification.

## Pattern-equivalence mapping (Facade ≅ documented-trace realization)

| Facade-pattern element | `.ct.json` encoding |
|---|---|
| subsystem module | `Module` row |
| subsystem operation | `Action` row + `action_module`, `action_outcome` |
| facade entry point (the unified interface) | `FacadeOp` row + `facade_outcome` |
| one subsystem call the entry point makes | `Call` row + `call_facade`, `call_action`, `call_outcome` |
| the documented composition of an entry point | `SpecCall` row + `spec_facade`, `spec_action`; the entry point's call set is the fiber of `call_facade` |
| "the entry point does what it documents" | L2–L5 (completeness + attribution, both directions) |
| "it makes each subsystem call once" | L6 (key on the call pair) |
| "a call returns what the subsystem call returns" | L1 (per-call transparency) |

The facade's *distinguishing* claims that remain unstatable — aggregation of the
per-call outcomes into the entry point's answer, call order, and surface
reduction/coverage — are recorded in the expressibility section with executed
evidence, not transformed into a weaker surrogate.

## Fixtures

| Path | Role |
|---|---|
| `facade.ct.json` | the clean above-floor model: 4 modules, 7 actions, 4 entry points, 8 documented calls, 8 calls, 5 equations, 1 key; `check` clean, `redundant` empty |
| `fault-dropped-call.ct.json` | L4: documented call `f_checkout`/`a_inv_reserve` keeps its spec row; its call row is gone and `spec_impl` was left pointing at the neighbouring call |
| `fault-extra-call.ct.json` | L2/L3: an undocumented call (`f_checkout`/`a_cart_clear`) has no `call_spec` cell |
| `fault-attribution-drift.ct.json` | L2/L4: `c5_check_cap` performs `a_pay_refund` while its documented call says `a_pay_capture` (the outcomes are equal, so L1 stays silent) |
| `fault-duplicated-call.ct.json` | L6: a second `Call` row repeats `(f_checkout, a_pay_capture)`; all five equations hold |
| `fault-aggregate-drift.ct.json` | the entry point's answer (`o_failed`) disagrees with its call trace; no predicate reaches it |
| `fault-call-outcome-drift.ct.json` | L1: one call reports `o_failed` while its action reports `o_sent` |
| `floor/facade-floor.ct.json` | the near-floor control: 1 module, 1 action, 1 entry point, 1 documented call, 1 call, 5 equations, 1 key |
| `floor/fault-dropped-call.ct.json` | control C1: the documented call's call row and its `spec_impl` cell are dropped |
| `floor/fault-call-outcome-drift.ct.json` | control C2: the single call's outcome drifts |

Faults are edits to the *instance*, never to the schema: the equations and the
key are the law and are identical in every fixture.

## Fault matrix (pre-registered)

Elements: `call:<FacadeOp>/<Action>` (one documented or implemented call pair) or
`op:<FacadeOp>` (one entry point's answer).

| Entry | Fixture | Class | Injected change | Probe elements (predicted) | Baseline | Unique | Verdict |
|---|---|---|---|---|---|---|---|
| `near-floor` (control) | clean floor | — | none | ∅ | ∅ | ∅ | `killed` |
| `near-floor` (control) | `floor/fault-dropped-call` | structural | documented call loses its implementation and its link cell | `call:f_add_item/a_cart_add` (`dangling:spec_impl`) | catches | ∅ | `killed` |
| `near-floor` (control) | `floor/fault-call-outcome-drift` | behavioral | the one call's outcome drifts | `call:f_add_item/a_cart_add` (`violation:eq0`) | catches | ∅ | `killed` |
| `trace-completeness` | `fault-dropped-call` | structural | documented call's implementation deleted, link re-pointed | `call:f_checkout/a_inv_reserve` (`violation:eq3`) | catches | ∅ | `killed` |
| `undocumented-call` | `fault-extra-call` | structural | undocumented call with no `call_spec` cell | `call:f_checkout/a_cart_clear` (`dangling:call_spec`) | catches | ∅ | `killed` |
| `attribution-drift` | `fault-attribution-drift` | structural | call rerouted to another action, equal outcome | `call:f_checkout/a_pay_capture`, `call:f_checkout/a_pay_refund` (`violation:eq1`, `violation:eq3`) | catches both | ∅ | `killed` |
| `trace-multiplicity` | `fault-duplicated-call` | structural | duplicated call pair | `call:f_checkout/a_pay_capture` (`violation:key0`) | catches | ∅ | `killed` |
| `aggregate-drift` | `fault-aggregate-drift` | behavioral | the entry point's answer drifts from the trace | ∅ (probe silent) | catches | ∅ | `killed` (named gap) |
| `call-transparency` | `fault-call-outcome-drift` | behavioral | call outcome drifts from its action's outcome | `call:f_checkout/a_notify_receipt` (`violation:eq0`) | catches | ∅ | `killed` |

Pre-registered unique total: **0**. The conventional arm is deliberately
**exhaustive** over the four entry points and the eight documented pairs — no
sampling game. Parity is the expected outcome (E4): every fault the law can name,
a competent recorded-composition replay names too. What the kata adds is the
boundary: which facade clauses have no predicate, and the executed evidence that
they do not.

## Near-floor control

The floor is the smallest model that still carries all six clauses: one module,
one action, one entry point, one documented call, one call. Two faults are
injected — completion loss (C1) and outcome drift (C2). The conventional replay
is exhaustive at this size (one pair, one aggregate), so both faults are caught
by both arms and the control self-negates: `unique` is empty and the verdict is
`killed`. If it ever recorded a unique detection, the floor claim would be
refuted.

## Conventional baseline protocol (frozen before the run)

`prototype/facade_baseline.py` replays the **frozen documented contract** in
`faults.json` — never a read-back of the model's own `SpecCall` rows, which
would check the model against itself:

1. **composition** — for each sampled entry point, the observed multiset of
   `(entry point, action)` call pairs must equal the documented multiset
   (missing → a documented call never happened; duplicated → it happened twice;
   unexpected → an undocumented call);
2. **aggregate** — the entry point's `facade_outcome` must equal the documented
   aggregate;
3. **transparency** — for each sampled documented pair, the call's
   `call_outcome` must equal the named action's `action_outcome`.

The sample is frozen: all four above-floor entry points and all eight
documented pairs; the floor's single pair. Order is **not** asserted — the model
has no order structure to assert it against (see the gap list). The script runs
the baseline arm first, then the probe arm, maps both onto the element
vocabulary, and compares everything with the pre-registered expectations.
`--assert` exits 1 on any drift; `--hash` prints the script sha256 pinned in the
ledger.

## Comparison rule

`unique = executed probe elements − baseline-identified elements`, with these
fixed rules:

- an equation witness resolves through its source object: a `Call` row resolves
  to `call:<call_facade>/<call_action>`; a `SpecCall` row resolves to
  `call:<spec_facade>/<spec_action>`;
- a key witness resolves through the key's object rows (a duplicate pair
  resolves to the one pair it repeats);
- a dangling witness resolves through the arrow's source object and row;
- the probe never counts a hypothesized `underconstraint` finding as a
  detection, and this kata does not count `redundant` (empty on its clean model).

## Pre-registered predictions (hypotheses, not results)

Every expressible clause is predicted **parity**: the probe names the element the
exploratory fault lives on, the exhaustive conventional replay names the same
element, and `unique` is empty. The aggregate fault is predicted **probe-silent**
(no equation or key reaches `facade_outcome` from the call fiber), also with an
empty unique set, recorded with verdict `killed` and the fold named as an
expressibility gap.

## Kill condition (pre-registered)

- The control must self-negate (`unique` empty, verdict `killed`).
- A positive entry is `survived` only with a non-empty unique set from executed
  evidence and an executed baseline arm; an empty unique set is `killed`.
- A fault the probe cannot name is recorded as `killed` **with** the missing
  predicate named — never upgraded to a detection by adding a weaker surrogate
  law after the run.

## Falsifiers

- Any probe finding outside the pre-registered element set, or any baseline
  detection outside the frozen sample.
- A clean fixture that fires, or a `--assert` drift.
- A unique detection anywhere: the parity claim would fail.
- A probe finding on `fault-aggregate-drift`: the fold-gap claim would fail.
- An expressibility drift (a new arrow sourced at `FacadeOp`, or the forward
  closure reaching a `Call` row): the recorded surface boundary would be stale.

## Observed results (recorded after the run)

**Baseline arm** (`python3 prototype/facade_baseline.py --assert`, exit 0):
the recorded run executed the script as `sha256:08d96a74c3e2a3c05670cb30b183571a17405f1f0b7c96bc717244ef10b7c13b`;
7 entries, 1 probe-silent facade-defining run, **0 unique elements** across all
executed runs.

| Entry | Fixture | Baseline detection | Probe finding (verbatim) | Unique |
|---|---|---|---|---|
| `near-floor` | clean floor | — | — | ∅ |
| `near-floor` | `floor/fault-dropped-call` | `composition:f_add_item/a_cart_add:missing` | `{"label":"executed","kind":"dangling","witness":{"arrow":"spec_impl","src":0}}` | ∅ |
| `near-floor` | `floor/fault-call-outcome-drift` | `transparency:f_add_item/a_cart_add:o_declined!=o_ok` | `{"label":"executed","kind":"violation","witness":{"equation":0,"row":0,"lhs":1,"rhs":0}}` | ∅ |
| `trace-completeness` | `facade.ct.json` | — | — | ∅ |
| `trace-completeness` | `fault-dropped-call` | `composition:f_checkout/a_inv_reserve:missing` | `{"label":"executed","kind":"violation","witness":{"equation":3,"row":5,"lhs":5,"rhs":3}}` | ∅ |
| `undocumented-call` | `fault-extra-call` | `composition:f_checkout/a_cart_clear:unexpected` | `{"label":"executed","kind":"dangling","witness":{"arrow":"call_spec","src":8}}` | ∅ |
| `attribution-drift` | `fault-attribution-drift` | `composition:f_checkout/a_pay_capture:missing`, `composition:f_checkout/a_pay_refund:unexpected` | `{"label":"executed","kind":"violation","witness":{"equation":1,"row":4,"lhs":5,"rhs":6}}`, `{"label":"executed","kind":"violation","witness":{"equation":3,"row":4,"lhs":6,"rhs":5}}` | ∅ |
| `trace-multiplicity` | `fault-duplicated-call` | `composition:f_checkout/a_pay_capture:duplicated` | `{"label":"executed","kind":"violation","witness":{"key":0,"rows":[4,8]}}` | ∅ |
| `aggregate-drift` | `fault-aggregate-drift` | `aggregate:f_checkout:o_failed!=o_ok` | — (probe silent) | ∅ |
| `call-transparency` | `fault-call-outcome-drift` | `transparency:f_checkout/a_notify_receipt:o_failed!=o_sent` | `{"label":"executed","kind":"violation","witness":{"equation":0,"row":6,"lhs":3,"rhs":0}}` | ∅ |

Dense row ids cited above: `Call` rows sort as `c1_add` 0, `c2_cancel_ref` 1,
`c3_cancel_rel` 2, `c4_check_add` 3, `c5_check_cap` 4, `c6_check_res` 5,
`c7_check_rcpt` 6, `c8_refund` 7 (and `c9_check_gift` / `c9_check_cap_dup` 8 in
their fixtures); `SpecCall` rows sort the same way (`s6_check_res` is 5);
`Outcome` rows in the drift fixtures sort `o_failed` 0, `o_held` 1, `o_ok` 2,
`o_sent` 3; in the floor drift fixture `o_declined` 0, `o_ok` 1.

Both clean fixtures are silent (`check` exit 0, findings `[]`), and
`catherine redundant` on the clean above-floor model is empty. The recorded
element sets, baseline detections, and unique arithmetic are exactly the
pre-registered ones; no discrepancy was found.

**Discovery questions (hypothesized), verbatim, on every fixture** —
`catherine underconstraint --max-path-length 2 --json`, exit 6:

```json
{"label":"hypothesized","kind":"underconstraint","witness":{"case":"parallel_paths","detail":{"lhs":["call_action","action_outcome"],"rhs":["call_facade","facade_outcome"]},"question":"should these agree?"}}
{"label":"hypothesized","kind":"underconstraint","witness":{"case":"parallel_paths","detail":{"lhs":["call_facade","facade_outcome"],"rhs":["call_outcome"]},"question":"should these agree?"}}
{"label":"hypothesized","kind":"underconstraint","witness":{"case":"parallel_paths","detail":{"lhs":["spec_action","action_outcome"],"rhs":["spec_facade","facade_outcome"]},"question":"should these agree?"}}
{"label":"hypothesized","kind":"underconstraint","witness":{"case":"parallel_paths","detail":{"lhs":["spec_action","action_outcome"],"rhs":["spec_impl","call_outcome"]},"question":"should these agree?"}}
{"label":"hypothesized","kind":"underconstraint","witness":{"case":"parallel_paths","detail":{"lhs":["spec_facade","facade_outcome"],"rhs":["spec_impl","call_outcome"]},"question":"should these agree?"}}
```

These are schema-level questions, recorded verbatim and never counted as
detections. All five ask a *pointwise* variant of the aggregation question
("should a call's outcome agree with its entry point's outcome?"). None of them
is the aggregation law, and one of them would be **false** of a competent
facade: in the clean model `c6_check_res` observes `o_held` while its entry
point answers `o_ok`. Accepting the pointwise equation would forbid a facade
whose parts report intermediate states, so the honest model leaves the question
open — which is exactly the fold gap below.

**Expressibility observations (executed, `catherine query`).** On the clean
above-floor model with entry point `f_checkout` (`FacadeOp#2`):

- the only arrow sourced at `FacadeOp` is `facade_outcome`, so there is **no
  forward path from an entry point into its call fiber**;
- forward closure `--seed FacadeOp#2 --follow facade_outcome` yields exactly
  `FacadeOp#2 → Outcome#1` — the answer is a forward leaf;
- the fiber is reachable only backward: `--seed FacadeOp#2 --follow call_facade
  --direction backward` yields `Call#3, #4, #5, #6` (plus the seed).

The fiber is therefore observable as a *query result* but never relatable to the
aggregate by an equation: equations compose forward only, and the language has
no fold, no reverse composition, and no identity arrow.

### Named expressibility gaps (observed, never forced)

1. **Aggregate fold.** The entry point's answer is a fold over the fiber of
   `call_facade`. `fault-aggregate-drift.ct.json` sets `f_checkout` to
   `o_failed` while every call observes `o_ok`/`o_held`/`o_sent`; `check` exits 0
   with no findings, because `facade_outcome` is unreachable from the call rows
   (verified above). The conventional replay catches the drift
   (`aggregate:f_checkout:o_failed!=o_ok`). No weaker pointwise equation is
   substituted: the `underconstraint` questions above show why one would be
   wrong.
2. **Call order.** `Call` rows are an unordered set; dense row ids are
   name-sorted presentation, not invocation order. A reordered trace is
   unrepresentable, so no fixture can inject it and the baseline cannot assert it.
3. **Surface reduction / coverage.** "The facade exposes at most as many
   operations as the subsystem" and "every promised capability is covered" need a
   cardinality or existential predicate; equations and keys are row-wise and
   there is none. Facade is also allowed to expose a *subset*, so this clause is
   not an invariant of the pattern in the first place — it is named so the row is
   not read as licensing a coverage check it never ran.
4. **Spec aliasing.** Two `SpecCall` rows naming the same `Call` row satisfy
   L2–L5 simultaneously (the same action, the same entry point), and mutual
   inverse of `spec_impl`/`call_spec` needs an identity arrow. Duplicate
   *documentation* stays invisible; duplicate *implementation* is caught by L6.

### What this does and does not license

- It licenses a **structural** claim: with the documented trace declared, the
  facade's completeness, attribution, and call multiplicity are executable
  equations/keys, and they fire on the drifted elements (L2–L6 observed).
- It does **not** license a detection-rate claim: the conventional arm is
  exhaustive and every expressible fault is parity (unique ∅ everywhere).
- It does **not** close the disposed row on the aggregation axis: the entry
  point's answer has no predicate, and the executed evidence for that is
  `fault-aggregate-drift` being probe-silent while the conventional replay names
  the drift.

### Pinned identities

Baseline script: the recorded run executed
`sha256:08d96a74c3e2a3c05670cb30b183571a17405f1f0b7c96bc717244ef10b7c13b` (the
pin the ledger entries carry). After the recorded run, a gate-mandated
structural refactor decomposed `baseline_arm` into four helper functions (no
behaviour change); the committed script is now
`sha256:05e158dd28c8a9432bf223b0d52b3c47b96d4933aa83c7df8e07d0ff36ec26a9`.
The arm was re-executed on the refactored script: `--assert` exits 0 and its
report is byte-identical to the recorded one except the `baseline_sha256` field.
The sealed entries are not edited for this: their pin names the artifact that
actually ran, and a pin correction cannot be superseded without breaking the
validator's control-first rule (superseding the control removes the earlier
effective control the positive entries depend on). The refactor is recorded here
instead of being papered over.

Model hashes (kernel `model_hash` over the parsed model; pinned again in each
ledger entry):

| Fixture | Model hash |
|---|---|
| `facade.ct.json` | `sha256:f4f0b74456637f4a2b4d24a26bebb5fac25f39448910e7f4d2045ac569d5ae1f` |
| `fault-dropped-call.ct.json` | `sha256:a936b41fb3d589d13c0d0e7fd2e10de06e32e8fd0c584a9d0b9d122658899b03` |
| `fault-extra-call.ct.json` | `sha256:b8828e3af1db4e43d71883958503ddc490b36ccc3650fd84c322ce710c3992b8` |
| `fault-attribution-drift.ct.json` | `sha256:3d9887b8cde89d1ebe3f6d779902143f88345fe7992079e25a6eede012e8d9f0` |
| `fault-duplicated-call.ct.json` | `sha256:3ab0aa3ed7ef113e464110fa6d9e24287afce8fe38d31a574270a216eb6a4483` |
| `fault-aggregate-drift.ct.json` | `sha256:2b1047032942551a9cd1b7a6f1909804ef84e43de8c35944a1b265c50970193d` |
| `fault-call-outcome-drift.ct.json` | `sha256:b2abe1a1a8089a2cc1913395f1db49f30ee6fea2410b50aa1bafcffbe17a42cb` |
| `floor/facade-floor.ct.json` | `sha256:d398b3d3c16b3f06f7ec9ffc2ec23653dfff0f09a62b8b1ce73faace90bcf5d6` |
| `floor/fault-dropped-call.ct.json` | `sha256:07b383d674953efc3e7322f9a19068612fa4fd4dd520eabd19cbd611bd869908` |
| `floor/fault-call-outcome-drift.ct.json` | `sha256:2048d39fe3ac8938dcb03f8cc0844355b4f87510887f2a723fbbaa8f3d732db0` |

## Execution

```
python3 prototype/facade_baseline.py --assert        # baseline + probe arms, frozen expectations
python3 prototype/facade_baseline.py --hash          # the pin carried in the ledger
cargo run -q -p catherine-cli -- check fixtures/katas/facade/facade.ct.json --json
cargo run -q -p catherine-cli -- underconstraint fixtures/katas/facade/facade.ct.json --max-path-length 2 --json
cargo run -q -p catherine-cli -- query fixtures/katas/facade/facade.ct.json --seed FacadeOp#2 --follow call_facade --direction backward --json
cargo run -q -p catherine-ledger --bin ledger-check -- ledger/katas/facade
```