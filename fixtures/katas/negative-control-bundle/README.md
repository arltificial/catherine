# Kata slice — negative-control bundle (pre-registration)

Committed **before** the probe run (beads `catherine-lgl.9`). This slice is the
suite's honesty gate: a pattern whose lens the surface cannot express must
**not** produce fabricated findings. Nothing in the pre-registration section
may be edited after the run: a correction is a superseding ledger entry.

## Hypothesis

The completed weak-lens dispositions (`fixtures/katas/weak-lens-set/README.md`)
record the Flyweight identity gap: the kernel has no object-identity predicate,
so a duplicated flyweight row with equal intrinsic value is invisible to
`catherine check`. This bundle turns that gap into an active negative control
and adds two further silence cases. Hypothesis: on an unexpressible lens the
kernel emits **zero** findings while a scripted conventional test still catches
the seeded defect; on a pattern-lookalike with a legitimately differing path
the correct model declares the intended difference and the kernel emits no
false violation. Every positive arm is expected **killed/null** with an empty
unique set; the near-floor control must self-negate while proving the
comparison apparatus can name a real violation exactly as the conventional arm
does. A fabricated finding is a failure of the bundle, never a detection.

## Fixtures and candidate lenses

Each arm has a clean fixture (the conventional contract holds) and one seeded
fault fixture.

| Arm | Clean fixture | Fault fixture | Candidate lens in kernel terms |
|---|---|---|---|
| control (cache floor) | `cache-control.ct.json` | `cache-control-fault-stale.ct.json` | `source∘current = cached`: one cache entry holds its source's current value |
| Flyweight | `flyweight-shared.ct.json` | `flyweight-shared-fault-duplicate.ct.json` | client→key→factory and client→flyweight name one shared instance; **handle and intrinsic values are equal data, but row identity has no predicate** |
| procedural CRUD | `crud-procedural.ct.json` | `crud-procedural-fault-stale-read.ct.json` | **none**: no pattern and no equations; the model is an operation snapshot |
| Decorator lookalike | `decorator-lookalike.ct.json` | `decorator-lookalike-fault-undiscounted-charge.ct.json` | `charged = list∘discount` (the declared transformation); a naive delegation lens `charged = list` over-reports |

The Flyweight fault duplicates the shared instance: `factory(k2)` still
resolves the canonical row `f1`, while client `c2` points at a second row `f2`
whose `handle` and `intrinsic` values equal `f1`'s. Both expressible value
equations pass; only row identity differs, and the conventional identity test
catches it.

The CRUD fixture declares no equations at all. Its seeded fault is a stale
read-after-write (`read_back(q2)` returns `r1` instead of the created `r2`);
the kernel has nothing to evaluate and must say nothing.

The Decorator lookalike has two products whose `charged` price legitimately
differs from `list` because a `discount` transformation is applied. The
correct model declares that transformation; a naive delegation lens would flag
both products. The seeded fault leaves `p1` legitimate and drifts `p2`; the
kernel must emit the true `Product#1` violation and **never** a false
`Product#0` violation.

## Pre-registered element vocabulary

Detection is arithmetic, not judgement: an **element** is the named fault
witness both arms may identify. `unique = probe_elements − baseline_elements`.

| Fault | Element |
|---|---|
| control stale cache | `cache:source.current!=cached@Cache#0` |
| Flyweight duplicate instance | `flyweight:duplicate-instance@Client#1` |
| CRUD stale read | `crud:read_back!=created@Request#1` |
| lookalike undiscounted charge | `lookalike:charged!=list.discount@Product#1` |

The naive delegation lens's would-be elements (not detections) are
`lookalike:charged!=list@Product#0` and `lookalike:charged!=list@Product#1`.

## Fault matrix (pre-registered)

| Fault | Lens expressible | Kernel finding expected | Conventional catch expected | Unique expected | Verdict expected |
|---|---|---|---|---|---|
| control stale cache | yes (coherence equation) | exactly 1 `violation` `eq0@row0` | yes | 0 | `killed` (control self-negates) |
| Flyweight duplicate instance | **no** (identity predicate absent) | 0 | yes (row identity) | 0 | `killed` (null; gap recorded) |
| CRUD stale read | **no** (no pattern, no equations) | 0 | yes (read-after-write) | 0 | `killed` (null; gap recorded) |
| lookalike undiscounted charge | yes (declared transformation; the naive claim is the lens that over-reports) | exactly 1 `violation` `eq0@row1`; **no** `eq0@row0` | yes (declared transformation), naive lens would over-report `Product#0` | 0 | `killed` (parity; no false violation) |

## Near-floor control

`cache-control-fault-stale.ct.json` has one cache entry, one source, and one
stale cached value. The scripted conventional test checks the single
coherence equation, so it catches everything the kernel can catch. The
kernel's finding must be exactly the conventional one; unique must be empty
and the control's verdict `killed`.

## Baseline protocol (frozen before the run)

`prototype/negative_control_bundle_baseline.py` is the conventional arm: a
scripted per-fixture test reading the same committed fixtures, never calling
`catherine`. It checks:

- control: every cache's `cached` equals `current(source(cache))`;
- Flyweight: every client's `flyweight` is **the same row** as
  `factory(key(client))` — identity, not value equality;
- CRUD: every request's `read_back` equals `created`, and the created record's
  `stored_in` equals the request's `target`;
- lookalike: every product's `charged` equals `discount(list(product))`, and
  separately computes the naive `charged = list` set, asserting it has the two
  pre-registered rows on both the clean and fault fixtures (the legitimate
  difference a naive lens would over-report).

It runs on every clean fixture (expected: no detections) and every fault
fixture (expected: exactly the element above), and `--assert` fails loudly on
any mismatch. `--hash` prints the script sha256 for the ledger arm.

## Probe arm

The real `catherine` binary, no kernel changes and no new CLI:
`cargo run -q -p catherine-cli -- check <fixture> --json`. Findings are
recorded verbatim. The probe arm is expected to emit one `violation` for the
control and for the lookalike's true drift, **zero** findings for the
Flyweight duplicate and for the CRUD snapshot, and **never** the lookalike's
false `Product#0` violation.

## Verdict procedure (frozen)

1. Control first: empty unique ⇒ `killed`. A non-empty control unique set is a
   falsifier and the run is invalid.
2. Positive with the lens unexpressible at fixture level (no equation can name
   the claim) and zero probe findings ⇒ `killed`, with the expressibility gap
   recorded; the null result is the deliverable.
3. Positive with an expressible lens: unique empty ⇒ `killed` (parity); unique
   non-empty with an executed baseline arm ⇒ `survived`.
4. No verdict is forced upward: `survived` is not a goal, and unique is
   non-empty only if a real detection is demonstrated.
5. A probe finding outside the pre-registered set is a fabricated finding: it
   fails the bundle, and the run is invalid rather than rounded up.

## Kill condition / falsifiers

- The control's unique set is non-empty or its verdict is not `killed` — the
  run is invalid; the control cannot self-negate means the comparison is void.
- Any probe finding outside the pre-registered set — in particular any
  lookalike `eq0@row0` finding, which is a false violation on the declared
  legitimate difference. A fabricated finding fails the bundle.
- Baseline `--assert` mismatch — the conventional arm is wrong or a fixture
  changed; re-pin only with a superseding entry.
- A positive unique set that is non-empty because the baseline was weakened
  rather than because the kernel added information — the slice is refuted as a
  comparison, not promoted to `survived`.

## Execution

```
python3 prototype/negative_control_bundle_baseline.py --hash
python3 prototype/negative_control_bundle_baseline.py --assert
cargo run -q -p catherine-cli -- check fixtures/katas/negative-control-bundle/<fixture>.ct.json --json
cargo test -p catherine-ledger --test negative_control_bundle_kata
cargo run -q -p catherine-ledger --bin ledger-check -- ledger/katas/negative-control-bundle
```

## Observed results (appended after the executed run)

The pre-registered text above is unchanged. Observed probe findings are the
real CLI's (`catherine check --json`), recorded verbatim in the ledger.
| Fixture | Lens expressible | Kernel findings emitted | Conventional catch | Unique | Verdict |
|---|---|---|---|---|---|
| `cache-control.ct.json` (clean) | yes | 0 | none (contract holds) | 0 | clean |
| `cache-control-fault-stale.ct.json` | yes (coherence equation) | exactly 1 `violation` `eq0@row0` (`lhs=0`, `rhs=1`) | `cache:source.current!=cached@Cache#0` | 0 | `killed` — control self-negates |
| `flyweight-shared.ct.json` (clean) | no | 0 | none (contract holds) | 0 | clean |
| `flyweight-shared-fault-duplicate.ct.json` | **no** (identity predicate absent) | **0** | `flyweight:duplicate-instance@Client#1` (row identity) | 0 | `killed` — null result, gap recorded |
| `crud-procedural.ct.json` (clean) | no | 0 | none (contract holds) | 0 | clean |
| `crud-procedural-fault-stale-read.ct.json` | **no** (no pattern, no equations) | **0** | `crud:read_back!=created@Request#1` | 0 | `killed` — null result, gap recorded |
| `decorator-lookalike.ct.json` (clean) | yes | 0 | none; a naive `charged = list` lens would over-report `Product#0` and `Product#1` | 0 | clean |
| `decorator-lookalike-fault-undiscounted-charge.ct.json` | yes (declared transformation) | exactly 1 `violation` `eq0@row1` (`lhs=1`, `rhs=0`); **no** `eq0@row0` | `lookalike:charged!=list.discount@Product#1`; the naive lens would add the false `Product#0` | 0 | `killed` — parity, no false violation |

Unique detections: **0 for every arm**. The bundle records silence where the
lens is unexpressible and a true-only finding where the lookalike's
legitimate difference is declared; it never claims a detection-rate win.

Recorded gaps (also in each ledger entry):

- Flyweight: row identity / share count is data in this model (`handle` and
  `intrinsic` values are equal on the duplicate), so both expressible value
  equations pass and the duplicate is invisible to `check` (0 findings); the
  conventional row-identity test catches it.
- CRUD: read-after-write consistency is procedural and has no path-equation
  form; with no equations declared, `check` has nothing to evaluate, and the
  conventional CRUD test catches the stale read.
- Lookalike: the kernel cannot distinguish an intended difference from a
  defect except through declared arrows/equations; `check` has no
  underconstraint detector, so a wrongly omitted equation would be invisible.
- Control: only the one declared cache-coherence equality is checked;
  population, eviction, expiry, and concurrent refresh are unmodeled.
