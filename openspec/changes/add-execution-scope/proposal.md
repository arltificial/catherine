# Change: Add an execution-scope field to the check and findings payloads
> Beads: `catherine-kpn` · epic `catherine-kpn` (three-axis gate: axis A
> referential agreement, axis B schema evolution, axis C executable
> limits). This change is axis C's honesty floor: without it, axis C's
> done-when ("every run distinguishes clean from unchecked") is
> unmeasurable, because a below-floor document, a constraint-free
> document, and a zero-row instance all exit 0 with an empty
> findings envelope — indistinguishable from a genuine clean pass.

## Why

Today a `catherine check` / `catherine findings` run that executed zero
constraints over zero rows emits exactly the same JSON bytes as one
that ran a full check walk and found nothing. Exit 0 plus an empty
`data.findings` array is the payload for both "checked and clean" and
"nothing was checked": a schema with no accepted constraints, an
instance with no rows, and a document below the execution floor all
produce it. Any consumer — an agent looping on exit codes, or the
wasm/web view — cannot tell a genuine pass from a vacuous one, so the
"clean" verdict is not honest. This is the axis-C gap: executable
limits are only enforceable if the payload reports what was actually
executed.

## What Changes

- The check and findings payloads gain one additive field,
  `data.scope`, with exactly three snake_case keys:
  `constraints_checked`, `rows_evaluated`, `equations_over_zero_rows`.
  A vacuous run reports zeros — honestly "nothing was checked" —
  instead of a clean-shaped empty envelope indistinguishable from a
  pass.
- The scope is computed in one shared place and threaded through the
  single envelope-construction path both subcommands use
  (`crates/catherine-cli/src/render/envelope.rs`), so check and
  findings can never disagree about it.
- The field is part of the **public payload contract**, documented in
  the `cli` and `findings` specs: the wasm boundary (`boundary.rs`)
  and the web consumer (`web/src/types/ct-schema-v1.ts`,
  `web/src/findings.ts`) bind to it. It is not an internal
  diagnostic.
- Strictly additive: existing payload fields, all other envelope
  bytes, and every exit code are unchanged. Single-document runs keep
  today's bytes exactly, plus the new field.

### Scope boundary — explicitly out

Only the `check` and `findings` payloads carry `data.scope`.
`search`, `diff`, `migrate`, `rules`, pattern/naturality lenses, and
`query` do **not** execute accepted constraints over an instance, so
they get no scope field. This change also does **not** close the
vacuous-clean-instances registry row — it only makes vacuity visible;
the policy response to a zero scope is a separate decision.

## Capabilities

### Modified Capabilities
- `cli`: the envelope contract gains the additive `data.scope` field
  on check/findings payloads.
- `findings`: the payload contract gains the additive `data.scope`
  field; a vacuous run is distinguishable from a clean pass.

### New Capabilities

## Impact

- Affected code (implementation, per tasks.md): the shared scope
  computation and `envelope.rs` payload construction in
  `crates/catherine-cli`; the wasm boundary mirror in
  `crates/catherine-wasm/src/boundary.rs`; the web bindings in
  `web/src/types/ct-schema-v1.ts` and `web/src/findings.ts`.
- Affected specs: `cli` and `findings` deltas only (this directory).
  No other capability is modified; no crate or web code is touched by
  this change document itself.
- Beads: implements the payload-contract ticket of `catherine-kpn`.
