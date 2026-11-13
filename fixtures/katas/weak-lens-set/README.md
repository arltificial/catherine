# Kata slice — weak lens set (pre-registration)

Committed **before** the probe run (beads `catherine-lgl.8`). This slice is
exploratory and negative-leaning by design: its purpose is to map where a
*weak* structural lens does **not** buy anything over a scripted conventional
test. Nothing in the pre-registration section may be edited after the run: a
correction is a superseding ledger entry.

## Hypothesis

`docs/ref/ploeh-grounding.md` §6 records four pattern↔abstraction
equivalences as HYPOTHESIZED (Composite≅monoid, CoR≅catamorphism,
Visitor≅sum type, State≅state shape). Five GoF patterns have **no proven
equivalence** in that section: Observer, Bridge, Flyweight, Command,
Prototype. Hypothesis: a candidate structural lens for each can be modeled
with the kernel's existing surfaces (arrows + path equations + totality),
but on a seeded fault the lens buys **zero unique detections** over a
scripted conventional test of the same pattern — parity or a null result is
the expected deliverable, never a detection-rate win.

## Fixtures and candidate lenses

Each pattern has a clean fixture (the lens holds) and one seeded-fault
fixture. The control is a near-floor single-subject/single-observer model.

| Pattern | Clean fixture | Fault fixture | Candidate lens in kernel terms |
|---|---|---|---|
| control (Observer floor) | `control-floor.ct.json` | `control-floor-fault-stale-seen.ct.json` | one `subject∘event = seen` coherence equation over one observer |
| Observer | `observer.ct.json` | `observer-fault-stale-delivery.ct.json` | `subject∘event = seen`: delivered event is the subject's current event |
| Bridge | `bridge.ct.json` | `bridge-fault-divergent-op.ct.json` | `impl∘impl_op = op`: the abstraction's result is the implementor's |
| Flyweight | `flyweight.ct.json` | `flyweight-fault-duplicate-instance.ct.json` | client→flyweight→intrinsic sharing arrows; **no equation** — the identity claim has no path-equation form |
| Command | `command.ct.json` | `command-fault-undo-drift.ct.json` | `after∘undo = before` and `before∘redo = after`: the undo/redo inverse laws |
| Prototype | `prototype.ct.json` | `prototype-fault-clone-drift.ct.json` | `clone∘intrinsic = source∘intrinsic`: the clone image preserves intrinsic value |

The Flyweight fixture deliberately carries **zero equations**. It models the
shared-instance arrows (`client→key→factory` and `client→flyweight`) but the
kernel has no object-identity predicate, so a duplicated flyweight row with
equal intrinsic value is invisible to `catherine check`. This is the slice's
recorded unexpressible-lens case, not a model defect.

## Pre-registered element vocabulary

Detection is arithmetic, not judgement: an **element** is the named fault
witness both arms may identify. `unique = probe_elements − baseline_elements`.

| Fault | Element |
|---|---|
| control stale seen | `observer-control:seen!=subject.event@Observer#0` |
| Observer stale delivery | `observer:seen!=subject.event@Observer#1` |
| Bridge divergent op | `bridge:op!=impl_op.impl@Abstraction#1` |
| Flyweight duplicate instance | `flyweight:duplicate-instance@Client#1` |
| Command undo drift | `command:after.undo!=before@Command#1` |
| Prototype clone drift | `prototype:clone.intrinsic!=source.intrinsic@Copy#0` |

## Fault matrix (pre-registered)

| Fault | Lens expressible | Probe finding expected | Conventional catch expected | Unique expected | Verdict expected |
|---|---|---|---|---|---|
| control stale seen | yes (coherence equation) | 1 violation `eq0@row0` | yes | 0 | `killed` (control self-negates) |
| Observer stale delivery | yes (weak coherence form) | 1 violation `eq0@row1` | yes | 0 | `killed` (parity) |
| Bridge divergent op | yes (delegation equation) | 1 violation `eq0@row1` | yes | 0 | `killed` (parity) |
| Flyweight duplicate instance | **no** (identity predicate absent) | 0 | yes | 0 | `killed` (null; gap recorded) |
| Command undo drift | yes (inverse law) | 1 violation `eq0@row1` | yes | 0 | `killed` (parity) |
| Prototype clone drift | yes (field-equality form) | 1 violation `eq0@row0` | yes | 0 | `killed` (parity) |

The Command equations are canonically ordered: `after∘undo = before` is
`eq0`, `before∘redo = after` is `eq1`. The seeded fault breaks only `eq0`.

## Near-floor control

`control-floor-fault-stale-seen.ct.json` has one subject, one observer, and
one stale `seen` cell. The scripted conventional test checks both
notification coverage (the single observer has a delivered event) and
payload coherence (it equals the subject's current event), so it catches
everything the lens can catch. Unique must be empty: the control
self-negates and its verdict is `killed`.

## Baseline protocol (frozen before the run)

`prototype/weak_lens_set_baseline.py` is the conventional arm: a scripted
per-pattern test reading the same committed fixtures, never calling
`catherine`. It checks:

- Observer: every observer's `seen` equals `event(subject(observer))`;
- Bridge: every abstraction's `op` equals `impl_op(impl(abstraction))`;
- Flyweight: every client's `flyweight` is **the same row** as
  `factory(key(client))` (identity, not value equality);
- Command: every command's `after.undo` equals `before` and
  `before.redo` equals `after`;
- Prototype: every copy's `clone.intrinsic` equals `source.intrinsic`.

It runs on every clean fixture (expected: no detections) and every fault
fixture (expected: exactly the element above), and `--assert` fails loudly
on any mismatch. `--hash` prints the script sha256 for the ledger arm.

## Probe arm

The real `catherine` binary, no kernel changes and no new CLI:
`cargo run -q -p catherine-cli -- check <fault fixture> --json`. Findings
are recorded verbatim. The probe arm is expected to emit one `violation` for
the five equation-bearing faults (including the control) and **zero**
findings for the Flyweight duplicate.

## Verdict procedure (frozen)

1. Control first: empty unique ⇒ `killed`; non-empty unique is a falsifier.
2. Positive with the lens unexpressible at fixture level (no equation can
   name the claim) and zero probe findings ⇒ `killed`, with the
   expressibility gap recorded; the null result is the deliverable.
3. Positive with an expressible lens: unique empty ⇒ `killed` (parity);
   unique non-empty with an executed baseline arm ⇒ `survived`.
4. No verdict is forced upward: `survived` is not a goal, and a detection
   rate win is never claimed from this slice.

## Kill condition / falsifiers

- Any probe finding outside the pre-registered set — the fixture or its
  expectation moved; record the discrepancy, never edit this matrix.
- Baseline `--assert` mismatch — the conventional arm is wrong or a fixture
  changed; re-pin only with a superseding entry.
- A positive unique set that is non-empty because the baseline was weakened
  rather than because the lens added information — the slice is refuted as
  a comparison, not promoted to `survived`.
- The control is non-empty or its verdict is not `killed` — the run is
  invalid; the control cannot self-negate means the comparison is void.

## Execution

```
python3 prototype/weak_lens_set_baseline.py --hash
python3 prototype/weak_lens_set_baseline.py --assert
cargo run -q -p catherine-cli -- check fixtures/katas/weak-lens-set/<fault>.ct.json --json
cargo test -p catherine-ledger --test weak_lens_set_kata
cargo run -q -p catherine-ledger --bin ledger-check -- ledger/katas/weak-lens-set
```

## Observed results (appended after the executed run)

The pre-registered text above is unchanged. Observed probe findings are the
real CLI's (`catherine check --json`), recorded verbatim in the ledger.

| Pattern | Lens expressible | Probe finding emitted | Conventional baseline catch | Unique | Disposition |
|---|---|---|---|---|---|
| control (single observer/subject) | yes (coherence equation) | yes, 1 `violation` `eq0@row0` | yes | 0 | `killed` — control self-negates |
| Observer | yes (weak coherence form) | yes, 1 `violation` `eq0@row1` | yes | 0 | `killed` — parity |
| Bridge | yes (delegation equation) | yes, 1 `violation` `eq0@row1` | yes | 0 | `killed` — parity |
| Flyweight | **no** (identity predicate absent) | **no** (0 findings) | yes | 0 | `killed` — null result, gap recorded |
| Command | yes (inverse-law form) | yes, 1 `violation` `eq0@row1` | yes | 0 | `killed` — parity |
| Prototype | yes (field-equality form) | yes, 1 `violation` `eq0@row0` | yes | 0 | `killed` — parity |

Unique detections: **0 for every pattern**. The slice records parity for the
four expressible weak lenses and a null result for the unexpressible
Flyweight identity claim. It does not claim a detection-rate win, and it
does not upgrade any pattern to a ploeh §6 equivalence: §6's proven-four
remain the only equivalences, and these five stay HYPOTHESIZED candidates.

Recorded gaps (also in each ledger entry):

- Observer: dynamically registered observers and notification totality are
  not expressible; the model names the observer rows and subject edges up
  front.
- Bridge: independent variation of the two hierarchies is not expressible;
  only the one declared delegation path is checked.
- Flyweight: object identity / share count has no kernel predicate; the
  duplicate row is invisible to `check` (0 findings), while the conventional
  identity test catches it.
- Command: history, queue order, and receiver dispatch are temporal and not
  expressible; the before/after states are pre-declared.
- Prototype: aliasing and mutation independence are not expressible by a
  static path equality over declared values.