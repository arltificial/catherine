# Plan — landing the three-axis gate (sequencing, test shapes, gates)

Date: 2026-11-24 · project: catherine · depends on: the design artifact
"the three-axis gate: what catherine will and will not do next"

Sequencing principle: cheapest first, and documentation before code — the two
honesty corrections change what the project claims about itself and cost
nothing but edits.

## Phase 0 — documentation only (no code, no pipeline)

0.1 **Fix the `search` semantics mismatch.**
    Files: README.md:22, docs/PROPOSAL.md:109-110. Reword to "bounded
    *satisfying*-instance search (smallest model of the schema within bounds) —
    `diff` supplies the breaking/separating instance". Verification: the
    implementation contract in `crates/catherine-analysis/src/search.rs`
    (`SearchOutcome::Witness` = "the smallest fully resolved instance
    satisfying every accepted equation").

0.2 **Fix the "model this conventionally" claim.** Either implement it as a
    `warnings`/`hints` line on a document with no executed constraints (or a
    below-floor rule table), or restate the docs as the process rule it is.
    Locations: README.md:30-31, docs/USING.md:97-99 and 224-226,
    docs/PROPOSAL.md:113-114, docs/DESIGN.md:68-69.

0.3 **Rewrite the one-sentence value proposition** around referential claims
    plus the unmatched evolution axis, and stand up one normative constraints
    surface (axis A/B/C table + the "state-shape domain" definition) that
    PROPOSAL, DESIGN and USING all point at, with the expressibility registry
    linked as its executable half.

0.4 **Registry cells.** The identity-gap row states the workaround (identity
    laws as fixed-point over a concrete arrow). The `vacuous-clean-instances`
    row stays `gap` and says why (a payload field is not a warning).

Gate: `just check-gap-claims` stays green; a documentation-only change needs
no pipeline.

## Phase 1 — small code

1.1 **Item 4 — the `unknown` payload names the bound to pass.** One field:
    which flag and a suggested bound. Axis: ergonomics over A.

1.2 **Item 1 — scope count on every answer** (constraints checked, rows
    evaluated, equations over zero rows). Axis: C (honesty). Scope corrected:
    the envelope is catherine-local (`crates/catherine-cli/src/render/envelope.rs`),
    but the wasm consumer and the openspec cli/findings specs read the payload
    — so this goes through an openspec change, with `ah check --changes` and
    `just check-wasm` as gates. It does *not* close the
    `vacuous-clean-instances` row.

Test shape (red → green): a zero-row document and a constraint-free document
both report their (empty) scope, so "clean" is distinguishable from
"unchecked"; single-document runs keep today's bytes and exit codes elsewhere.

## Phase 2 — the evolution axis (B)

2.1 **Item 2 — mapped-arrow endpoint typing** in the schema-map parser
    (`crates/catherine-core/src/parse/schema_map.rs`; the code already names
    the missing check: "the composable-path typing check is phase B").
    Test shape: a map whose arrow path's endpoints do not match the mapped
    objects is a labeled refusal; a well-typed map migrates unchanged.
    **Flip protocol:** the `non-functorial-schema-map` row flips to `closed`
    with sealing evidence in the same change
    (`docs/ref/expressibility-dispositions.md:30-33`; the gap suite parses that
    table).

2.2 **Item 3 — `migrate` reuses the diff classifier, or hints at it.** Axis B,
    reuse over new semantics. The classifier exists
    (`diff` → "weakened — a v1-enforced constraint is not enforced by v2", with
    a separating witness); what is missing is wiring, because the migrate
    document embeds `schema_map.target_schema`. **Flip protocol:** the
    `dropped-equations-on-migrate` and `dropped-pullbacks-on-migrate` rows
    flip to `closed` in the same change — or the change lands only as the
    cheap hint and the rows stay open, which must be stated.

## Phase 3 — the optional asset

3.1 **The architecture policy gate.** Permitted-edge list authored from
    DESIGN.md:156-159, README.md:205, docs/kani-lane.md (never from
    Cargo.toml); an extractor reading only `[dependencies]`; the extractor
    gets its own test; the four-run negative-control bundle (clean;
    `core→cli`; `core→genesis_vibes`; `kani→core`; plus the missing-edge
    case); the failure message names the offending edge.

    This is the only item that adds new code to the repository's trusted
    path — it lands behind the project's TDD pipeline, and the policy is a
    design artifact with an amendment path, not a convenience edit.

## Gates for the whole plan

- `cargo test --workspace` and the contract suite stay green.
- `just check-gap-claims` and `cargo test -p catherine-cli --test
  expressibility_gaps` — the registry is the executable half of the constraint
  set, so a capability change without a row flip fails here.
- `ah check` (and `ah check --changes <name>` for the payload change).
- `just check-wasm` for the payload change.

## Done-when

Phase 0's corrections landed; items 1.1–2.2 landed; the registry rows that
the landed capabilities close are `closed` with sealing evidence and the rest
are still `gap` and say why; `catherine check` reports its execution scope;
the architecture gate, if it lands, has its policy, its extractor test and its
negative-control bundle. Beyond that the backlog does not grow by default: a
new item must name the axis it serves (A, B or C) or be refused.

## Explicitly out of this plan

Identity path form; cardinality/existence predicates; a general value or
multi-valued-cell type; default row bounds for `search`/`diff`; a new finding
kind for "model this conventionally". Reasons are recorded in the design
artifact.

