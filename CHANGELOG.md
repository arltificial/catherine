# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
