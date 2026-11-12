# Brief: catherine-3zs — PREVENTION-C: enum-variant coverage ratchet + advisory diff-coverage pre-push gate

Ticket: P2. Class C recurrence gate (u3k shipped because no fixture exercised `ConstraintStatus::Proposed`).

## Target paths

- `crates/catherine-cli/tests/variant_coverage.rs` — NEW workspace test (cheap tier)
- `lefthook.yml` — pre-push block only, OUTSIDE the `# ddl:managed:start/end` markers (hook-swallow-guard lives outside them on purpose; same rule for you)
- `.wai/resources/reflections/` NO — do not touch reflections. `CHANGELOG.md` gets the entry at orchestrator commit time — skip it.

## Contract

Cheap tier (must land):
- Enumerate the contract enums explicitly in the test: `EpistemicLabel` + `DomainQuestion` + `FindingKind` (`crates/catherine-core/src/findings.rs` :58/:126/:164), `ConstraintStatus` (`crates/catherine-core/src/model.rs` :57), `SearchOutcome` (`crates/catherine-analysis/src/search.rs` — read for the variant list).
- The test asserts every variant of each enum is exercised by the EXECUTED fixture corpus: run the CLI's check/findings/search paths over `crates/catherine-cli/tests/fixtures/` + `fixtures/parity/` (reuse the existing test helpers in `crates/catherine-cli/tests/`), collect produced findings/outcomes, and assert each enum variant appears at least once — with the assertion phrased as "variant X must be constructed in at least one executed fixture" and a failure message naming the missing variant. Enum-variant lists are hand-written const arrays + a compile-time exhaustiveness guard (e.g. a `match` over the enum with no wildcard arm) so a NEW variant breaks this test — that is the ratchet.
- If a variant genuinely has no producing fixture today (e.g. an outcome only reachable under conditions the corpus lacks), the test FAILS by design — do NOT weaken it; instead add the minimal missing fixture under the existing fixture dirs so coverage is real. Report exactly which variants needed new fixtures.

Stronger tier (advisory, must be wired):
- Add a pre-push lefthook command `diff-coverage-advisory` that runs `cargo llvm-cov --diff-only ... report-only` semantics: if `cargo-llvm-cov` is not installed, print a one-line advisory and exit 0 (advisory mode per ticket — gate at ~80% is a LATER phase, not this ticket). If it IS installed, run report-only diff coverage and print the summary; still exit 0 this phase (report-only), non-zero only on tool crash. Keep it fast-fail safe: the block must not slow pre-push beyond the llvm-cov run itself.
- Do NOT add `cargo-llvm-cov` as a repo dependency or modify `versions.ddl.toml` (that is for pinned build tools; llvm-cov stays a dev-optional tool).

## Discipline

Edit only; `cargo check`/`cargo test -p catherine-cli --test variant_coverage` allowed. NEVER: fmt/clippy/other targets/workspace gates, `just ci`, ah/pretender/bd/lefthook runs, git commits, bd mutations.

## Acceptance (report with evidence)

1. variant_coverage test compiles + passes; names every covered variant; new-variant breakage demonstrated by the exhaustiveness guard (describe the guard).
2. List of variants that required new fixtures + which fixture files were added.
3. lefthook pre-push gains the advisory block, outside ddl markers, missing-tool-safe.
4. Exact commands for the orchestrator to verify.
