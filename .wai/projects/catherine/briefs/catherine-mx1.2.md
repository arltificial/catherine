# Brief — catherine-mx1.2: pattern-lens registry + replay harness

Beads ticket: `catherine-mx1.2` (P2, lane:afk). Parent epic: `catherine-mx1`. Blocked by `catherine-mx1.1` (spec).
Branch: `main`.

## Why

Delegation rationale: the seam is the contract twelve parallel lens implementors consume; it
must be written once, test-first, and committed before any lens work starts. The lead keeps
pipeline control and wires the tracker.

## Desired outcome

Ship the pattern-lens seam in `catherine-analysis` plus its CLI surface, its contracts, and
the replay harness the lens tickets use as their Meter.

Fixed architecture (do not redesign):

```rust
// crates/catherine-analysis/src/pattern/mod.rs
pub enum LawKind { Identity, Totality, Coverage, Order, Delta }
pub struct PatternMeta {
    pub id: &'static str,
    pub name: &'static str,
    pub abstraction: &'static str,
    pub law: LawKind,
    pub scope: &'static str,
}
pub struct LensInput<'a> { pub model: &'a catherine_core::model::Model, pub values: &'a [String] }
pub enum LensOutcome { Findings(Vec<Finding>), Unknown { scope: String } }
pub trait PatternLens {
    fn meta(&self) -> &'static PatternMeta;
    fn run(&self, input: &LensInput<'_>, caps: &ScopeCaps) -> LensOutcome;
}
pub fn registry() -> &'static [&'static dyn PatternLens];
pub fn find(id: &str) -> Option<&'static dyn PatternLens>;
pub fn run(id: &str, input: &LensInput<'_>, caps: &ScopeCaps) -> Result<LensOutcome, LensError>;
```

- Lenses bind their declared domain with `catherine_core::rule_lens::RuleDomain::from_presentation`
  and reuse `catherine_analysis::rule_lens::rule_lens`; findings are the existing P2 taxonomy
  kinds with witnesses. No new finding kinds, no new heuristics.
- Unknown pattern id: labeled `bad_argument` error, CLI exit 2, never empty success.
- CLI: `catherine pattern <model.ct.json> [--id <pattern>] [--values ...] [--list]`, same
  envelope/exit conventions as `catherine rules` (reuse its runner where possible).
- Reference lens is TEST-ONLY (`crates/catherine-analysis/tests/pattern_reference.rs`), Visitor
  variant coverage; the shipped Visitor lens belongs to `catherine-mx1.9`. No production lens
  ships in this ticket.
- Replay harness: `crates/catherine-analysis/tests/pattern_replay.rs` plus the checked-in
  expectation table `crates/catherine-analysis/tests/pattern_expectations.rs`, as specified
  in `openspec/changes/add-pattern-lenses/design.md` Decision 7. The foundation ships the
  table COMPLETE for all 12 in-scope patterns (and their kata dirs) derived from
  `ledger/katas/*/` at commit `e818263`: fixture, values, fault, recorded class, recorded
  `(kind,label)` multiset, parity flag. Lens tickets never edit this table. The harness must
  make `cargo test -p catherine-analysis --test pattern_replay <pattern-id>` the per-lens
  Meter.
- Registry wiring is the orchestrator's: `mod.rs` exposes an explicit `registry()` slice; the
  foundation ships it with zero production lenses. The lead adds one line per landed lens and
  re-runs the replay. Do not make the registry scan the filesystem or use link-time
  registration.
- Contracts: `.espectacular/pattern-lens/` toml files for the seam scenarios that now have
  tests (registry metadata, replay gate, fail-closed unknown id, taxonomy reuse), following
  `.espectacular/rule-lens/` format; every contract cites an existing test flag.
- Docs: README command list + `docs/USING.md` gain the `catherine pattern` surface, one
  paragraph + one command example each.
- File headers: every new module/test file carries the crate's `//!` Purpose /
  Responsibilities / Rationale header.

## Out of scope

- No per-pattern production lens (those are `catherine-mx1.3` … `.14`).
- No ledger edits, no disposition-table edits, no detection-rate claims.
- No new finding kinds, no changes to `rule_lens` output semantics.

## Exact files

- `crates/catherine-analysis/src/pattern/mod.rs`
- `crates/catherine-analysis/tests/pattern_replay.rs`
- `crates/catherine-analysis/tests/pattern_reference.rs`
- `crates/catherine-cli/src/pattern.rs`
- `crates/catherine-cli/src/cli.rs`
- `README.md`, `docs/USING.md`
- `.espectacular/pattern-lens/`

## Red → green → refactor

1. RED: failing tests for `find("unknown")` error, registry metadata for the reference lens,
   replay harness over the reference fixture asserting the recorded visitor classes.
2. GREEN: module, registry, reference lens, CLI wiring, harness, contracts, docs.
3. REFACTOR: within-ticket only; the cross-lens refactor is `catherine-mx1.15`.

## Completion criteria (runnable; exit 0 = done)

```bash
cargo test -p catherine-analysis --test pattern_replay
cargo test -p catherine-cli --test cli
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
ah check
cargo run -q -p catherine-cli -- pattern fixtures/katas/visitor-sum-types/visitor.ct.json --list
cargo run -q -p catherine-cli -- pattern fixtures/katas/visitor-sum-types/visitor.ct.json --id visitor --values kind=lit,neg
```

The last command exits 0 with `findings` (or `outcome: "unknown"` + scope); an unknown id exits 2
with a labeled error. `ledger-check ledger/katas` stays green.

## Spawn model

Agent: `task` (harness default model). Context ceiling: ≤20% of the model window.

## Verification (orchestrator, after return)

Full command list above; inspect `git diff` for taxonomy/ledger drift; confirm the reference
lens is test-only and no production lens was added.

## Follow-up threshold

If a fixed signature cannot express a law without changing `rule_lens` semantics or adding a
finding kind, stop and report the exact signature conflict instead of widening the seam.