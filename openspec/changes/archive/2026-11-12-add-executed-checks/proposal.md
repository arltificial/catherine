# Change: Add executed checks — instance tables and path-equation enforcement
> IDs: E1–E10 = docs/DESIGN.md §0 evidence base · C1–C9 = docs/PROPOSAL.md §2 conclusions

## Why

Declarations do not enforce; checks do (E3 — Catlab's ACSet path equations
are documentation, and the smoke gate proved it). Executed checks ARE the
product: "if nothing else ships, this must" (DESIGN §6, P1). The P0
kernel parses model documents; nothing yet evaluates declared equations
against instances. Beads ticket: `catherine-0gk` (P1; blocked by
`catherine-bpz`, blocks `catherine-1x7`).

## What Changes

- Kernel instance tables ported from specodelic `acset/instance.rs`
  (925 LOC, Apache-2.0): per-object dense row ids, per-arrow
  `Vec<Option<RowId>>` morphism tables — behind a **graph-parity gate**
  (E8) with concrete mechanics: parity fixtures vendored from
  specodelic's corpus (schema/instance exports + expected finding sets)
  under `fixtures/parity/`, run as a named test target that must pass
  before any rewrite of ported logic
- Instance builder from the parsed `.ct.json` presentation: typing
  enforced at build (an arrow value naming a row of the wrong object is a
  typing violation, not silently stored — adapted from acset-core
  `typing_enforced_at_build`)
- **Executed path-equation checks**: `check(schema, instance) ->
  Vec<CheckOutcome>` evaluates every equation on every applicable row;
  violations carry witness rows with both path values
  (`lhs=<RowId>, rhs=<RowId>`)
- Equation `status` semantics (the decision deferred from P0): `accepted`
  equations produce violation findings; `proposed` equations are executed
  but their failures are emitted as **hypothesis-labeled** outcomes — a
  proposed equation is a domain question under review, not a broken
  invariant
- Totality rules: empty schema or empty instance → zero outcomes, OK;
  self-loop arrows are legal; dangling values short-circuit a path
  evaluation at that arrow (the path cannot compose — recorded, not
  crashed)
- Openspec capability `kernel-checks`; parity gate results recorded in the
  change before any rewrite of ported logic

## Impact

- Affected specs: `kernel-checks` (new capability; consumes
  `model-document`)
- Affected code: `crates/catherine-core` (new `instance` + `checks`
  modules); no CLI/wasm surface yet
- Beads: implements `catherine-0gk`; unblocks `catherine-1x7` (P2
  taxonomy)
- This is the E3 bet: the phase that justifies the project's existence
