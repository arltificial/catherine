# Kata — mediator (pre-registration)

Committed **before** the recorded run (beads `catherine-r5k.5`). Nothing in
the pre-registration sections is edited after the run: a correction is a
superseding ledger entry.

## Hypothesis

A mediator is a **star**: colleagues are attached to one hub, and the hub
dispatches their interactions. The disposition row in
`docs/ref/pattern-kata-dispositions.md` records that "star topology wiring
has no equation with a truth value" and that "a discovery question is
plausible but unvalidated". This kata tests that claim on two surfaces:

1. **The coherence law** (an equation with a truth value): for every
   interaction, the hub that dispatches it is the hub each participant is
   attached to —

   - `dispatched_by = listener ; hub` (parsed index 0)
   - `dispatched_by = speaker ; hub` (parsed index 1)

   Both sides of each equation are `Interaction → Mediator` paths, so the
   law is expressible. Its only fault class is a **second center**: a
   second `Mediator` row exists and one party or one interaction is wired
   to it. With a single declared mediator the equations are forced by
   range alone and detect nothing.

2. **The exclusivity half as a discovery question** (no truth value):
   participants of one interaction should share one hub
   (`listener ; hub = speaker ; hub`), and a colleague should have no
   direct colleague-to-colleague edge at all. The first is raised by
   `catherine underconstraint` as a `hypothesized` parallel-path question;
   the second has no surface whatsoever (absence is not a path).

The claim under test is that the coherence equation finds a second-center
fault that a conventional per-arrow suite cannot see (the suite never
composes two arrows), and that the discovery-only candidate does **not**
survive as a law: a question carries no witness row and no verdict.

## Pattern-equivalence mapping (Mediator ≅ star topology)

| Mediator element | `.ct.json` encoding |
|---|---|
| colleague | object `Colleague` (dense rows = participants) |
| mediator | object `Mediator` (rows = hubs; a second row is the migration fault) |
| colleague's only outbound link | arrow `hub : Colleague → Mediator` — the schema has no Colleague→Colleague arrow |
| mediated interaction | object `Interaction`, arrows `speaker : Interaction → Colleague`, `listener : Interaction → Colleague` |
| dispatch through the hub | arrow `dispatched_by : Interaction → Mediator` |
| "the hub that dispatches is the hub the parties hold" | equations `dispatched_by = listener ; hub` and `dispatched_by = speaker ; hub` (`accepted`) |
| "one participant is not wired to a second center" | a null or drifted hub cell — the kernel's dangling/violation surface |
| "should both participants share one hub?" | `catherine underconstraint --max-path-length 2`, `hypothesized` |
| "there is no direct colleague-to-colleague edge" | nothing: the schema simply has no such arrow; absence has no equation |

The kernel has no universal/existential form and no negative-arrow form.
"Colleagues may not speak directly" is therefore a **schema-level fact**
(the arrow does not exist), not a checkable law; this is a named
expressibility gap, not a silent substitution.

## Fault matrix (pre-registered)

Element vocabulary: one element is one instance row named by a detection,
spelled `colleague:<name>` or `interaction:<name>`. The baseline
identifies an element only when one of its pre-registered per-row
assertions fails on that row.

| Entry | Fixture | Injected fault | Kernel surface (predicted) |
|---|---|---|---|
| `near-floor` (control) | `near-floor.ct.json` + two fault variants | two-colleague/one-interaction floor, one hub; a bypass (`dispatched_by = null`) and a documented-route swap | `dangling dispatched_by@Interaction#0`; route swap: `check` emits nothing |
| `party-second-center` | `party-second-center.ct.json` | `bob.hub = hub_legacy` while every interaction is dispatched by `hub_main` | `violation eq0@row0` (i_ab, listener bob), `violation eq1@row1` (i_bc, speaker bob) |
| `dispatch-second-center` | `dispatch-second-center.ct.json` | `i_ca.dispatched_by = hub_legacy` while both parties hold `hub_main` | `violation eq0@row2` and `eq1@row2` (i_ca) |
| `bypass` | `bypass.ct.json` | `i_bc.dispatched_by = null` (no hub) | `dangling dispatched_by@Interaction#1` |
| `hub-exclusivity-question` | `mediator.ct.json` | none: the clean one-hub model | `check` silent; two `underconstraint` questions, including `listener ; hub = speaker ; hub` |

## Near-floor control

The control is one hub (`hub_main`), two colleagues, one interaction. With
a single `Mediator` row, every `dispatched_by` and `hub` cell can only be
`hub_main`, so both accepted equations are tautologies: the coherence law
has **no fault class at this floor**. The two injected faults are the ones
a per-arrow suite checks — a bypass (totality) and a documented-route swap
(spec fidelity). Both are caught by the baseline; the probe catches the
bypass at the same element and is silent on the swap. The probe's unique
set must be **empty** and the control verdict `killed`. If the control
survives, the kata is measuring test coverage, not modeling.

The route-swap variant is a renaming of the clean control under the
schema's `speaker`/`listener` symmetry, so its **model hash equals the
clean control's** (`sha256:8103ca8c…`): the fault lives in the documented
route, not in the model's structure. The baseline catches it because its
documented-route assertion is external spec data; the kernel does not and
cannot.

## Conventional baseline protocol (frozen before the run)

`prototype/mediator_baseline.py` asserts, for **every row** of each
fixture:

1. every colleague's `hub` resolves to a declared `Mediator` row;
2. every interaction's `speaker` and `listener` resolve to declared
   `Colleague` rows and are distinct;
3. every interaction's `dispatched_by` resolves to a declared `Mediator`
   row;
4. every interaction's `(speaker, listener)` pair equals the
   pre-registered documented route (`fixtures/katas/mediator/faults.json`
   → `documented_routes`).

It deliberately does **not** assert any composition of two arrows
(`speaker ; hub`, `listener ; hub`) and does not assert a specific
mediator row: composing arrows is the modeling act under test, and a
second declared mediator row is exactly the migration fault. It executes
the probe arm too (`catherine check --json`, plus
`catherine underconstraint --max-path-length 2 --json` for the discovery
entry), maps both outputs to elements, and compares everything to the
frozen expectations. `--assert` exits 1 on any drift; `--hash` prints the
script sha256 (pinned in the ledger's `comparison.baseline.artifact`).

## Comparison rule

`unique = executed probe elements − baseline-identified elements`, with
these fixed rules:

- a `dangling` witness `(arrow, row)` resolves through the arrow's source
  object (here always `Interaction`) to `interaction:<name>`;
- a `violation` witness names its parsed equation index and row; all
  accepted equations are `Interaction`-sourced, so the row is an
  interaction;
- a hypothesized `underconstraint` finding is a discovery question and is
  **never** counted as a detection;
- the eq0/eq1 split is the parser's equation order, not declaration
  order: `eq0` is the `listener` equation, `eq1` the `speaker` equation
  (verified by a scratch probe with only `carol` on a second center: as
  listener she fires `eq0@row1`, as speaker `eq1@row2`).

## Pre-registered predictions (hypotheses, not results)

| Entry | Baseline detections | Probe elements | Unique | Predicted verdict |
|---|---|---|---|---|
| `near-floor` (clean) | ∅ | ∅ | ∅ | `killed` |
| `near-floor` (bypass) | `interaction:i_ab:dispatch-unresolved` | `interaction:i_ab` | ∅ | `killed` |
| `near-floor` (route swap) | `interaction:i_ab:route-mismatch` | ∅ | ∅ | `killed` |
| `party-second-center` | ∅ | `interaction:i_ab`, `interaction:i_bc` | 2 | `survived` |
| `dispatch-second-center` | ∅ | `interaction:i_ca` | 1 | `survived` |
| `bypass` | `interaction:i_bc:dispatch-unresolved` | `interaction:i_bc` | ∅ | `killed` |
| `hub-exclusivity-question` | ∅ | ∅ | ∅ | `killed` |

Discovery questions predicted on the clean model, verbatim
(`--max-path-length 2`):

```json
{"label":"hypothesized","kind":"underconstraint","witness":{"case":"parallel_paths","detail":{"lhs":["listener"],"rhs":["speaker"]},"question":"should these agree?"}}
{"label":"hypothesized","kind":"underconstraint","witness":{"case":"parallel_paths","detail":{"lhs":["listener","hub"],"rhs":["speaker","hub"]},"question":"should these agree?"}}
```

The second is the **hub-exclusivity question**. It names the two composed
paths and no row; it is raised on the one-hub clean model where the answer
is forced by range, so it is schema-level and instance-blind.

## Kill condition

- The control must self-negate (empty unique, verdict `killed`).
- A positive entry survives only with a non-empty unique set derived from
  **executed** evidence and an executed baseline arm.
- If a hypothesized question is the only "product", the entry is `killed`
  under the recorded rule: a question is not a detection. The kill is a
  recorded observation, not a fabrication.
- If exclusivity-as-absence cannot be expressed with existing kernel
  surfaces, it is recorded as a named gap.

## Falsifiers

- Any probe finding outside the pre-registered element set.
- Any baseline detection outside the frozen expectations.
- A different `underconstraint` question set.
- A fixture edit after the run: a correction must be a superseding ledger
  entry.

## Observed results (recorded after the run)

**Baseline arm** (`python3 prototype/mediator_baseline.py --assert`,
exit 0): sha256
`863b8c04a045b7571242b172540e0ba172aece8df9071ba37dd0263460b456d3`,
5 entries, 3 unique elements. The observed per-run table matches the
pre-registered predictions row for row (see the printed report).

**Probe arm, executed findings verbatim** (labels are what the kernel
emitted):

- clean model (`mediator.ct.json`): exit 0, `findings: []`.
- control bypass: exit 4,
  `{"label":"executed","kind":"dangling","witness":{"arrow":"dispatched_by","src":0}}`
- control route swap: exit 0, `findings: []` (the probe is silent; the
  baseline's documented-route assertion catches the swap).
- `party-second-center`: exit 3,
  `{"label":"executed","kind":"violation","witness":{"equation":0,"row":0,"lhs":1,"rhs":0}}`,
  `{"label":"executed","kind":"violation","witness":{"equation":1,"row":1,"lhs":1,"rhs":0}}`
  — the listener equation fires on `i_ab` (bob listens), the speaker
  equation on `i_bc` (bob speaks); every interaction of the second-center
  colleague is named.
- `dispatch-second-center`: exit 3,
  `{"label":"executed","kind":"violation","witness":{"equation":0,"row":2,"lhs":0,"rhs":1}}`,
  `{"label":"executed","kind":"violation","witness":{"equation":1,"row":2,"lhs":0,"rhs":1}}`
  — both equations on `i_ca`, the interaction dispatched by the second
  center while both parties hold the main hub.
- `bypass`: exit 4,
  `{"label":"executed","kind":"dangling","witness":{"arrow":"dispatched_by","src":1}}`.
- `hub-exclusivity-question`: `check` exit 0, `findings: []`;
  `underconstraint --max-path-length 2` exit 6 with the two hypothesized
  findings quoted above.

**Model hashes** (`catherine_core::hash::model_hash`):

| Fixture | Model hash |
|---|---|
| `mediator.ct.json` | `sha256:69d3c894a86268f722f8dc3c1c79dbdc109b42976bf2e0204b17e68551387d7f` |
| `near-floor.ct.json` | `sha256:8103ca8c35ca460868be13d87e8b08906f4b81064d27e00c59122590259f161b` |
| `near-floor-fault-route.ct.json` | `sha256:8103ca8c35ca460868be13d87e8b08906f4b81064d27e00c59122590259f161b` (same model: rename-isomorphic to the clean control) |
| `near-floor-fault-bypass.ct.json` | `sha256:e43d53b69a2c9ce79e486ef4639ad385d5591fe89134b592404a53006866741b` |
| `party-second-center.ct.json` | `sha256:8d9c424464aee1ea8f02c54109ab5b0bbb4d7bde5dfe3ca0653b9319d31673d2` |
| `dispatch-second-center.ct.json` | `sha256:c6911e666f59118cc90d4b406bbcd2114e478f33858aab8a2a6915dd85d5cd42` |
| `bypass.ct.json` | `sha256:44a639bfc4c490384bd4413f1493ffb5c37869b290ec8e308983e31fa07457fe` |

**Verdicts.** Control `killed`; `party-second-center` and
`dispatch-second-center` `survived` (scoped: the coherence law catches
second-center topology faults that per-arrow assertions cannot see);
`bypass` `killed` (parity: a per-arrow totality assertion catches a null
dispatch at the same element); `hub-exclusivity-question` `killed` (the
discovery question is observably present and correct, but a question is
not a law — it names no row, carries no truth value, and is raised even
where one hub makes it vacuously true).

**Gap record (the candidate, named honestly).** The candidate law was
"hub exclusivity as a discovery question". Outcome:

- *coherence* is a real law with a truth value (`dispatched_by =
  participant ; hub`) and survives, scoped, on second-center faults;
- *exclusivity* has no independent law: the kernel raises it only as a
  `hypothesized` parallel-path question (direct-only relation, see
  `crates/catherine-analysis/src/parallel.rs`), the question is
  instance-blind, and the exclusivity-as-absence half ("no direct
  colleague-to-colleague edge") has **no surface at all** — a missing
  negative-arrow/universal predicate, which is why the discovery-only
  candidate is killed rather than promoted.