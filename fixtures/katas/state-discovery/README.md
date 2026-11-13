# Kata — state-discovery (pre-registration)

Committed **before** the recorded run (`catherine-lgl.4`, P2 pattern
kata). Nothing in the pre-registration sections is edited after the run:
a correction is a superseding ledger entry.

## Hypothesis

A legacy state machine modelled as a `.ct.json` schema — states, events,
per-event transition tables, and declared (state × event) cells — exposes
two materially different products:

1. **Executed findings** where the model's declared equations and total
   tables disagree with the instance: a declared cell with no
   transition (`dangling`), a transition with no target (`dangling`), a
   documented target that drifts from the transition's target
   (`violation`), and a transition naming an undeclared state (an
   executed construction rejection, not a finding).
2. **Discovery questions** the kernel raises but never settles: the
   `underconstraint` arm asks whether same-endpoint schema paths
   (`from`/`to`; `cell_state`/`cell_target`) should agree, labeled
   `hypothesized`.

The claim under test is that the executed findings are named at the
element a conventional transition-table sample would miss, and that the
hypothesized questions are recorded as questions. The kata does **not**
claim a detection-rate win: the near-floor control is expected to kill
the mechanism at the two-state floor, and the unreachable-state question
is expected to be a null result.

## Pattern-equivalence mapping (State pattern ≅ state shape)

| State-pattern element | `.ct.json` encoding |
|---|---|
| state vocabulary | object `State` (dense rows = states) |
| event vocabulary | object `Event` (dense rows = declared events) |
| transition table row | object `Transition`, arrows `from : Transition → State`, `on : Transition → Event`, `to : Transition → State` |
| declared (state, event) cell | object `Cell`, arrows `cell_state`, `cell_event`, `cell_target : Cell → State`, `cell_transition : Cell → Transition` |
| "the table agrees with the target" | equation `cell_transition ; to = cell_target` (`accepted`) |
| "the table agrees with the source/event" | equations `cell_transition ; from = cell_state`, `cell_transition ; on = cell_event` (`accepted`) |
| "every declared cell has an exit" | a `Cell` row whose `cell_transition` cell is `null` — the kernel's dangling surface |
| "which parallel paths should agree?" | `catherine underconstraint --max-path-length 1`, `hypothesized` |

The kernel has no existential/coverage equation form. "Every reachable
state has an outgoing transition for every declared event" is therefore
encoded as declared `Cell` rows plus total instance vectors; an unfilled
declared cell is an executed `dangling` finding, not a universal check.
This is a named expressibility gap, not a silent substitution.

## Fault matrix (pre-registered)

Element vocabulary: one element is one declared cell, spelled
`cell:<State>/<Event>`. The baseline identifies an element only when one
of its pre-registered sample assertions fails on that exact cell.

| Entry | Fixture | Injected fault | Kernel surface (predicted) |
|---|---|---|---|
| `near-floor` (control) | `near-floor.ct.json` + two fault variants | two-state/two-event machine; retarget `On/stop`; drop `Off/start`'s transition | `violation` eq2 row `c_on_stop`; `dangling cell_transition@c_off_start` |
| `dangling-target` | `dangling-target.ct.json` | `t_paused_stop.to = null` (declared transition, no target) | `dangling to@Transition#2` |
| `missing-exit` | `missing-exit.ct.json` | declared cell `Stopped/stop` with `cell_transition = null` (no exit) | `dangling cell_transition@Cell#6` + two `underconstraint` questions |
| `drift-and-undeclared` | `drift-target.ct.json`; `unknown-target.ct.json` | cell target drifts from transition target; a transition names `Ghost` | `violation` eq2 row `c_playing_pause`; parse rejection `syntax` naming `t_playing_stop` |
| `unreachable` | `unreachable.ct.json` | `Buffering` has an outgoing `ready` edge but no incoming transition | `check` emits nothing; two `underconstraint` schema questions; reachability is an observation, not a finding |

## Near-floor control

The control is a two-state (`Off`, `On`), two-event (`start`, `stop`)
machine with four declared cells. The conventional sample covers all
four cells, so both injected faults are caught by the baseline; the
probe's unique set must be **empty** and the control verdict `killed`.
If the control survives, the kata is measuring test coverage, not
modeling.

## Conventional baseline protocol (frozen before the run)

`prototype/state_discovery_baseline.py` asserts, for each cell in the
pre-registered `sample` of `fixtures/katas/state-discovery/faults.json`:
the cell is declared; its `cell_transition` resolves; the transition's
`to` resolves to a declared state; `from`/`on`/`to` agree with
`cell_state`/`cell_event`/`cell_target`. Cells outside the sample are
never inspected. The script also executes the probe arm and the
`underconstraint` arm, maps witnesses to `cell:<State>/<Event>`
elements, and compares everything to the frozen expectations. `--assert`
exits 1 on any drift; `--hash` prints the script sha256 (pinned in the
ledger's `comparison.baseline.artifact`).

The baseline arm runs **before** the probe arm inside the script; the
ledger records the script hash and the executed mode.

## Comparison rule

`unique = executed probe elements − baseline-identified elements`, with
these fixed rules:

- a `dangling` witness `(arrow, row)` resolves to the cell named by its
  `Cell` row, or to the cell whose `cell_transition` names the
  `Transition` row (for `to`/`from`/`on`);
- a `violation` witness names its equation and row; all three accepted
  equations are `Cell`-domain, so the row is a cell;
- a hypothesized `underconstraint` finding is a discovery question and is
  **never** counted as a detection;
- an executed construction rejection is recorded in `run.probes` and
  never counted in `unique_detections` (it is a refusal, not a finding);
- the unreachable-state observation is recorded, not counted.

## Pre-registered predictions (hypotheses, not results)

| Entry | Baseline detections | Probe elements | Unique | Predicted verdict |
|---|---|---|---|---|
| `near-floor` | both variant faults | both cells | ∅ | `killed` |
| `dangling-target` | ∅ (sample omits `Paused/stop`) | `cell:Paused/stop` | 1 | `survived` |
| `missing-exit` | ∅ (sample omits `Stopped/stop`) | `cell:Stopped/stop` | 1 | `survived` |
| `drift-and-undeclared` | ∅ (sample omits `Playing/pause`/`stop`) | `cell:Playing/pause` | 1 | `survived` |
| `unreachable` | ∅ (cell assertions pass) | ∅ | ∅ | `killed` |

Discovery questions predicted on the four-state schema, verbatim:
`parallel_paths:cell_state=cell_target` and `parallel_paths:from=to`,
both `hypothesized`, question text `should these agree?`.

## Kill condition

- The control must self-negate (empty unique, verdict `killed`).
- A positive entry survives only with a non-empty unique set derived from
  **executed** evidence and an executed baseline arm. It is killed if the
  probe arm adds nothing outside the sampled cells.
- If a hypothesized question is the only "detection", the entry is
  `inconclusive` at best; the hypothesis is never upgraded to an executed
  claim.
- If reachability cannot be expressed with existing kernel surfaces, it
  is recorded as a gap/null result rather than fabricated as a finding.

## Falsifiers

- Any probe finding outside the pre-registered element set.
- Any baseline detection outside the frozen sample.
- A different `underconstraint` hypothesis set.
- A fixture edit after the run: the model hash pin in
  `crates/catherine-ledger/tests/state_discovery_kata.rs` fails, and the
  correction must be a superseding ledger entry.

## Observed results (recorded after the run)

**Baseline arm** (`python3 prototype/state_discovery_baseline.py
--assert`, exit 0): sha256
`7d98b54bb65ea4e4c00962fc1d0e3e57ce19acb134f5d767b1177badd11fec3e`,
5 entries, 3 unique elements. Observed detections:

| Fixture | Baseline detection | Probe element | Unique |
|---|---|---|---|
| `near-floor.ct.json` | — | — | — |
| `near-floor-fault-retarget.ct.json` | `sample:On/stop:target-mismatch` | `cell:On/stop` | ∅ |
| `near-floor-fault-missing-exit.ct.json` | `sample:Off/start:transition-missing` | `cell:Off/start` | ∅ |
| `dangling-target.ct.json` | — | `cell:Paused/stop` | 1 |
| `missing-exit.ct.json` | — | `cell:Stopped/stop` | 1 |
| `drift-target.ct.json` | — | `cell:Playing/pause` | 1 |
| `unknown-target.ct.json` | — | construction: `cell:Playing/stop` | not counted |
| `unreachable.ct.json` | — | — | ∅ |

**Probe arm, executed findings verbatim.** Labels below are what the
kernel emitted, not assignments:

- control retarget:
  `{"label":"executed","kind":"violation","witness":{"equation":2,"row":3,"lhs":1,"rhs":0}}`
- control missing exit:
  `{"label":"executed","kind":"dangling","witness":{"arrow":"cell_transition","src":0}}`
- dangling target:
  `{"label":"executed","kind":"dangling","witness":{"arrow":"to","src":2}}`
- missing exit:
  `{"label":"executed","kind":"dangling","witness":{"arrow":"cell_transition","src":6}}`
- drift:
  `{"label":"executed","kind":"violation","witness":{"equation":2,"row":3,"lhs":1,"rhs":3}}`
- undeclared state (`unknown-target.ct.json`): exit 1, envelope
  `code: syntax`, message
  ``table `to` maps `t_playing_stop` to `Ghost`, which is not a row of `State` (line 70, column 27)``
  — recorded as a construction probe, not a finding.
- unreachable (`unreachable.ct.json`): `check` emitted `[]`; the
  reachability observation is `["Buffering"]` unreachable from `Idle`.

**Discovery questions (hypothesized), verbatim, on every four-state
fixture** — `catherine underconstraint --max-path-length 1 --json`,
exit 6:

```json
{"label":"hypothesized","kind":"underconstraint","witness":{"case":"parallel_paths","detail":{"lhs":["cell_state"],"rhs":["cell_target"]},"question":"should these agree?"}}
{"label":"hypothesized","kind":"underconstraint","witness":{"case":"parallel_paths","detail":{"lhs":["from"],"rhs":["to"]},"question":"should these agree?"}}
```

These are schema-level questions. They name no cell and detect no fault;
they were recorded in `run.findings` and `comparison.fired` with the
`hypothesized` label and excluded from every `unique_detections` set.

**Ledger.** `ledger-check` exits 0: `ok: 5 entries (5 kata), rollup
verdict: (no rollup)`. Verdicts: control `killed`; `dangling-target`,
`missing-exit`, `drift-and-undeclared` `survived`; `unreachable`
`killed`. The observed results match the pre-registered predictions for
all five entries; no discrepancy was found, and the reachability claim
remains a recorded gap/null result, never a finding.