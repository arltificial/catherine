# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
