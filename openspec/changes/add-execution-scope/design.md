# Design: add-execution-scope

Grounded in the current implementation (2026-11-24 read):
`crates/catherine-cli/src/render/envelope.rs` (the deduplicated
envelope construction — `FindingsData { findings }` and
`SourcedFindingsData` are the only two check/findings payload shapes),
`openspec/specs/findings/spec.md`, and the wasm boundary
(`crates/catherine-wasm/src/boundary.rs`, which already mirrors
envelope-shaped payloads for the web).

## Decision 1: `data.scope` is part of the public payload contract (recorded, not relitigated)

The scope field lives in the **public** payload contract documented in
the `cli` and `findings` specs — not as an internal diagnostic on the
warnings channel, not as a sidecar log. Rationale: the wasm/web
consumer binds to the payload, and axis C's done-when requires *every*
consumer to distinguish clean from unchecked. A private diagnostic the
web layer cannot see would leave the wasm consumer making the exact
same vacuous-clean mistake the field exists to prevent. Additive JSON
fields keep the contract compatible for every existing consumer; the
cost is a permanent field, accepted up front.

## Decision 2: exactly three keys, snake_case, no nesting

`data.scope = { constraints_checked, rows_evaluated,
equations_over_zero_rows }`. Naming follows the existing payload
conventions (`data.findings`, witness keys like `coproduct`/`row`,
the contradiction scope's `keys`/`coproducts`/`pullbacks` — snake_case
single nouns, no camelCase). The three keys are minimal and
sufficient:

- `constraints_checked` — how many accepted constraints the check walk
  actually executed. Zero ⇒ nothing was checked regardless of findings.
- `rows_evaluated` — how many instance rows the walk touched. Zero ⇒
  vacuous instance.
- `equations_over_zero_rows` — how many accepted equations bind zero
  rows. This is the subtle vacuity: constraints > 0 and rows > 0 can
  still mean every equation ranged over an empty table.

Deeper instrumentation (per-constraint timings, per-family counts) is
deliberately excluded — the goal is clean-vs-unchecked
distinguishability, not profiling; extra keys would invite the wasm
binding to over-commit to unstable detail.

## Decision 3: computed once, shared by check and findings

The scope is computed in **one place** in the CLI layer and threaded
through the single envelope-construction path
(`render/envelope.rs`) that both subcommands already share. Two
call sites computing their own scope is how the two subcommands
drift; the deduplicated-envelope invariant is the existing repo
pattern (the file's header states one path per envelope shape) and
this field follows it. The kernel (`catherine-core`) already counts
what the walk touches during execution; the CLI surfaces it, it does
not recompute it.

## Decision 4: zeros are the honest empty scope, never omission

A zero-row document, a constraint-free document, or any run whose walk
touches nothing reports `data.scope` with all three keys `0`. Omitting
the field (or serializing `null`) would recreate the ambiguity inside
the field itself — absent vs zero has no agreed meaning. The field is
always present on check/findings payloads, including multi-document
composition runs, where the scope is the joint walk's total.

## Decision 5: bytes elsewhere are frozen

Existing payload fields, all other envelope bytes, and all exit codes
are unchanged. Single-document runs keep today's encoding exactly,
plus the new field — verified by golden fixtures updated by adding the
field, never by regenerating. Below-floor and parse-failure paths are
untouched: an envelope that never reached the check walk carries no
execution scope to report, and its behavior is today's behavior.
