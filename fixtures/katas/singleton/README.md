# Kata — singleton (pre-registration)

Committed **before** the recorded run (`catherine-r5k.7`, P3 gap kata).
Nothing in the pre-registration sections is edited after the run: a
correction is a superseding ledger entry. This kata's disposition is
`unexpressible`; the kata tests whether that row can be upgraded.

## Law, named first

**Candidate law (tested, not assumed): exact cardinality / unique element.**

> The instance contains exactly one row of the declared singleton object
> `Engine`: `|rows(Engine)| = 1`, with every access path resolving to that
> one row.

The law has two halves and the kata tests them separately:

1. **Uniqueness/existence** — `|rows(Engine)| = 1`. This is an
   object-level cardinality claim over a row set.
2. **Shared reference** — for every client, `engine(c)` and `fallback(c)`
   are the *same* `Engine` row (row identity, not equal intrinsic values),
   and `engine(c) ; mode = seen(c)`.

The second half is expected to be expressible as a pointwise path equation
(`engine = fallback`, `engine ; mode = seen`). The first half is expected
to have **no** surface: the kernel's constraints are relative to rows that
already exist, and none of them counts or requires rows.

## What the current surface offers (read before the run)

| Surface | Form | Cardinality power |
|---|---|---|
| arrows | total vector per source row, `null` = dangling | a non-null accessor forces its target row to exist (typing), but never bounds the count |
| path equations | pointwise equality of two paths with equal endpoints | relative to existing rows only; cannot say "one" |
| declarative keys (format 1.3) | tuple injectivity over the object's rows; empty tuple rejected at parse | gives `≤ 1` *per equal tuple*, never existence and never `= 1` over the object |
| rule lens | gap/overlap/derivability over the declared finite input domain | domain coverage, not row count |
| `search` bounds | `max_rows` per object, default 0 | an exploration budget over synthetic instances; zero-size bounds are UNKNOWN, never a claim about the document |
| instance rows | declared data (an object with no rows parses cleanly) | data, not a constraint |

Predicted result: the shared-reference half runs (executed violations), the
cardinality half never fires, and the missing predicate is a **cardinality /
existence constraint over an object's row set** (`exactly_one(S)`,
`nonempty(S)`), which the format has no field, parser pass, or finding
witness for.

## Pattern-equivalence mapping (Singleton pattern ≅ unique element)

| Singleton-pattern element | `.ct.json` encoding |
|---|---|
| the class whose instance is unique | object `Engine` (rows = instances) |
| the private constructor / single allocation | no surface; instance rows are data |
| `getInstance()` | arrow `engine : Client → Engine` |
| a second access path (global registry, DI lookup) | arrow `fallback : Client → Engine` |
| "every caller gets the same object" | equation `engine = fallback` (`accepted`) |
| the unique instance's state | arrow `mode : Engine → Mode` |
| "the caller observes the instance's state" | equation `engine ; mode = seen` (`accepted`) |
| "only one instance exists" | **no encoding** — see the candidate law above |
| value-injectivity approximation of "no duplicate" | key `{Engine: [mode]}` (format 1.3) |

The key is deliberately included in two fixtures to measure how far the
approximation reaches: it catches two rows with *equal* intrinsic values,
and it passes two rows with *different* values. It is an injectivity claim,
never a cardinality claim.

## Element vocabulary (frozen)

An element is a named fault, not a witness shape: both arms must identify
the same element to count as a catch. Duplicate rows are one element even
when one arm names it by a key collision and the other by a row count.

| Element | Meaning |
|---|---|
| `singleton:engine-duplicate` | the instance has more than one `Engine` row |
| `singleton:engine-missing` | the instance has no `Engine` row |
| `singleton:engine!=fallback@Client#<i>` | client row `i`'s access paths resolve to different `Engine` rows |
| `singleton:accessor-dangling@Client#<i>` | client row `i`'s access path is `null` / does not resolve |

Probe-element mapping (frozen): an executed `violation` with an
equation witness maps to `singleton:engine!=fallback@Client#<row>`; an
executed `violation` with a key witness on key 0 (`Engine`,`mode`) maps to
`singleton:engine-duplicate`; an executed `dangling` witness on `engine`
or `fallback` maps to `singleton:accessor-dangling@Client#<src>`; an
executed construction rejection is recorded as a probe, never counted; a
hypothesized finding is never counted.

## Fault matrix (pre-registered)

| Entry | Fixtures | Injected fault | Baseline detections | Probe expectation | Unique | Predicted verdict |
|---|---|---|---|---|---|---|
| `control` (control) | `control.ct.json` (clean), `control-fault-disagreement.ct.json` | `c1.engine = e2` while `c1.fallback = e1` (a second access path escapes the singleton) | `engine-duplicate`, `engine!=fallback@Client#0` | `violation` eq0 row0 (lhs `e2`, rhs `e1`) | ∅ | `killed` |
| `duplicate-instance` | `duplicate-instance.ct.json` | two `Engine` rows with **distinct** modes, one per client; key `{Engine:[mode]}` accepted | `engine-duplicate` | none (the key is injective: the modes differ; the equations hold per client) | ∅ | `killed` (gap) |
| `equal-duplicate` | `equal-duplicate-instance.ct.json` | two `Engine` rows with the **same** mode; key `{Engine:[mode]}` accepted | `engine-duplicate` | `violation` key0 rows `[0,1]` | ∅ | `killed` (parity: the key catches equal values, not the count) |
| `missing-instance` | `missing-instance.ct.json`; `missing-instance-dangling.ct.json`; `empty-key.ct.json` (probe) | zero `Engine` rows, no clients (primary); zero `Engine` rows, one client with null accessors; a key with an empty arrow tuple | `engine-missing`; on the dangling variant also `accessor-dangling@Client#0` | none on the primary (0 findings); 2 `dangling` on the dangling variant; `invalid_key` parse rejection on the probe | ∅ | `killed` (gap + parity on the dangling variant) |

## Near-floor control

`control-fault-disagreement.ct.json` is the smallest instance that can
carry the singleton's expressible half: one client, one mode, two engine
rows (clean: one row). The conventional arm checks the count and both
access paths, so it catches everything the kernel catches on this instance.
The control's unique set must be empty and its verdict `killed`; a
non-empty unique set is a falsifier, not a win.

## Conventional baseline protocol (frozen before the run)

`prototype/singleton_baseline.py` is the conventional arm: the test a team
writing a singleton would already have. It reads the committed fixtures as
JSON (never calling `catherine`) and, per fixture, asserts:

1. `rows(Engine)` has exactly one element — otherwise
   `singleton:engine-duplicate` (>1) or `singleton:engine-missing` (0);
2. every client's `engine` and `fallback` cells resolve to a declared
   `Engine` row — otherwise `singleton:accessor-dangling@Client#<i>`;
3. every client's two access paths name the **same** row (identity) —
   otherwise `singleton:engine!=fallback@Client#<i>`;
4. every client's `seen` equals `mode(engine(client))` — otherwise
   `singleton:engine.mode!=seen@Client#<i>`.

The probe document `empty-key.ct.json` is never run through the baseline
arm: it is a construction probe, not an instance.

`--assert` exits 1 on any drift from the frozen `baseline_expected` lists
in `faults.json`; `--hash` prints the script sha256 (pinned in the ledger's
`comparison.baseline.artifact`).

## Comparison rule

`unique = executed probe elements − baseline-identified elements`, with:

- a key witness counts once per *object*, not per row pair (the element is
  the duplicate, and a 3-row collision is still one element);
- a construction rejection is recorded in `run.probes`, never counted;
- a hypothesized finding is never counted;
- an empty unique set is `killed`: with no executed probe element outside
  the baseline, the lens buys nothing (parity) or is silent (null result).

## Kill condition

- The control must self-negate (empty unique, verdict `killed`).
- A gap entry (`duplicate-instance`, `missing-instance`) is `killed` when
  the probe emits no element the baseline misses; the gap is recorded in
  `expressibility_gap`, not upgraded to a finding.
- The `equal-duplicate` entry is `killed` even though the kernel emits a
  violation, because the key witness identifies the same element the
  baseline already identifies: value-injectivity is not cardinality.

## Falsifiers

- Any probe finding outside the pre-registered element set.
- A key collision on `duplicate-instance.ct.json` (distinct modes) — that
  would mean keys are not injectivity and the surface is different from
  what this pre-registration read.
- Any `check` finding on `missing-instance.ct.json` (zero rows) — that
  would mean the kernel does constrain existence.
- A fixture edit after the run: the fixture hash pin in the ledger entry
  fails, and the correction must be a superseding entry.
## Observed results (recorded after the run)

**Baseline arm** (`python3 prototype/singleton_baseline.py --assert`, exit
0): sha256
`ff109499c871f2b8361c809daddac175dcac9b805d8227aa938864e8479f5873`,
6 fixture runs over 3 entries, all matching the frozen
`baseline_expected` lists.

**Probe arm, executed findings verbatim.** Labels below are what the
kernel emitted, not assignments:

- `control-fault-disagreement.ct.json`: exit 3,
  `{"label":"executed","kind":"violation","witness":{"equation":0,"row":0,"lhs":1,"rhs":0}}`
- `duplicate-instance.ct.json` (two rows, distinct modes, key accepted):
  exit 0, `"findings":[]`
- `equal-duplicate-instance.ct.json` (two rows, equal modes):
  exit 3, `{"label":"executed","kind":"violation","witness":{"key":0,"rows":[0,1]}}`
- `missing-instance.ct.json` (zero rows, no clients): exit 0,
  `"findings":[]`
- `missing-instance-dangling.ct.json` (zero rows, null accessors):
  exit 4,
  `{"label":"executed","kind":"dangling","witness":{"arrow":"engine","src":0}}`
  and
  `{"label":"executed","kind":"dangling","witness":{"arrow":"fallback","src":0}}`
- `empty-key.ct.json` (construction probe): exit 1, envelope
  `code: invalid_key`, message
  ``key declares no arrows: an injectivity claim needs at least one arrow (line 16, column 38)``
  — recorded as a probe, never a finding.

Observed comparison:

| Entry | Baseline detections | Probe elements | Unique |
|---|---|---|---|
| `control` (clean) | — | — | ∅ |
| `control` (fault) | `engine-duplicate`, `engine!=fallback@Client#0` | `engine!=fallback@Client#0` | ∅ |
| `duplicate-instance` | `engine-duplicate` | ∅ | ∅ |
| `equal-duplicate` | `engine-duplicate` | `engine-duplicate` (key witness) | ∅ |
| `missing-instance` | `engine-missing` | ∅ | ∅ |
| `missing-instance-dangling` | `engine-missing`, `accessor-dangling@Client#0` | `accessor-dangling@Client#0` | ∅ |

Every prediction held: the shared-reference half is an executed equation
and the cardinality half never fires. The control self-negates (empty
unique, `killed`); all four entries are `killed`, and the two null entries
(`duplicate-instance`, `missing-instance`) carry the expressibility gap.

**The named missing law (still gap).** Candidate law: *exact cardinality /
unique element* — `|rows(Engine)| = 1`, i.e. the declared singleton object
has exactly one row. Missing surface predicate: an object-level
**cardinality/existence constraint over a row set** (`exactly_one(S)`,
`nonempty(S)`, `|rows(S)| = n`). Why it is missing: the format's
constraints are relative to rows that already exist — path equations are
pointwise over existing rows, declarative keys are value-tuple injectivity
(and reject the empty tuple, the one spelling that would mean
"at most one"), the rule lens covers a declared input domain rather than a
row count, and `search` bounds are an exploration budget over synthetic
instances (default 0, zero-size = UNKNOWN). Instance rows are declared
data, so an empty object parses cleanly and a second row is legal unless
an author-declared key or equation happens to catch it. The Singleton row
stays `unexpressible`; re-opening it requires the cardinality/existence
predicate, not another fixture. No detection-rate claim is made: the
shared-reference half buys parity with the conventional test, and the
cardinality half is a null result.

**Ledger.** `ledger-check ledger/katas/singleton` exits 0:
`ok: 4 entries (4 kata), rollup verdict: (no rollup)`. Entries:
`0001-0dfd4f40.json` (control), `0002-77d75c28.json` (duplicate, gap),
`0003-7ba567ec.json` (equal duplicate, parity), `0004-690f0c6f.json`
(missing instance, gap + construction probe). All verdicts `killed`.
