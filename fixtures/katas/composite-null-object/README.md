# Kata — Composite with a Null Object (`composite-null-object`)

Ticket: `catherine-lgl.1` (P2 kata slice). Status: **pre-registration
committed before the recorded run**; observed counts below are added only
after both arms execute and are labelled observed, never presented as
predictions.

## Hypothesis (falsifiable)

A legacy Composite tree with a Null Object has executable structural laws
the `catherine` kernel can enforce — every child's `parent` resolves, every
node's `parent` belongs to the same composite root, and the `nil` row is a
neutral identity rather than a real leaf — and the kernel names those
defects on the committed fixtures. The claimed edge over a conventional
scripted traversal test is **not detection rate** (parity on the sampled
check faults); it is that the kernel check is exhaustive (no sample
boundary) and the Δ migration surface names each lost child link as a
witness. If the unique set is empty everywhere, the kata is a null result
and the control verdict carries it.

## Pattern ↔ abstraction mapping (ploeh grounding §6, candidate 2)

The Composite pattern's `flatten` is a monoid: `Nil` is the identity and
concatenation is associative. **Associativity is not expressible with the
v1 kernel surfaces**: the instance language has no list, multiset, order,
operation object, or function-space object, so there is no path equation
that can state `flatten(flatten(a) ++ b) = flatten(a ++ flatten(b))`. No
kernel feature is invented for it; the gap is recorded in the ledger
entry (`expressibility_gap`). The closest executable laws are:

| Law | Kernel statement | What it enforces |
|---|---|---|
| L1 root closure + nil neutrality | `parent ∘ top = top` | a node's parent resolves to the same composite root; `parent(nil)=nil` is forced at the nil row (any real parent breaks L1) |
| L2 root idempotence | `top ∘ top = top` | the root map is idempotent, so every root value is a fixed point |

Conditional totality ("every **non-root** node has a real parent") is also
not expressible: the kernel has one global equation language, no
per-row guard. The model encodes root termination with the `root` and
`nil` self-loops instead, and states so rather than hiding the gap.

A third gap is recorded in the ledger: `top(nil)=nil` is part of the
intended model but is **not** independently enforced — with
`top(nil)=root`, both L1 and L2 still hold, because the kernel cannot pin
a path value to the distinguished row. Nil neutrality is enforced for the
parent fixed point, not for `top`.

## Model

One sort `Node`; rows are node identifiers plus the distinguished `nil`
row (the Null Object) and the `root` row. Row ids are dense in sorted row
name order (the kernel's documented assignment).

- `parent: Node → Node` — each node's parent; `root` and `nil` are
  self-looped (no real parent).
- `top: Node → Node` — the composite root each node belongs to; `nil`
  maps to itself.
- Equations: `parent ∘ top = top` (L1), `top ∘ top = top` (L2), both
  `accepted`.

Above-floor rows `c1…c5, nil, root` form a three-level tree with two
root branches (`c1→c2→c3`, `c4→c5`). Near-floor floor rows
`child, nil, root` form a one-level composite.

## Fault matrix (frozen before the executed run)

| Fixture | Class | Injected fault | Expected kernel elements |
|---|---|---|---|
| `floor/fault-floor-control.ct.json` | floor control | `parent(child)=null`; `parent(nil)=root` | `dangling:parent@Node#0`, `violation:eq0@Node#1` |
| `fault-broken-parent-link.ct.json` | broken child→parent link | `parent(c4)=null` (deep) | `dangling:parent@Node#3` |
| `fault-nil-as-real-leaf.ct.json` | nil treated as a real leaf | `parent(c5)=nil` (deep node mounted on Nil) | `violation:eq0@Node#4` |
| `fault-lost-child-migrate.ct.json` | lost child on flatten/Δ | schema map drops `parent` into the flat target, keeps `top` | seven `lost_cell:parent#0…#6` |

The `fault-lost-child-migrate` fixture is the clean above-floor model with
a `schema_map` to a `FlatNode` target that keeps every row and the `top`
map but drops the `parent` arrow: the flattening keeps the nodes and loses
every child→parent link, loudly, cell by cell.

## Near-floor control

`floor/fault-floor-control.ct.json` is a one-level composite (three rows).
The conventional traversal test's pre-registered sample covers every real
node (`child`), and it always checks the `nil` row, so it catches **both**
injected check faults. The probe catches the same two elements; the unique
set must be empty and the control verdict `killed`. The floor control does
**not** include a Δ migration fault: the conventional script does not
execute a schema map at all, so a migrate loss is a property of the
baseline's surface coverage, not of the composite's depth, and putting it
in the floor would not be a near-floor test of the control's claim.

## Conventional baseline protocol (committed before the run)

`prototype/composite_null_object_baseline.py`, sha256
`83d443b58296df3cdb5213efff6888fe974f190e8e01fe85684aa390469b4a6a`,
is the scripted traversal test: for the pre-registered node sample of each
fault it asserts `parent` resolves and `top(node) = top(parent(node))`; it
always asserts the `nil` row is self-looped on `parent` and `top`. It does
not enumerate unsampled nodes and does not run `migrate`. `--assert`
executes both arms and compares detections, identified elements, probe
elements, unique count, and derived verdict with `faults.json`; a mismatch
exits 1. `--hash` prints the pin.

Comparison rule: an element is `dangling:<arrow>@<object>#<row>`,
`violation:eq<n>@<object>#<row>`, or `lost_cell:<arrow>#<row>`; an arm
identifies an element only by naming it. `unique = probe elements minus
baseline-identified elements`. Verdict: empty unique → `killed`; non-empty
unique with an executed arm → `survived`.

## Kill condition

- Any probe element outside the frozen expectations, or a baseline
  `--assert` mismatch: the fixture/arm moved; record the discrepancy and
  re-pin only with a superseding ledger entry.
- The floor control must self-negate (unique empty, `killed`); otherwise
  the floor is not at the conventional test's floor and the kata's unique
  claims are not admissible.
- If the Δ surface emits no lost-cell witness for the flatten fixture, the
  migration claim is refuted and the null result is the deliverable.
- The monoid associativity law remains a recorded expressibility gap in
  every outcome; it is never claimed as checked.

## Execution

```
python3 prototype/composite_null_object_baseline.py --assert
cargo test -q -p catherine-ledger --test composite_null_object_kata
cargo run -q -p catherine-ledger --bin ledger-check -- ledger/katas/composite-null-object
```

## Observed results (recorded after the executed run)

| Fixture | Baseline detections | Probe elements | Unique | Verdict |
|---|---|---|---|---|
| floor control | `dangling:parent@Node#0`, `nil-parent:Node#nil` | 2 (`dangling`, `violation`) | 0 | killed |
| broken parent link | none | 1 `dangling:parent@Node#3` | 1 | survived |
| nil as real leaf | none | 1 `violation:eq0@Node#4` | 1 | survived |
| lost child migrate | none | 7 `lost_cell:parent#*` | 7 | survived |

Observed unique total: 9 (control 0; positive 9). The kernel check is
exhaustive where the baseline is sampled, and the Δ surface names each
dropped child link. No detection-rate win is claimed beyond these
pre-registered elements.