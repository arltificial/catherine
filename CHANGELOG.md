# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- ddl-managed tooling baseline: lefthook gates (pre-commit: `ah check`,
  pretender gate, `spk lint`; pre-push: `ah check`), beads no-db tracked
  export, wai workspace, espectacular (`ah`) config, openspec skeleton.
- whisper/turu knowledge workspace (repo-scope tracked; `.whisper/private/`
  gitignored — machine-local, never pushed).
- `docs/DESIGN.md` — CT modeling workbench design (Rule-of-5 converged),
  vendored methodology reference in `docs/ref/`.
- Beads tickets `catherine-bpz..alr` — phases P0–P7 with dependency chain;
  P6 is the project stop condition.
