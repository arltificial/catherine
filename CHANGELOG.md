# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0/).

## [Unreleased]

## [0.4.1] - 2026-11-24

### Added

- Kani bounded-verification lane scaffolding (`add-kani-verification-lane`,
  `catherine-xdi`): committed release pin (`kani-version.toml`, Kani
  0.68.0), `just verify` recipe + `scripts/check-kani-lane.sh` (pin check,
  per-harness three-valued outcomes, 300 s wall budget, undetermined
  fails without a recorded disposition), eleven partitioned
  `#[cfg(kani)]` harnesses over the kernel walk (panic-freedom per check
  family, sortedness, five witness-soundness obligations re-evaluated
  independently of the kernel), provocation machinery
  (`scripts/provoke-kani-harnesses.sh`), and a nightly verify workflow.
  Deliberately outside `just ci`. Stop-gate verdict at scaffolding time: hold
  (`ledger/kani/0001-8f4caa62.json`) — Kani 0.68's String-keyed
  BTreeMap model makes the proofs undetermined on this hardware
  (evidence + dispositions in `docs/kani-lane.md`); revisit triggers
  recorded. Superseded within this release — see Changed.
- Lean assurance-twin pullback model (`add-lean-kernel-model`): the Lean
  twin models the format-1.6 pullback square and its
  `commutes`/`covers`/`unique` verdicts (the `commutes` witness is
  extracted by the soundness theorem), so format-1.6 documents join the
  fixture-corpus parity oracle. Development artifact: no runtime
  dependency, no shipped binary change.

### Changed

- Interned execution core (`catherine-6g2`): `checks::check` and
  `TypedInstance::build` route through a new interned core
  (`engine/{compile,families,lane}`) over dense canonical ids; the
  parsed-path family walkers are deleted and findings are ordered by
  pure-int sort keys exactly equivalent to the previous `Finding::cmp`.
  Parsed behavior unchanged and pinned (169 core tests, contract suite,
  fixtures, parity). By release the lane advances from the hold verdict
  to 14 of 18 harnesses fully proved at the 8 GB / 300 s standard
  (ledger seals 0006–0010): the equation family proves (unwind 8, the
  sort relocated, the claim split per side), then the key and coproduct
  families (flat concrete-index slot scan, membership-only coproduct
  image, per-witness-kind dispatch, sized unwind); the remaining four
  stay held at the propositional-reduction memory wall
  (`docs/kani-lane.md`).
- Lane sources and engine shape (`catherine-6g2`): the Kani lane sources
  move onto the engine and harness crates; the family walkers consume a
  `FindingSink` instead of a `Vec` plus `push_bounded`, arena the build
  path's symbolic tables behind bounded temporaries, resolve equation
  paths as bounded tables (concrete-bound resolution walk), and the
  coproduct soundness obligations split into
  `catherine-kani/src/soundness/coprod.rs`; scope-TERM budget kills are
  classified as undetermined.

### Fixed

- `cfg(kani)` modules are registered via check-cfg lints, so
  `cargo clippy -D warnings` passes on every checkout (`catherine-6g2`).

## [0.4.0] - 2026-11-18

### Added

- Pullback checks (`add-pullback-checks`, `catherine-4vs`): the additive
  `schema.pullbacks` section (format 1.6) declares a square
  `{"apex": P, "legs": [p1, p2], "cospan": [f, g], "status": …}` where
  the legs source at the apex, each leg targets its paired cospan
  arrow's source, and the cospan arrows share a target.
  Accepted declarations execute as three verdicts: `commutes`
  (`f∘p1 = g∘p2` on every apex row), `covers` (every fiber pair over a
  common cospan target is projected by some apex row), and `unique` (no
  two apex rows project to the same pair). Each offending row or pair is
  one `violation`/`executed`, carrying `{pullback, row, lhs, rhs}` for a
  broken square, `{pullback, pair}` for an uncovered pair, or
  `{pullback, rows}` for duplicate projections. `catherine check` exits 3
  on a violating square and 0 on a clean one (including a zero-row apex
  over an empty fiber). `proposed` declarations fire `hypothesized` and
  never judge satisfiability; status defaults to accepted. Malformed
  declarations fail `invalid_pullback` at their node — an undeclared
  object or arrow, a leg not sourced at the apex, a leg whose target is
  not its paired cospan arrow's source, cospan targets that differ, a
  leg or cospan list whose length is not two, or a repeated arrow name.
  An empty or absent section is hash-neutral, so every pre-existing
  document keeps its hash, and the schema-diff enforced set now
  includes accepted pullbacks.
  `schema.cones` cannot be declared (`pullback-absence`): it still fails
  `unknown_field`, and no general limit-cone machinery is introduced.
  Declaration surface: `catherine-1py`; executed checks: `catherine-a11`.

### Fixed

- The parser rejects a field the accepted format version does not define
  with a labeled `unknown_field` error at the field's line
  (`catherine-bak`): serde `deny_unknown_fields` on every raw document
  struct plus a per-kind predicate field whitelist. Unknown-key tolerance
  was the documented additive-minor policy, but an unknown minor already
  fails the version gate, so the only thing tolerance bought was typo
  silence — a misspelled `equations` key dropped the equation and
  reported a violated model as clean at exit 0 (external black-box
  probe). The instance-shape gate and its tolerance tests are gone: an
  unknown instance key is now rejected by name.
- The findings tables in `docs/USING.md`, `docs/DESIGN.md` and
  `docs/THEORY.md` define `dangling` as a partial map — a declared arrow
  with no value at a source row — and name the kind `redundant`; an arrow
  naming a nonexistent row is a parse error, not a finding
  (`catherine-f9z`).
- `README.md` scopes `catherine check`'s findings to the execution
  classes and points at the analysis subcommands (`catherine-4pc`).

### Changed

- `catherine redundant` reports one finding per derivable declaration, so
  an equation declared twice is named twice (`catherine-1lf`). The
  behavior was already pinned by the builder-monoid fault kata; it is now
  documented as intended in the subcommand help, `docs/USING.md`, the
  analysis README, and a CLI regression test instead of reading as
  duplicate output
- Olog label surface decided (`catherine-c65`): the model document keeps
  no label field in v1. English stays in equation `rationale` and in
  prose; the reopen triggers (round-trip rendering, agent
  disambiguation, annotation-drift detection, an E-license) and the
  preferred presentation-only, hash-excluded shape are recorded in
  `docs/ref/olog-mapping.md`.
- Property-based tests run on `hegeltest` (exact-pinned `=0.53.1`) instead
  of `proptest` (`catherine-7ko`): the round-trip, symmetric-ratchet,
  executed-checks, findings-taxonomy, pullback order-stability and analysis
  parity suites were ported property-for-property, with one shared
  `#[hegel::composite]` model generator in
  `crates/catherine-core/tests/common/generators.rs` and workspace
  `hegel.toml` profiles (development 100 cases, CI 200). Each port is proven
  wired by a seeded product mutation that the ported harness caught and
  shrank before the old file was deleted. The dead `proptest`
  dev-dependency is dropped from `catherine-core`, `catherine-analysis` and
  `catherine-ledger`; budgets and seeds come from `hegel.toml` and the
  `HEGEL_*` environment variables.

## [0.3.0] - 2026-11-17

### Added

- crates.io publication (`catherine-qmc`): a release tag publishes
  `catherine-core`, `catherine-analysis`, `catherine-ledger` and
  `catherine-cli` in dependency order (`catherine-wasm` stays out — the
  wasm boundary is consumed through the web bundle). A crate whose version
  already exists is skipped, so a partial publish is retried by re-running
  the tag. Intra-workspace dependencies carry the workspace version, and
  the release assertion fails when the tag, a package version or a sibling
  requirement disagree.
- Lean 4 assurance twin (`catherine-xvh`, `add-lean-kernel-model`): an
  external reference semantics of the kernel with machine-checked
  soundness, equation-completeness and dangling-discipline proofs, a
  fixture-parity oracle against the Rust check, and an axiom audit
  (`propext`, `Classical.choice`, `Quot.sound` only) wired into CI. The
  twin also models the pullback square's commutes/covers/unique verdicts
  (commutes witness extracted by the soundness theorem), so format-1.6
  documents join the parity corpus. Development artifact: no runtime
  dependency and no shipped binary change.
- Ledger decision entries (`catherine-xvh`): the ledger records the
  hold/extend/retire stop-gate decision as a sealed entry kind.

### Changed

- Release notes come from this changelog (`catherine-ply`): the publish job
  extracts the version's section and appends a compare link; a missing or
  empty section fails the job with the version named instead of publishing
  a blank body.

### Fixed

- Canonical relabeling no longer explores equivalent individualization
  branches for rows that are interchangeable under a transposition
  automorphism (`catherine-2f6`): a model with several objects of
  indistinguishable rows used to multiply the search and hang `model_hash`.
  The chosen labeling is unchanged; the pruning only skips branches that
  serialize identically.
- The deployed `kernel-checks` spec restates the coverage-suppression
  requirement so its SHALL leads the requirement text; strict validation of
  the whole spec set is green again.

## [0.2.0] - 2026-11-16

### Added

- Coproduct exhaustiveness (`add-coproduct-exhaustiveness`, `catherine-iib`):
  the additive `schema.coproducts` section (format 1.5) declares a closed
  variant set `{"object": C, "arrows": [V_k → C, …], "status": …}` — the
  same grammar as `keys`, asserting coverage and disjointness of C's rows
  over the injection images. Accepted declarations execute as constraints:
  a row outside every injection is one `violation`/`executed` naming the
  row, a row in two images one naming the row and both injections;
  `proposed` declarations fire `hypothesized`. Malformed declarations fail
  `invalid_coproduct` at their node, an empty/absent section is
  hash-neutral, and the schema-diff enforced set now includes accepted
  coproducts (add → `accreted`, drop → `weakened` with a rogue-variant
  witness). Probe: `ledger/probes/coproduct/` recorded the pre-registered
  catalog before the check landed; the executed run reports `survived`
  with two unique detections (uncovered `voucher`, dual-covered `shared`)
  against an executed baseline that never reported them.
- Joint check at a seam (`add-model-composition`, `catherine-0qz`):
  `catherine check`/`findings` accept two or more documents, gluing them by
  shared declared name (objects, arrows, rows) and executing every input's
  equations and keys over the merged instance; `underconstraint` likewise
  accepts the joint form for seam-only parallel-path questions. Overlapping
  declarations are claims and must agree — disagreeing arrow endpoints are a
  labeled `schema_conflict`; differing row sets on a shared object, or a
  commonly-declared cell valued differently (including `null` against a
  value), a `row_conflict`; both name the documents and node; and a glue
  sharing no object, arrow, or `(object, row)` pair warns
  `empty_composition_interface` on the envelope and on stderr instead of
  passing silently. Cross-document equations live in an input document
  (versioned, reviewed, enforced evidence) and resolve post-glue with the
  same resolver the parser uses, with spans surviving, so a document that
  resolves alone reports the same first error at any arity and
  single-document output stays byte-identical; a claim naming a sibling's
  arrows is joint-run evidence, and only the self-contained form is
  standalone-parseable (hence model-hashable and diffable). Findings gain an
  additive `source` attribution on multi-document runs only. Composition
  produces findings, never a merged document. Probe:
  `ledger/probes/composition/` records the executed hand-merged baseline
  (parity; merge-step elimination named as a gap).
- Instance morphism check (`add-instance-morphism`, `catherine-s60`): the
  additive `instance_map` section (format 1.4) declares a claimed row
  mapping from the document's instance to another instance of the same
  schema — hashed model evidence, omitted from canonical bytes when empty,
  so pre-1.4 hashes and sealed ledger `model_hashes` are untouched. Totality
  is enforced at parse (`incomplete_instance_map`), a cell naming a missing
  target row is a construction-time `unknown_map_target` error, and
  `catherine naturality <src> <tgt>` executes every schema arrow against
  every mapped row: a non-commuting square is one `violation`/`executed`
  naming the arrow, source row, and both composed values (no new finding
  kinds). The surface verifies a given map only — no synthesis, no search.
  Probe: `ledger/probes/naturality/` records the executed `distinguish`
  baseline (which answers "instances differ", never "this map cell is
  incoherent").
- Expressibility gap suite (`add-expressibility-gap-suite`, `catherine-j6g`):
  `docs/ref/expressibility-dispositions.md` is the registry of the twelve
  known expressibility limitations, each row pairing a corpus-sourced claim
  (`identity-empty-path-equations`, `optional-references`,
  `unknown-top-level-fields`, `declared-cones-ignored`,
  `non-functorial-schema-map`, `dropped-equations-on-migrate`,
  `vacuous-clean-instances`, `distinguish-observation-scope`,
  `associativity-scaffolding`, `composition-absence`, `naturality-absence`,
  `coproduct-absence`) with an executable CLI case under
  `crates/catherine-cli/tests/expressibility_gaps/`. Each row's `gap` or
  `closed` disposition is asserted in both directions — a fixed limitation
  fails until its row flips, a regression fails until it flips back — and
  `just check-gap-claims` (inside `just ci`) rejects a limitation claim in
  README, CHANGELOG, `docs/**` or an openspec close note that names no row
  id. Rows register verified-at-HEAD behavior; the change that fixes a
  limitation flips its row in the same commit.
- Pattern lenses (`add-pattern-lenses`, `catherine-mx1`): the twelve
  executed rows of `docs/ref/pattern-kata-dispositions.md` ship as
  registered, law-scoped checks (`catherine pattern <model> [--id <pattern>]
  [--values ...] [--list]`). Each lens carries its GoF name, universal
  abstraction, one law kind (`identity | totality | coverage | order |
  delta`), and the disposition row's scope string; the replay meter
  (`crates/catherine-analysis/tests/pattern_replay.rs`) reproduces the
  sealed ledger's per-fault classes and keeps parity rows silent. Findings
  reuse the existing taxonomy (`checks` channel) and the migrate engine's
  `lost_row`/`lost_cell` findings (`migrations` channel, exit 8); an
  unknown id is a labeled `bad_argument` (exit 2); a bounded non-answer is
  `unknown` with its exact scope. No detection-rate claim: coverage-only
  and parity scopes are printed by `--list`.
- Pattern dictionary gaps (`catherine-r5k`): the disposed/unexpressible
  rows were re-opened with named laws and sealed ledger entries —
  Abstract Factory, Factory Method, Facade, Iterator, and Mediator became
  executed-scoped; Flyweight became executed-parity under a key-based
  sharing law (the prior "unexpressible" was a fixture defect); Template
  Method stays a named gap (no override/dispatch predicate); Singleton
  stays unexpressible (missing cardinality/existence predicate). Every
  entry is sealed via `ledger-record` with a committed conventional
  baseline arm.

### Changed

- `docs/ref/pattern-kata-dispositions.md` marks the twelve executed rows
  `implemented`, records the r5k outcomes, and defines the `still gap`
  disposition.

## [0.1.0] - 2026-11-13

### Added

- Declarative keys (step 2 of the post-granularity capability gaps,
  `catherine-2s6`, format 1.3): `schema.keys` declares
  `{"object": O, "arrows": [a, ...], "status": "accepted" | "proposed"}` —
  the arrows' value tuple is injective over `O`'s rows. Accepted keys are
  executed by `check` as constraints (one `violation`/`executed` per
  colliding pair; proposed keys `hypothesized`; rows with a dangling key cell
  are excluded), the `violation` witness becomes an untagged
  equation/key union (`KeyWitness {key, rows}`; the equation encoding is
  byte-identical), and `diff` treats accepted keys as enforced constraints
  (added → `accreted`, dropped → `weakened` with the key witness). Absent or
  empty `keys` is omitted from canonical bytes, so every pre-1.3 document
  keeps its hash and sealed ledger entries stay valid; `KNOWN_MINOR` is 3.
  Keys range over arrows only: a column-level FD needs the value-object
  convention or `rules --project` until attribute columns arrive (format
  1.4). New parse code `invalid_key`.

- Rule-table projection bridge (step 1 of the post-granularity capability
  gaps, `catherine-x94`): `catherine rules <model> --project <Object>
  --columns <arrow,...> [--action <arrow>] [--values ...]` synthesizes a
  `rule_presentation` from an object's instance rows — one rule per row in
  dense order, string-equality column predicates over the arrows' target
  row names, identity or `--action` actions, fixed `DeclaredDisjoint` — and
  analyzes it with the shipped rule lens (no new finding kind, no format
  change, `KNOWN_MINOR` stays 2). The domain is the distinct observed
  column tuples, optionally unioned with a `--values` product declaration;
  a combination no row has stays silent unless it is declared (a declared
  one is a genuine `underconstraint` gap). Refusals are labeled:
  `bad_argument` (the document already carries a `rule_presentation`, an
  unknown or not-sourced arrow, a malformed column list),
  `projection_incomplete` (a null projected cell, naming the row and the
  arrow), `empty_projection`. The projected claim is outside the model hash
  and the ledger — evidence about a projection, never sealed model
  evidence.

- Ledger science hardening (P11, `catherine-1od`, task 2): any ledger
  entry — kata or probe — that records non-empty `unique_detections`
  must carry `comparison.baseline` naming a committed artifact with
  `mode: executed`; a cited-only arm cannot support `survived` (the
  ceiling is `inconclusive`), refused as `UniqueWithoutBaseline` /
  `CitedBaselineCannotSurvive`, and `ledger-record` refuses the same
  shapes at write time. Supersession is now a usable correction path
  after a rollup: superseded entries stop being effective, claim-bearing
  rules evaluate effective entries only, and the effective rollup (last
  entry) must equal the effective kata id set. `ledger/rule-lens/0002`
  was corrected by appended entries `0004`/`0005`, so its five unique
  detections are backed by the executed
  `prototype/rule_lens_baseline.py` arm (7/12 faults caught) rather than
  prose.

- Domain-probe catalogs (P11, `catherine-1od`, tasks 3–4): probe
  expectations are pre-registered in `catherine_ledger::catalog`
  (`probe-migration`: lost-data, non-invertibility, floor-visible-loss;
  `probe-rule-lens-extracted`: gap, overlap, shadow, derivable,
  floor-near-trivial) and content-addressed by `probe_catalog_hash`. A
  kata record carries `catalog_hash`; the validator refuses a mismatch
  or a registered probe without its pre-registration
  (`CatalogHashMismatch`), so editing a catalog or entry after sealing
  forces a superseding entry. `harness::derive_probe_verdict` derives
  survived / inconclusive / killed from the recorded comparison and
  returns no verdict for an unregistered probe or an arm-less
  uniqueness claim. Round-trip tests cover the green fixture probe
  ledger, the full task-1 rejection matrix on mutated ledgers, and the
  sealed kata rollup's byte-identity under probe recording.

- Schema diff (P10, `catherine-dvx`, LICENSED E4): the
  `catherine-analysis::schema_diff` classification of a v1→v2 model
  schema edit over the **enforced** equations (accepted `(lhs, rhs)`
  pairs; rationale, document order, and proposed equations are
  presentation). `cosmetic` (presentation-only) and `accreted`
  (constraints only added) emit zero findings; `weakened` emits exactly
  one existing-taxonomy `violation`/`executed` finding with a witness
  instance v2 accepts and v1 rejects; `equivalent-in-scope` is proven
  only when the bounded enumeration exhausts with no separating instance
  (scope echoed); an expired budget is `unknown`, and differing
  objects/arrows are `incomparable` (needs a declared schema map, never
  a guess). The bounded enumeration is the new shared
  `search::enumerate` engine (found / exhausted-with-scope /
  unknown-with-scope) that also backs `search`, so candidate order and
  budget semantics cannot drift. CLI: `catherine diff <v1> <v2>
  [--rows OBJECT=COUNT] [--max-candidates N]` — weakened exits 3 through
  the existing class table, completed classifications exit 0 with the
  outcome in the payload, incomparable is the labeled
  `schema_incomparable` error. Fixtures `fixtures/schema_diff/`
  (weakened/refactor/cosmetic pairs), tests `tests/schema_diff.rs` and
  `tests/cli_diff.rs`, and the pre-registered validation record in
  `openspec/changes/add-schema-diff/tasks.md` (weakening caught with
  witness, refactor proven, cosmetic silent, budget expiry `unknown`).
  No kernel change — kernel delta 0 LOC.

- Rule lens pre-registered validation (P8, `catherine-nq5`) — **verdict:
  survived-scoped**. 12-fault matrix on a 240-input above-floor kata
  (16 compound rules, PriorityOrder): conventional 45-assertion
  baseline 7/12, lens 12/12, **5 unique detections — all
  behavior-preserving structural redundancy** (shadowed/derivable
  rules; the caveat is recorded in the ledger entry, never rounded into
  a behavioral win). Near-floor control self-negated (killed, 0/3
  unique). Rule-lens ledger `ledger/rule-lens/` (control → kata →
  derived rollup) green; durable derivation test added.
- Rule lens phase C (P8, `catherine-nq5`, HYPOTHESIZED): discovery
  support and the CLI surface. `catherine-analysis::rule_lens::discover`
  checks a candidate table against declared input→output examples —
  strict per-example vocabulary validation, then a sample-scoped report
  (`examples_checked`; never an exhaustiveness claim): uncovered examples
  aggregate into one `underconstraint`/`hypothesized` `UncoveredInputs`
  finding (dense example indices), while examples whose induced action
  differs (or is ambiguous under an undeclared policy) become
  `violation`/`executed` findings witnessed by the example. No new
  finding variants. `catherine rules <doc> --values column=v1,v2,...
  [--max-candidates N]` runs the structural producer over the declared
  finite domain: the genesis findings envelope (`data.findings` verbatim;
  budget expiry is `outcome: "unknown"` + the exact scope, exit 0), one
  human line per finding, and the existing class exits (clean 0, gap 6,
  overlap under `DeclaredDisjoint` 3, redundancy 7, min over multiples);
  a document without `rule_presentation` is the labeled
  `no_rule_presentation` error (exit 1). `tests/cli_rules.rs` and the
  negative matrix carry the surface's acceptance rows.

- Rule lens phase B (P8, `catherine-nq5`, HYPOTHESIZED): the semantic
  kernel in `catherine-core::rule_lens` — a validated finite `RuleDomain`
  over the declared typed columns, predicate evaluation (`mod` as a
  non-negative remainder, byte-wise strings), R_i regions, effective
  regions E_i = R_i \ ⋃_{j<i} R_j under `PriorityOrder`/`FirstMatch`
  (unchanged regions under `DeclaredDisjoint`, absent under no policy),
  gap/overlap/shadow/derivable detection, budget-bounded enumeration
  (`UNKNOWN { scope }` on expiry, empty scope for a zero budget), and
  rule-table equivalence with a first-differing-input witness. Sim/kernel
  parity is pinned by the oracle's seven analysis and four equivalence
  cases as Rust tests. `catherine-analysis::rule_lens` maps the four
  classes onto the existing taxonomy — gap → underconstraint
  (`hypothesized`, the reserved `UncoveredInputs` arm), overlap under
  `DeclaredDisjoint` → violation (`executed`), shadow/derivable →
  redundant (`executed`) — with no new finding variants; priority
  policies and absent policies emit no overlap finding. The
  pre-registered validation katas remain later phases.

- Rule lens phase A (P8, `catherine-nq5`, HYPOTHESIZED): the additive
  `format_version` 1.2 `rule_presentation` section in `catherine-core` —
  input domain X as typed attribute columns (`int`/`bool`/`string`,
  byte-wise string order), rules with a tiny decidable predicate language
  (typed comparisons, `mod`, `and`/`or`/`not`) and a required non-empty
  action, and an explicit `PriorityOrder`/`FirstMatch`/`DeclaredDisjoint`
  policy that is never defaulted; six labeled parse codes
  (`undeclared_rule_domain`, `invalid_column_type`, `unknown_column`,
  `invalid_predicate`, `invalid_policy`, `missing_action`); the section
  is skip-serialized when absent, so every pre-rule document's
  `model_hash` is pinned unchanged (the hash-stability guard);
  `KNOWN_MINOR` raised 1→2 per convention; ct-schema-v1.json +
  regenerated TS types carry the syntactic surface. The semantics land
  first as a self-checking hand-computed oracle
  (`prototype/rule_lens_sim.py`: effective regions, gap/overlap/shadow/
  derivable, equivalence with witness) plus four golden fixtures
  (`fixtures/rule_lens/`, one per bug class, mapped in its README onto
  the existing finding taxonomy — no new variants). Analysis/findings
  producers, CLI surface, and the pre-registered validation katas are
  later phases.

- Δ∘Δ′ non-invertibility proof (P7, `catherine-alr` task 6): element-
  pair return identity per the migration spec scenario — a kept source
  (object, row) whose candidate-inverse return differs (or is missing)
  is the exhibited offender, with the sibling sharing its forward image
  named; per-object forward images avoid cross-object row-id collisions;
  identity round-trips prove clean (`None`). The ignored round-trip test
  is un-ignored and passing alongside its new negative regression.
- grounded-instance harness (P9 adjunct, `catherine-ga7`, bogart-side):
  spec-driven corpus→`.ct.json` extraction with a provenance sidecar
  (byte-identical parity with the hc6 pilot model), a quantified
  seeded-defect must-gate (7/7 finding-bearing detections with corpus
  witnesses, 2/2 negative controls clean), a real-data
  contradiction-with-data demo (row `pg_70953`, exit 3), and the
  extraction contract in bogart's README. Zero catherine kernel delta —
  provenance stays bogart-side.
- `catherine migrate` subcommand (P7 phase C, `catherine-alr`): envelope
  output with the migrated-instance row counts, the loud lost-data
  findings (`executed`), and the row correspondence; exit 8 on lost
  data / 0 lossless, consistent across human and JSON modes;
  negative-matrix rows for the new surface; bogart raw→serving
  rehearsal fixture (10 findings, verified live) as the P9 dry run.
- Δ migration axis phase A (P7, `catherine-alr`): total schema maps as
  the additive `format_version` 1.1 section (`crates/catherine-core`) —
  a partial map is the labeled `partial_schema_map` build error naming
  the unmapped element; the section is skip-serialized when absent, so
  every pre-map document's `model_hash` is pinned unchanged (the
  hash-stability guard); `partial_schema_map` joins the closed error
  codes with its CLI remediation hint; ct-schema-v1.json + regenerated
  TS types carry the syntactic surface; three RED migration fixtures
  (dropped object, field narrowing, non-invertible round-trip) pin the
  phase-B lost-data expectations. KNOWN_MINOR raised 0→1 per
  convention. Execution (pullback reindex, lost-data findings, `catherine
  migrate`) is phase B/C.
- kata validation ledger (P6, `catherine-9vv`) — THE PROJECT STOP
  CONDITION, executed: `crates/catherine-ledger` (content-addressed
  entries via the P0 hash port, validator refusing bad science —
  control-first ordering, closed verdict vocabulary, model-hash
  references, append-only supersession — recorded-procedure rollup,
  pre-registered per-kata expectation catalog, fixture harness,
  `ledger-check`/`ledger-record` bins), 4 kata fixtures with executed
  runs (`fixtures/katas/`), ledger entries #1–#4 + derived rollup #5:
  **project verdict survived (scoped)** — FizzBuzz control self-negated
  (killed, zero unique detections), Bowling/Gilded Rose/Vending
  survived with named expressibility gaps and the Vending "6/6"
  proposal-vs-series discrepancy recorded; kata presets wired into the
  web view (load→check→render verified headless); P7/P8 unblock per
  the derived verdict.
- Δ migration CLI + rehearsal fixture (P7 phase C, `catherine-alr`
  tasks 7–8): `catherine migrate <doc.ct.json>` reindexes along the
  document's total schema map and emits the genesis-vibes success
  envelope with `data.rows` (target rows per object), `data.lost` (the
  kernel `MigrationFinding` encoding verbatim) and `data.row_map` (the
  kernel correspondence verbatim); lost data is a finding-bearing run
  through the exit table's one additive extension — class 8, lost data
  (lowest severity, `migrate` only; shipped numerals unchanged) — a
  lossless migration exits 0, and a document without a `schema_map` is
  the labeled `no_schema_map` error (never an empty migration);
  negative-matrix rows cover the new surface (partial map →
  `partial_schema_map`, absent map, garbage instance rows, bad
  version, unknown flag) and the variant-coverage ratchet extends to
  `MigrationFindingKind`; bogart rehearsal fixture
  (`fixtures/migration/bogart_rehearsal.ct.json`): raw corpus →
  serving projection with deliberate drops (4 lost rows + 6 lost
  cells, documented in `fixtures/migration/README.md`) — the P9 dry
  run. Kernel `migrate.rs` logic unchanged this phase (task 9: the
  shared schema-map validation is the parser's alone and the finding
  taxonomies share `EpistemicLabel` only — no shape worth unifying).
- bogart pilot corpus (P9, `catherine-hc6`): models 1–3 + RED fixture
  landed in `at/bogart` (FTS-anchor equation/totality, identity-domain
  disjointness — vacuous-by-design until P8, capture-selector
  relation), bogart CI gating on `catherine check` at pinned rev
  `268c142` with class exit codes, live-corpus run triaged 69/69 (2
  real defects filed as bogart `bogart-4wt`; 67 noise accounted —
  planner-intent granularity, not suppressed), pilot report + evidence
  in bogart docs. Catherine kernel unchanged.
- vampiro seam-composition gate (P2, `catherine-w6z`): `vampiro check
  -p crates --mode gate --severity-threshold high` wired into lefthook
  pre-push (complementary drift gate; does not replace
  catherine-348/za6/3zs), tool pinned `0.5.0` in `versions.ddl.toml`;
  the B2 law-axis false-negative (shape-valid cross-origin
  `TypedInstance::build` join, catherine-18t) documented and filed
  upstream as feedback (body + repro hash preserved under
  `.wai/projects/catherine/research/` — upstream slug unresolvable by
  `gh`, manual filing URL recorded).
- variant-coverage ratchet (P2, `catherine-3zs`): a workspace test
  driving the real `catherine` binary over the fixture corpus asserting
  every variant of the contract enums (`EpistemicLabel`, `FindingKind`,
  `DomainQuestion`, `ConstraintStatus`, `SearchOutcome`) is constructed
  by at least one executed fixture — exhaustive match functions make a
  new variant break the ratchet until it is accounted for; the two
  producer-less variants (`UncoveredInputs` P8-reserved,
  `Unknown` render-reserved) are covered by cited direct construction.
  Advisory diff-coverage block added to lefthook pre-push
  (report-only; the ~80% gate is a later phase).
- symmetric-structure property ratchet (P2, `catherine-1ml`): the
  WL-pruning assumption behind the canonicalizer's refined-partition
  branch skip now carries executable evidence — a proptest strategy
  biased toward automorphism-rich models (rotation cycles, identity-pair
  involutions, twin rows, full collapse) asserting `model_hash`
  invariance under row-presentation permutation.
- wasm boundary + minimal web view (P5, `catherine-9b5`): `crates/catherine-wasm`
  (`check`/`analyze` JSON in/out over the kernel's own findings encoding,
  512 KiB document cap enforced before the kernel is invoked with a typed
  `code`-tagged error, `analyze` reports the search outcome discriminator —
  witness findings never collapse into silence); `web/` static site (no
  framework, no server): JSON-textarea document, file load/save, SVG schema
  graph, findings panel rendering kind + epistemic label + witness; TS types
  generated from `ct-schema-v1.json`; wasm32 compile-only gate in `just ci`
  and CI; parity test pins browser and terminal to identical finding
  encodings across the fixture corpus (19 fixtures).
- `docs/THEORY.md` — theory documentation: each categorical construction
  (schemas, instances/functors, path equations, findings taxonomy,
  migration, rule lens), what it buys, where validated, reading order
  (Johnson → Spivak → AlgebraicJulia), glossary.
- README: project overview, deliverables manifest, docs map, roadmap,
  tooling guide.
- Rule lens (P8, `catherine-nq5`): rules as subobjects + resolution
  policy; kata-6 boundary reframed as a triviality-floor claim;
  pre-registered two-kata validation.
- Bogart pilot corpus (P9, `catherine-hc6`): openspec prose invariants →
  executed findings; genesis-vibes adopted for CLI plumbing (P3).
- `catherine` CLI (P3, `catherine-86i`): `catherine check` and
  `catherine findings` — one execution path over `.ct.json` model
  documents, findings printed in the DESIGN §5 format (human) or the
  genesis-vibes JSON envelope (`data.findings`, the kernel encoding
  verbatim); labeled errors with line/column, stable code, and
  remediation hint on stderr; exit codes mirror finding classes
  (clean 0, parse/build 1, usage 2, violation 3, dangling 4,
  contradiction 5, underconstraint 6, redundant 7 — multi-class runs
  exit with the minimum class code); genesis-vibes consumed by the CLI
  crate only — the kernel stays genesis-free.

### Changed

- Arrow-granularity rule re-scoped after its second execution
  (`catherine-8v9`): a storage-granularity model of bogart's FTS anchor
  (2 arrows with named storing columns, 0 equations) produces **zero**
  findings over the 289-page slice and the full 74,257-page corpus — the
  same model class that excludes the pilot's 67 noise findings also
  excludes its 2 real `bogart-4wt` ambiguity defects, because no table
  stores a selector→capture column (live census: 1,441 selectors, 3
  ambiguous). The rule therefore governs *structural* claims only;
  derivation/selection claims must be declared explicitly (rule lens,
  P8). Status stays PROPOSED; `docs/USING.md` and
  `docs/ref/ct_methodology_playbook.md` §2 carry the scope limit.
  Evidence: `fixtures/probes/granularity-bogart/` +
  `prototype/bogart_storage_granularity.py`.
- Specs deployed: the 10 completed openspec changes are archived into
  `openspec/specs/` (9 capabilities) and every deployed scenario is bound
  to an executable contract under `.espectacular/<capability>/` — 52/52
  covered, so `ah check` now checks something (it previously had zero
  deployed specs and was vacuous). All 79 contract test entries pass.
  Two scenarios had no verifying test and gained one: kernel dependency
  purity (`crates/catherine-wasm/tests/purity.rs`) and the web save path
  (`web/src/save.ts` + `web/test/save.test.ts`, with a `vitest` runner in
  `.espectacular/config.toml`). `add-bogart-pilot` remains an open change
  (models 4–5 unimplemented).
- License: MIT → Apache-2.0 (matches code donors specodelic/wild/genesis;
  switched before any code landed).
- underconstraint: the direct-only `related()` limit — transitive accepted
  chains do not suppress the pair question — is documented (module header +
  analysis README) and pinned by a fixture test (`catherine-7e8`; adjudicated:
  v1 question-raiser semantics kept, congruence closure deferred as future
  capability).

### Fixed

- rewrite.rs scan-order docstring corrected: the position-then-rule scan is
  leftmost-outermost, not leftmost-innermost (`catherine-mrq`; docs-only,
  behavior unchanged).

### Added (initial baseline)

- ddl-managed tooling baseline: lefthook gates (pre-commit: `ah check`,
  pretender gate, `spk lint`; pre-push: `ah check`), beads no-db tracked
  export, wai workspace, espectacular (`ah`) config, openspec skeleton.
- whisper/turu knowledge workspace (repo-scope tracked; `.whisper/private/`
  gitignored — machine-local, never pushed).
- `docs/DESIGN.md` — CT modeling workbench design (Rule-of-5 converged),
  vendored methodology reference in `docs/ref/`.
- Beads tickets `catherine-bpz..alr` — phases P0–P7 with dependency chain;
  P6 is the project stop condition.
