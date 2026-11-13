# Grounding: the ploeh "From design patterns to category theory" series against catherine

Audience: maintainers and agents who know one side and want the other mapped.
Source corpus: Mark Seemann's article series hub
<https://blog.ploeh.dk/2017/10/04/from-design-patterns-to-category-theory/>
(129 articles, 2017–2025, local snapshot: `~/Downloads/ploeh-series/`,
articles under `md/`, index in `INDEX.md`). Companion docs: `THEORY.md`
(the mathematics), `DESIGN.md` (the build).

## 1. Two different levels of the same thesis

The ploeh series and catherine are *not* the same kind of object, and the
grounding must not blur them:

- **ploeh grounds *programming API design* in algebra** — universal
  abstractions (monoids, functors, monads) as contracts for library and
  application code. His functors are typeclass-shaped API obligations;
  the laws exist to remove ambiguity from design patterns.
- **catherine implements *the mathematics itself*** — schemas as
  finitely-presented categories, instances as functors `𝒞 → Set`,
  migrations as Spivak's Δ pullback (THEORY.md §2, §3, §6).

But the founding claim is the same sentence in two dialects:

> ploeh: "…category theory concepts are governed by specific laws. … This
> makes the category theory concepts more specific, and less ambiguous,
> than design patterns." (Ellipses mark two cuts; the original has an
> intervening sentence: "In order to be a functor, for example, an object
> must obey certain simple and intuitive laws.")
>
> catherine: "declarations don't enforce; checks do" (THEORY.md §4;
> `checks.rs` purpose block).

Both refuse to credit a claim that has not been executed. The correspondences
below are grounded at file:line; the status vocabulary is
**embodied** (the abstraction is load-bearing in the code),
**embodied-unnamed** (present in shape, never named in docs),
**deliberate absence** (the series concept is intentionally not used),
or **opportunity** (vocabulary transfer that would sharpen docs or design) —
headings may add a parenthetical qualifier where the base status needs one.
Line numbers are valid at the pinned commit (§5); the symbol names are the
stable reference.

## 2. Grounded correspondences

Coverage: the entries below are the correspondences load-bearing in the
kernel; isomorphism-series articles are mapped only where they appear
inline (§2.5, §2.6, §4.4) — the series' ~100 further articles have no
distinct catherine counterpart to report.

### 2.1 Composite as a monoid — **embodied**

- ploeh: *Composite as a monoid* (2018-03-12): composites are monoids
  because adding leaves is associative with an empty composite as identity.
- catherine: `check(schema, instance) -> Vec<Finding>`
  (`crates/catherine-core/src/checks.rs:47`). Findings accumulate by
  concatenation; the empty result is real: "an empty schema or an empty
  instance yields zero findings — silence after evaluation is real success,
  not an unevaluated declaration" (`checks.rs:15–17`). The free-monoid result
  is *normalized* — `findings.sort()` under the taxonomy's total order
  (`checks.rs:66`, order contract in `findings.rs` module docs) — so
  grouping and walk order never leak into the observable output: repeated
  checks serialize byte-identically.

### 2.2 Null Object as identity — **embodied**

- ploeh: *Null Object as identity* (2018-04-23).
- catherine: the identity elements are explicit and typed: the empty model
  document `Model::empty()` (`model.rs:337`, openspec scenario "empty
  schema + empty instance parses cleanly") and the zero-findings result of
  §2.1. `distinguish` over an observation set with no differing witness
  returns `EquivalentInScope` — "in scope, and no claim beyond it"
  (`crates/catherine-analysis/src/distinguish.rs:42–56`): the
  zero-difference result, never a global equivalence claim.

### 2.3 Kleisli composition for Maybe — **embodied-unnamed**

- ploeh: *Kleisli composition*, *The Maybe monad* (2022-04): functions
  `a → Maybe b` compose by propagating the first `Nothing`.
- catherine: path composition is exactly this shape. "A dangling value
  short-circuits path composition at that arrow: the affected row is
  recorded … never reported as a violation, and the run continues"
  (`checks.rs:9–12`, walk at `checks.rs:49`). Each arrow application can
  fail (dangling reference); composition is `Option`-flavoured Kleisli
  chaining, and the failure is itself a typed witness (`DanglingWitness`),
  not a `None` to discard.

### 2.4 Applicative validation vs monadic fail-fast — **embodied (both, deliberately)**

- ploeh: *Applicative validation* (2018-11-05): applicative collects
  *all* errors; monadic bind fails fast. The choice is the whole article.
- catherine uses each at the right layer: **fail-fast (monadic)** inside
  path composition — you cannot compose past a dangling cell (§2.3);
  **collect-all (applicative)** at the check level — one finding per
  equation per applicable row, "never emits a finding without its
  witness", the run continues (module docs, `checks.rs`). This is the
  ploeh distinction, implemented without the vocabulary.

### 2.5 Visitor as a sum type — **embodied**

- ploeh: *Visitor as a sum type* (2018-06-25) and the Church-encoding
  series: the GoF Visitor is an isomorphic encoding of a sum type.
- catherine: every diagnostic is a Rust enum with per-variant witness
  payloads — `FindingKind` (`findings.rs:163–164`,
  `#[serde(tag = "kind", content = "witness")]`),
  `MigrationFindingKind` (`migrate.rs:46–47`), the tagged
  `DomainQuestion` union (`findings.rs:125`). Exhaustive `match` is the
  catamorphism over the sum; serde's tagged encoding is the wire
  isomorph. No Visitor/Church machinery needed — Rust enums are the free
  version (**deliberate absence** of the OOP encoding, per ploeh's own
  isomorphism argument).

### 2.6 Result isomorphism / Tester-Doer — **embodied**

- ploeh: *Result isomorphism* (2025-10-15), *Tester-Doer isomorphisms*
  (2019-07-15): failures as values, never out-of-band queries.
- catherine: "reference integrity as a *value*, not an exception"
  (THEORY.md §5); instance construction returns every parsed arrow value
  in one of three documented places — morphism value, dangling record, or
  a named `TypingViolation` carrying row and arrow — total and
  typing-enforcing at build (`instance.rs:5–13`). Nothing throws; every
  failure shape is a return value with a witness.

### 2.7 Backwards compatibility / LSP as profunctor — **embodied-unnamed**

- ploeh: *Backwards compatibility as a profunctor* (2021-12-13),
  *Liskov Substitution Principle as a profunctor* (2021-12-06),
  *Postel's law as a profunctor* (2021-11-29): contract evolution is
  variance-sensitive — guarantees may extend (covariant), obligations may
  not grow (contravariant).
- catherine: the schema-diff classification is a witness-based variance
  analysis: `Cosmetic` / `Accreted` (v2 enforces strictly more —
  strengthening) / `Weakened` (v2 accepts an instance v1 rejects, with
  the witness instance attached) / `StrengthenedInScope` (mixed edit:
  strengthening witnessed, weakening not, in the exhausted scope) /
  `EquivalentInScope` (no separating instance in the exhausted scope) /
  `Unknown` (budget expired — a scope-bounded non-answer, never an
  equivalence claim)
  (`crates/catherine-analysis/src/schema_diff.rs:55–175`). The last two
  variants are catherine's epistemic discipline inside the diff: scope
  and non-answers are part of the claim. "Accreted" is
  covariant extension; "Weakened-with-witness" is exactly the profunctor
  obligation test — a witness input in the old domain whose output
  violates the new contract. catherine *executes* the variance test with
  counterexamples where ploeh argues it *statically*.

### 2.8 Functor laws: deny the title without the laws — **embodied (inverted emphasis)**

- ploeh: *Set is not a functor* (2018-12-03): `HashSet` must not claim
  `Functor` because it cannot obey the functor laws.
- catherine: the schema-map layer refuses functoriality until execution:
  "no functoriality (path pullback/reindex execution is phase B)"
  (`model.rs:140`); "Declared data only — no functoriality check"
  (`parse/schema_map.rs:7,47`); the functor title is earned only in the
  migration engine (Spivak pullback, `migrate.rs:18`). Same discipline,
  opposite direction: ploeh *refutes* a claim the laws fail; catherine
  *withholds* a claim until the laws are executed — and stamps the
  result with an epistemic label instead of a bare claim.

### 2.9 Catamorphisms (folds) — **embodied-unnamed**

- ploeh: the catamorphism series (2019-04-29 onward): collapse a
  structure into a value by folding, one case at a time.
- catherine folds everywhere without the name: the kernel walk collapses
  (schema, instance) → `Vec<Finding>`; `fold_search` collapses a search
  report into findings plus outcome (`catherine-wasm/src/boundary.rs:224–228`);
  key accumulation folds into `BTreeMap`s
  (`catherine-analysis/src/query.rs:356`). "Derivable" findings are the
  fold-flavoured test: a stored path is redundant iff it is produced by
  composing others (THEORY.md §5).

### 2.10 Monads beyond Maybe (State, Lazy, Async, IO) — **deliberate absence**

- ploeh spends much of the series on State/Reader/IO/async monads.
- catherine's kernel is synchronous, total, and pure: ordered `BTreeMap`
  walks, deterministic sorts (`migrate.rs` determinism note), no hidden
  state threading — the State pattern content maps only at the rule-lens
  level (rules as subobjects over an enumerable domain, effective regions
  `E_i = R_i \ ⋃_{j<i} R_j`, THEORY.md §7), which is closer to ploeh's
  *Chain of Responsibility as catamorphisms* (2019-07-22): first-match
  and priority resolution — two of the three declared policies
  (THEORY.md:165–166) — are folds over the ordered rule list.

### 2.11 Church encoding, invariant/contravariant/profunctor *typeclasses* — **deliberate absence**

- These exist to let *parameterized types* vary while preserving laws in
  OO/FP languages without sum types or HKTs. Rust enums + traits + the
  isomorphism series' own argument make the encodings unnecessary here;
  catherine inherits that conclusion by construction.

## 3. The epistemic-label contract is the series' thesis, type-level

`EpistemicLabel` has exactly three constructible variants — `executed`,
`hypothesized`, `unknown` — and `proved`/`illustrated` *do not exist in
the type*, "so the kernel cannot fabricate them" (`findings.rs:56` and
module docs). This is ploeh's ambiguity complaint turned into a type:
the vocabulary is not just shared (design-pattern style), it is
*total, ordered* (`derive(… PartialOrd, Ord)`), and illegal claims are
unrepresentable. THEORY.md §9 already records the model-level lineage
(thin category/preorder for contract accretion; identity as
canonical-JSON hashing); the ploeh corpus is the *API-design* mirror of
that lineage.

## 4. Opportunities (vocabulary transfer, no behaviour change)

These transfer *vocabulary into existing docs* — none of them licenses a
new abstraction; catherine's anti-overengineering principle (THEORY.md
§8, Johnson's "logic follows from the operations actually needed")
applies to vocabulary too.

1. **Name the algebra where it is load-bearing.** `checks.rs` /
   `findings.rs` doc comments could state: findings accumulate as a
   monoid; the sort is a normalization to canonical form, so
   per-equation batches of the *same* model add additively post-sort
   (each finding belongs to exactly one equation or dangling cell).
   Explicit boundary: this does NOT extend to sub-models — a combined
   model can carry cross-object equations no sub-model sees, so the
   additivity is a walk-order invariant over one model, not a
   model-composition invariant.
2. **Use the variance vocabulary in schema-diff docs.** Accreted =
   covariant strengthening; Weakened-with-witness = the contravariant
   obligation test exhibited by an input. Links P10 to a known corpus and
   makes the asymmetry memorable.
3. **One reading-order line.** THEORY.md §9's reading order covers the
   model-level corpus; a single line noting the design-level mirror
   (ploeh's series: universal abstractions with laws as API contracts)
   would position the two-level thesis for newcomers.

## 5. Snapshot provenance

Series pulled 2026-11 (129 md files: hub + 128 series articles, 2017 →
2025-10 `result-isomorphism`). The series is live; the hub's link list is
the authoritative current shape. Mapping verified against the tree at
`2c686e4` (post catherine-dvx archive).

## 6. Reverse-engineering kata candidates (proposed — HYPOTHESIZED, nothing validated)

The series' pattern↔abstraction dictionary is a hypothesis generator for
reverse-engineering katas: where ploeh proves an equivalence (Composite ≅
monoid, Chain of Responsibility ≅ catamorphism, Visitor ≅ sum type, State
≅ state shape), an existing OOP codebase that *uses* the pattern inherits
an executable check. Gilded Rose already models a fictional legacy system;
these candidates aim that mechanism at real pattern-shaped code:

1. **CoR-as-rule-lens kata.** Target: an OSS middleware/filter chain.
   Model handlers as subobjects over the request domain; the rule lens'
   gap/overlap/shadow findings become the pre-registered fault classes
   (reordered handlers = policy drift, unreachable handler = shadow,
   uncovered request class = gap).
2. **Composite-as-monoid kata.** Target: a composite logger or event
   bus. The equivalence yields executable law checks: identity (a Null
   Object that is not neutral is a bug class) and associativity of
   dispatch (ordering-dependent fan-out is a bug class).
3. **Visitor-as-sum-type kata.** Target: an AST/shape hierarchy in a
   language without exhaustiveness checking; the model checks coverage,
   and a silently-swallowed default arm is the fault class.
4. **Reverse state-machine kata.** Derive the schema *from* the code;
   the discovery mechanism (parallel paths without declared equations →
   "should these agree?") becomes interview questions about undocumented
   legacy invariants — the spec-ambiguity mechanism aimed at
   archaeology.

Validation protocol is the house method: pre-registered hypothesis,
seeded-fault matrix plus natural faults, conventional arm comparison
(parity expected — THEORY.md §8; the claimed edge is structural and
schema-level, not detection rate), LOC overhead, and a near-floor
negative control (a procedural/CRUD codebase where the tool must say
"model conventionally"), verdict survived-scoped or killed, one
kata-validation contract per the existing toml archetypes.

Status: HYPOTHESIZED. Each candidate must be able to fail the way
kata-6 killed the schema-lens rule table — the honest boundary applies
with full force to reverse-engineering claims.
