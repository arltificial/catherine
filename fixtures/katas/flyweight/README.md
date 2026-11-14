# Kata slice — flyweight sharing law (pre-registration + observed results)

Ticket `catherine-r5k.8` (epic `catherine-r5k`). This slice re-opens the
Flyweight row of `docs/ref/pattern-kata-dispositions.md`, which records
`unexpressible` on the strength of `ledger/katas/weak-lens-set/` entry 4
(a format-1.0 fixture with **zero equations**, so the kernel emitted zero
findings). Re-opening a row requires **naming the law first**; that is
what the section below does, before any recorded kernel run.

## Law named first

The candidate in the ticket: *sharing / identity law over declared
instance counts*. On the kernel surface that decomposes into four
statements with different expressibility. The names are fixed here and
are the vocabulary of the ledger entries.

**F-Intern — one shared instance per intrinsic state** (primary)

> For the declared instance, `intrinsic: Flyweight → Intrinsic` is
> injective over the flyweight rows: no two distinct flyweight rows carry
> the same intrinsic tuple.

Surface: a format-1.3 **declarative key** `{object: Flyweight, arrows:
[intrinsic], status: accepted}`. Each colliding unordered pair is one
executed `violation` with witness `{key, rows}`. This is the sharing
law's counting content that the surface *does* have: pair existence, not
arithmetic.

**F-Handle — the client's handle is the factory's instance**

> For every declared client `c`, `flyweight(c) = factory(key(c))` as a
> **row co-reference** (dense row id), not a value comparison.

Surface: the path equation `lhs [key, factory] / rhs [flyweight]` over
`Client → Flyweight`. The equation compares the two paths' target row
ids, so two rows with *equal intrinsic values* are still a violation —
which is exactly the identity content the earlier slice declared absent.

**F-Closure — no leaked instance** (named, unexpressible)

> Every declared Flyweight row is in the image of `factory`.

Missing surface predicate: a **surjectivity / image-closure predicate**
over declared rows (or an unreachable-instance finding).
`query --follow factory --direction backward` computes the preimage, but a
query result is data, never a finding; `check` emits nothing.

**F-Capacity — declared instance counts** (named, unexpressible)

> The number of flyweight instances, or one intrinsic state's share
> count, obeys a declared numeric bound.

Missing surface predicate: an **arithmetic/cardinality constraint** over
declared instance counts. The closed constraint vocabulary is path
equality, totality, injectivity keys, and presentation rules; none
carries a numeral over rows. F-Intern observes pairs, never counts.

## Fixtures

All fixtures are format 1.3 over `Client / Key / Flyweight / Intrinsic`:

- arrows `key: Client → Key`, `factory: Key → Flyweight`,
  `intrinsic: Flyweight → Intrinsic`, `flyweight: Client → Flyweight`;
- equation `eq0` = `lhs [key, factory]`, `rhs [flyweight]` (F-Handle);
- key 0 = `{object: Flyweight, arrows: [intrinsic]}` (F-Intern).

| Role | Clean fixture | Seeded-fault fixture | Law under test |
|---|---|---|---|
| near-floor control | `control-floor.ct.json` | `control-floor-fault-rerouted-handle.ct.json` | F-Handle |
| sharing (two clients, one key each) | `flyweight.ct.json` | `flyweight-fault-duplicate-intrinsic.ct.json` | F-Intern |
| sharing | `flyweight.ct.json` | `flyweight-fault-rerouted-handle.ct.json` | F-Handle |
| sharing | `flyweight.ct.json` | `flyweight-fault-orphan-instance.ct.json` | F-Closure |

Seeded faults:

- **rerouted handle** — a client holds a different (otherwise valid)
  instance than the factory returns for its key;
- **duplicate intrinsic** — the factory created a second instance for an
  intrinsic state that already has one (both handles agree, both rows are
  factory images: only injectivity can see it);
- **orphan instance** — a declared instance with a *distinct* intrinsic
  value that no key can produce (handles and injectivity both hold: only
  an image-closure predicate could see it).

`fixtures/katas/flyweight/faults.json` carries the machine-readable
pre-registration: element vocabulary, per-fixture expectations, and the
expected probe findings verbatim.

## Pre-registered element vocabulary

Detection is arithmetic, not judgement: an **element** is the named
witness both arms may identify. `unique = probe_elements −
baseline_elements`.

| Element | Meaning |
|---|---|
| `flyweight:handle-identity@Client#<idx>` | client row `idx`'s handle differs from `factory(key(client))` |
| `flyweight:interning-pair@Flyweight#<lo>/Flyweight#<hi>` | declared rows `lo < hi` collide on their intrinsic tuple |
| `flyweight:pool-closure@Flyweight#<idx>` | declared row `idx` is not in the image of `factory` |

Probe mapping: an equation witness `{equation: 0, row: r}` maps to the
handle element of `Client#r`; a key witness `{key: 0, rows: [lo, hi]}`
maps to the interning-pair element. `pool-closure` elements have **no
probe counterpart by construction** (F-Closure is unexpressible), so they
can never be unique.

## Fault matrix (pre-registered)

| Fault | Law | Probe finding expected | Conventional catch expected | Unique expected | Verdict expected |
|---|---|---|---|---|---|
| `floor-rerouted-handle` (control) | F-Handle | 1 violation `eq0@row0` | yes: handle 0 + closure 1 | 0 | `killed` (control self-negates) |
| `duplicate-intrinsic` | F-Intern | 1 violation `key0,[0,1]` | yes: interning pair 0/1 | 0 | `killed` (parity) |
| `rerouted-handle` | F-Handle | 1 violation `eq0@row1` | yes: handle 1 | 0 | `killed` (parity) |
| `orphan-instance` | F-Closure | **none** | yes: closure 1 | 0 | `killed` (null; gap recorded) |

## Near-floor control

`control-floor.ct.json` is the smallest instance with a key and a handle
equation: one client, one key, one flyweight, one intrinsic. Its seeded
fault reroutes the single handle and leaves a second (distinct-intrinsic)
row reachable by no key. The frozen conventional contract covers the
whole declared instance, so it catches everything the probe can see
(handle element 0) plus the pool element. Unique must be empty: the
control self-negates and its verdict is `killed`.

## Baseline protocol (frozen)

`prototype/flyweight_baseline.py` is the conventional arm: a scripted
Flyweight test reading the same committed fixtures, never calling
`catherine`. Its three assertions are the complete conventional contract
— deliberately including the pool-level ones so a non-empty unique set
can only come from the kernel's own surface being stronger, never from a
weakened baseline:

- **handle identity** — every client's handle is the row the factory
  returns for its key (row identity, not value equality);
- **interning pairs** — no two declared flyweight rows share an intrinsic
  tuple (ascending unordered pairs);
- **pool closure** — every declared flyweight row is an instance the
  factory can return.

`--assert` requires every clean fixture to yield no detection, every
fault fixture to yield exactly the pre-registered set, and the
pre-registration's own arithmetic (`probe_expected − baseline_expected =
unique_expected`) to hold. `--hash` prints the script sha256 for the
ledger arm.

## Probe arm

The real `catherine` binary, no kernel changes and no new CLI:
`cargo run -q -p catherine-cli -- check <fixture> --json`. Findings are
recorded verbatim in the ledger. One discovery-surface observation is
recorded too (never counted as a detection): `underconstraint
--max-path-length 3` on the clean base still raises
`flyweight∘intrinsic` vs `key∘factory∘intrinsic` as a hypothesized
parallel path — the kernel *asks* the composite consequence of F-Handle
rather than deriving it.

## Verdict procedure (frozen)

1. Control first: empty unique ⇒ `killed`; non-empty unique is a
   falsifier of the control and invalidates the run.
2. Positive with an expressible law: unique empty ⇒ `killed` (parity);
   unique non-empty with an executed baseline arm ⇒ `survived`.
3. Positive with the law unexpressible at fixture level (no predicate can
   name the claim) and zero probe findings ⇒ `killed`, with the gap
   recorded. The null result is the deliverable.
4. No verdict is forced upward: `survived` is not a goal of this slice,
   and no detection-rate claim is made from it.

## Kill condition / falsifiers

- Any probe finding outside the pre-registered set — the fixture or its
  expectation moved; record the discrepancy, never edit this matrix.
- Baseline `--assert` mismatch — the conventional arm is wrong or a
  fixture changed; re-pin only with a superseding entry.
- A non-empty unique set caused by a weakened baseline — the slice is
  refuted as a comparison, not promoted to `survived`.
- The control is non-empty or its verdict is not `killed` — the run is
  invalid; the comparison is void.

## Execution

```
python3 prototype/flyweight_baseline.py --hash
python3 prototype/flyweight_baseline.py --assert
cargo run -q -p catherine-cli -- check fixtures/katas/flyweight/<fixture>.ct.json --json
cargo run -q -p catherine-ledger --bin ledger-check -- ledger/katas/flyweight
```

## Observed results (appended after the run)

The pre-registration above is unchanged. The kernel findings below are the
real CLI's, recorded verbatim in the ledger. Timing, stated plainly: the
fixtures were authored first and the expectations were derived by hand
from each instance and the declared surfaces; the expectation set was
pinned in `faults.json` + this file, and the kernel runs recorded in the
ledger came after that pin and matched it element-for-element. (A first
`check` invocation on the authored fixtures was used to confirm the model
documents parse and to read the raw finding shape; no expectation was
changed after the pin.)

| Fixture | Exit | Kernel finding | Conventional catch | Unique | Verdict |
|---|---|---|---|---|---|
| `flyweight.ct.json` (clean) | 0 | none | none | 0 | — |
| `control-floor.ct.json` (clean) | 0 | none | none | 0 | — |
| `control-floor-fault-rerouted-handle` | 3 | `violation` `eq0@row0` lhs 0 rhs 1 | handle 0 + closure 1 | 0 | `killed` — control self-negates |
| `flyweight-fault-duplicate-intrinsic` | 3 | `violation` `key0` rows `[0,1]` | interning pair 0/1 | 0 | `killed` — parity |
| `flyweight-fault-rerouted-handle` | 3 | `violation` `eq0@row1` lhs 1 rhs 0 | handle 1 | 0 | `killed` — parity |
| `flyweight-fault-orphan-instance` | 0 | **none** | closure 1 | 0 | `killed` — null, gap recorded |

Baseline arm: `prototype/flyweight_baseline.py` sha256
`c5c9c28bc4e1bb75a7b2fb64bf41a66d908cd3fee468f7fd00150c26cdbd6d30`,
`--assert` exit 0 over 2 clean + 4 fault fixtures.

### What the slice establishes

1. **The prior disposition was too strong, in one direction.** "Identity is
   data; no predicate" holds only for the *unstated* case: the
   `weak-lens-set` fixture declared no equation and no key (format 1.0),
   so its zero findings measured that fixture, not the pattern. With the
   law declared, the kernel emits real findings for both the co-reference
   fault (equation witness, dense row ids) and the duplicate-intrinsic
   fault (key witness, colliding pair) — structurally, with no runtime
   identity needed.
2. **Parity, not advantage.** The complete conventional Flyweight test
   catches every fault the kernel catches, so every positive's unique set
   is empty and the verdict is `killed`. This slice claims no
   detection-rate win.
3. **Two laws remain unexpressible, named exactly.** F-Closure (image
   closure / no leaked instance) has no predicate: keys state injectivity
   only, equations compare path values from one source row, and
   `query` returns data rather than findings. F-Capacity (numeric counts)
   has no predicate: the kernel observes colliding pairs, never counts.
4. **Co-reference is model-local.** F-Handle proves the two declared paths
   name the same dense row; it says nothing about runtime aliasing,
   lifetime, reference counting, or mutation — the model has no such
   surface, and no claim is made about it.