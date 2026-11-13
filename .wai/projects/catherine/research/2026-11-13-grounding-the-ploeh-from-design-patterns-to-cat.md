---
tags: [pipeline-run:epic-orchestrator-2026-11-11-catherine-p4-bounded-analysis-catherine-1ay, pipeline-step:ship]
---

# Grounding the ploeh "From design patterns to category theory" series against catherine

## Why this research

The user pulled Mark Seemann's complete article series *From design patterns
to category theory* (hub: blog.ploeh.dk/2017/10/04, 129 articles, 2017–2025;
local snapshot `~/Downloads/ploeh-series/` with INDEX.md) and asked to ground
it against the current implementation. catherine is itself a category-theory
modeling workbench, so the question is concrete: does the design-level
vocabulary (monoids, functors, monads as API contracts) correspond to anything
load-bearing in the codebase, and would naming it buy anything?

**Decision:** the mapping is valuable as documentation, not as code change.
Written to `docs/ref/ploeh-grounding.md` (docs-only; no behavior change).

## Central finding: one thesis, two levels

- ploeh grounds **API design** in algebra: universal abstractions with laws
  replace ambiguous design patterns ("category theory concepts are governed by
  specific laws… less ambiguous than design patterns").
- catherine grounds **domain models** in execution: "declarations don't
  enforce; checks do" (THEORY.md §4).

Both refuse unevaluated claims. The series is the API-design mirror of
catherine's model-level thesis — worth one reading-order line in THEORY.md §9.

## Correspondence table (verified at commit 2c686e4, file:line-checked)

| ploeh article | catherine evidence | status |
|---|---|---|
| Composite as a monoid (2018-03-12) | `check → Vec<Finding>` checks.rs:47; `findings.sort()` :66 (free monoid normalized to canonical form); empty-in → zero-findings identity | embodied |
| Null Object as identity (2018-04-23) | `Model::empty()` model.rs:337; "silence after evaluation is real success" checks.rs:15–17 | embodied |
| Kleisli composition / Maybe monad (2022-04) | dangling value short-circuits path composition, run continues, typed `DanglingWitness` — checks.rs:9–12, :49 | embodied-unnamed |
| Applicative validation (2018-11-05) | **both layers deliberately**: monadic fail-fast *inside* path composition; applicative collect-all at check level (one finding per equation per row) | embodied |
| Visitor as a sum type (2018-06-25) | tagged enums `FindingKind` findings.rs:163–164, `MigrationFindingKind` migrate.rs:46–47, `DomainQuestion` findings.rs:125; Rust enums are the free encoding — Church encoding deliberately absent | embodied |
| Result isomorphism / Tester-Doer (2025/2019) | total, typing-enforcing build; failures as named values (`TypingViolation`) instance.rs:5–13; "reference integrity as a *value*, not an exception" THEORY §5 | embodied |
| Backwards-compat / LSP as profunctor (2021-12) | schema-diff `Cosmetic/Accreted/Weakened+Witness/EquivalentInScope` schema_diff.rs:55–175 — accrete = covariant strengthening; weakened-with-witness = contravariant obligation test **executed with counterexamples** (ploeh argues it statically) | embodied-unnamed |
| Set is not a functor (2018-12-03) | schema maps withhold functoriality until Δ executes: model.rs:140, parse/schema_map.rs:7,47; title earned only in migrate.rs (Spivak pullback) | embodied (inverted: ploeh refutes failing claims; catherine withholds claims until executed) |
| Catamorphism series (2019) | kernel walk folds (schema, instance) → findings; `fold_search` boundary.rs:224–228; fold accumulation query.rs:356 | embodied-unnamed |
| State/Reader/IO/async monads | synchronous, total, pure kernel (BTreeMap walks, deterministic sorts) | deliberate absence |
| Church encoding, invariant/contravariant typeclasses | Rust enums + traits make the encodings unnecessary (ploeh's own isomorphism argument) | deliberate absence |

## Sharpest single finding

`EpistemicLabel` (findings.rs:56): `proved`/`illustrated` **do not exist in
the type** — "so the kernel cannot fabricate them". That is the hub post's
ambiguity complaint turned into a type: vocabulary that is total, ordered
(`derive(Ord)`), with illegal claims unrepresentable. The design-level
analogue of the model-level epistemic contract.

## Opportunities captured (docs-only, in docs/ref/ploeh-grounding.md §4)

1. Name the monoid + sort-normalization invariant in checks.rs/findings.rs
   docs (`check(a) ++ check(b) = check(a+b)` holds only post-sort) — this is
   the invariant a compositional agent relies on when running `findings` on
   sub-models.
2. Variance vocabulary in schema-diff docs (accrete = covariant; weaken =
   contravariant obligation test) — links P10 to the known corpus.
3. THEORY.md §9 reading-order line pointing at the design-level corpus.
4. Builder-isomorphism note: `TypedInstance::build` and witness builders
   (bounds.rs) are build-once-total, not monoid-composable — correct choice
   per the isomorphism argument; note it only if witness merging composes later.

## Provenance

- Corpus snapshot: `~/Downloads/ploeh-series/` (129 md + INDEX.md), pulled 2026-11.
- Repo verified at `2c686e4` (post catherine-dvx archive, 0 open beads).
- Deliverable: `docs/ref/ploeh-grounding.md` (session produced it; citations
  line-checked).
- Not yet done: the four §4 opportunities are potential follow-ups — each is
  docs-only; none blocks a release. Ticketing deferred to the user's call.

## Negative results (equally load-bearing)

- **No code changes warranted.** Every correspondence is either already
  embodied or deliberately absent — the series validates catherine's existing
  shapes retroactively rather than prescribing new ones.
- **Danger of level-blurbing flagged**: ploeh's functor is a typeclass API
  obligation; catherine's is `𝒞 → Set` data. The doc's §1 exists to stop
  future readers from conflating "functor" in the two dialects.

