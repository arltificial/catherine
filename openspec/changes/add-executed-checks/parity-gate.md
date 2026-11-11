# Parity gate result (task 9, criterion 8)

Recorded per the proposal's mechanics: the run over the vendored
`fixtures/parity/` set and its result, recorded in the change **before any
rewrite of ported logic** — this file documents the gate run taken on the
pre-refactor tree (GREEN), and the REFACTOR (extracting the shared
`TypedInstance::compose` helper) re-ran the gate afterwards with identical
findings. Both runs are listed below; the refactor is behavior-identical.

## Run 1 — GREEN (pre-refactor; the run this record exists for)

```
cargo test -p catherine-core --test parity_gate
```

```
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured
```

## Run 2 — post-REFACTOR (behavior-identical re-verification)

```
cargo test -p catherine-core --test parity_gate
```

```
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured
```

## What the gate covers

The named target `tests/parity_gate.rs` runs the vendored fixture set under
`fixtures/parity/` and compares build reports (typing violations, dangling
cells), the built morphism tables, and check outcomes against each
fixture's vendored expected set — structurally, by kind and witness, never
snapshots. The expected sets ARE the parity contract. Donor ground truth:
specodelic's `tests/snapshots/{dangling,typing_allowed,typing_violations}.txt`
oracles and its green `acset_parity` suite (18 passed) over the mirrored
corpora.

| Fixture | Mirrors | Route | Expected findings (all matched) |
|---|---|---|---|
| `small` | `tests/fixtures/small/clean.md` | parse | clean: no reports, value stored, zero outcomes |
| `zero_file` | `tests/fixtures/zero_file` | parse | empty schema + instance boundary: clean |
| `single_intent` | `tests/fixtures/single_intent/one.md` | parse | rows without arrows: clean |
| `empty_instance` | degenerate boundary (change-mandated) | parse | declared schema/equation over zero rows: zero evaluations, zero outcomes |
| `dangling` | `tests/fixtures/dangling/{c1,c2}.md` | parse | 3 dangling cells stored as `None`, reported (arrow, src row); real edges stored |
| `typing_allowed` | `tests/fixtures/typing_allowed/{c1,c2}.md` | parse | cross-layer allowed edges stored as morphism values across 5 arrows |
| `typing_violation` | `tests/fixtures/typing_violations/{c1,c2}.md` | presentation | wrong-object value: typing violation naming (arrow, src row, value); value **not** stored; no check outcome for it |

The `typing_violation` fixture loads through the lenient presentation
loader because its wrong-object table value is gated structurally by P0's
parser (`Syntax`: the value is not a row of the arrow's target object);
the gate asserts that rejection explicitly. This is the acset-core
`typing_enforced_at_build` adaptation surface: the builder re-enforces
typing for every consumer past parse.

Shared acset behavior only: the gate covers instance tables, typing, and
dangling — not equation semantics, which the donor does not have (E10 new
code, covered by `tests/checks_properties.rs`).
