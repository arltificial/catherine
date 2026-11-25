# Design — the three-axis gate: what catherine will and will not do next

Date: 2026-11-24 · status: decision recorded · project: catherine

## Decision

We do not widen what catherine can express. We make the three axes it already
serves honest and enforceable, and we leave the constraint language where it
is.

Concretely: five small items (one payload field, one parser refusal, one
reuse, one hint, one registry cell), one optional repository asset (the
architecture policy gate), and a set of documentation corrections that cost
nothing but change what the project claims about itself.

Why: the differentiator that is real and hard to copy is not a feature list —
it is that catherine's *limits* are executable (14 expressibility registry rows,
8 open / 6 closed, each with a live CLI case and a CI gate: `just
check-gap-claims` and `cargo test -p catherine-cli --test expressibility_gaps`,
6/6 green). Widening the language dilutes what is already distinctive;
tightening the honesty of the existing axes strengthens it.

## The three axes, and the gate they form

- **Axis A — referential agreement.** Claims of the form "these two stored
  paths must land on the same row". Served by path equations, keys,
  coproducts, pullbacks, naturality, joint composition. Done when every such
  claim a model states is executed with a witness, and what cannot be stated
  is named in a registry row.
- **Axis B — schema evolution.** What a schema change does to the data.
  Served by `migrate` (Δ), `diff`, the schema-map parser. Done when no change
  can silently produce a wrong instance or a lost constraint.
- **Axis C — executable limits.** Knowing and proving what the tool cannot do.
  Served by the expressibility registry, the gap suite, the negative-control
  bundles. Done when every limitation claim names a registry row and a live
  case, and a stale row fails CI.

**The gate:** a proposed capability that serves none of A, B or C is refused by
default. This is the treatment the rule lens already received (a pre-registered
hypothesis, scoped to extracted tables only) — the gate is the repository's own
practice, written down.

## "State-shape domain", defined once

A domain whose claims are about the *shape and agreement of stored references*
— which rows exist, which references they carry, and which composed paths must
agree — rather than about computation over values (arithmetic, time, decisions
over data).

Rule of thumb: if the claim can be written as "these two stored paths must
land on the same row", it is in scope (axis A); if it is about evolution, it is
axis B; if it is about values, time or cardinality, it is out of scope and
belongs in the registry as a named gap (axis C).

This replaces eleven uses of "state-shape" across the design documents that had
no definition, on which the strongest claim in the proposal pivots.

## Two honesty corrections (documentation only, highest priority)

1. **The `search` surface is misdescribed.** README.md:22 and
   PROPOSAL.md:109-110 promise "the smallest breaking instance"; the shipped
   surface returns a *satisfying* witness (`SearchOutcome::Witness` in
   `crates/catherine-analysis/src/search.rs`). The breaking instance comes from
   `check` over a supplied instance and from `diff` as a separating witness.
2. **The tool does not say "model this conventionally"; the docs say it does.**
   README.md:30-31, USING.md:97-99 and 224-226, PROPOSAL.md:113-114,
   DESIGN.md:68-69 attribute the verdict to the product. `grep -rn "model this
   conventionally" crates/` returns no matches, and the FizzBuzz negative
   control is silent by design (exit 0, empty, `warnings: []`). Either
   implement it as a `warnings`/`hints` line, or restate it as the process rule
   it is. The honesty claim *is* the differentiator, so it must be true.

## The optional asset: the architecture policy gate

catherine can check catherine's own dependency architecture: nine permitted
edges authored from DESIGN.md:156-159, README.md:205 and docs/kani-lane.md
(never from Cargo.toml), with every actual `[dependencies]` edge required to
be covered by exactly one policy edge (coproduct coverage) and each pairing
required to agree on both endpoints (equations).

Demonstrated with four directions: the tree as it is → exit 0; a forbidden
edge (`core→cli`, `core→genesis_vibes`, `kani→core`) → exit 3 with an
"uncovered" witness row; a policy edge the tree lacks → exit 4, dangling.

Three conditions, or the gate is theatre: (1) the policy is a *design
artifact* with an amendment path — a policy regenerated from Cargo.toml is the
tautology a review caught; (2) the extractor is code and needs its own test,
and the four runs are the negative-control bundle; (3) the failure message
names the offending edge. What the gate would *not* prove is that the
architecture is right — only that it matches a policy someone wrote down.

## Not doing (and why)

- **Identity path form.** A working workaround exists in the shipped katas
  (identity laws stated as fixed-point over a concrete arrow), and equational
  algebra is the classic route to a generic checker.
- **Cardinality / existence predicates.** Real state-shape claims, but they
  widen the constraint language toward a constraint solver — admissible only
  after the state-shape definition lands, and then only with a triviality-floor
  test.
- **A general value / multi-valued-cell type.** That is a data model, not a
  kernel.
- **Default row bounds for `search`/`diff`.** A hint naming the bound to pass
  is enough; deriving bounds from the instance would change what "within
  bounds" means.
- **"Model this conventionally" as a new finding kind.** The honest verdict
  belongs in the `hints` channel, not in a new exit class.

## Done-when

The programme is finished when: the documentation corrections are landed; the
five items have landed; the registry rows that the landed capabilities close
are `closed` with sealing evidence and the rows they do not close are still
`gap` and say why; `catherine check` reports its execution scope, so "clean" is
distinguishable from "unchecked"; and the architecture gate, if it lands, has
its policy, its extractor test and its negative-control bundle.

Beyond that the backlog does not grow by default: a new item must name the axis
it serves, or be refused.

## Open items for the owner

1. Where this decision should live in the repository — `docs/`, or beads
   issues — which decides whether the documentation corrections land as doc
   PRs or as tracked work.
2. Whether the five items open as beads issues now, with which red→green test
   shapes.
3. Whether the architecture asset stays a proposal or becomes work; it is the
   only item needing new code in the repository's trusted path, so it should
   land behind the project's TDD pipeline.

## Method note

All HIGH findings of this session's reviews went through a TypeSafe
verification pass (one parallel `jev-latest` request per review). In the
review of the conclusion and proposal: of six HIGH findings, one survived above
the gate (0.96), two are REVIEW_REQUIRED, and three were dropped or
contradicted — a measured 75% false-positive rate. The lesson is calibration:
reserve HIGH for claims that would ship a wrong statement; process and design
gaps are MEDIUM.

