# Brief — catherine-mx1.1: add-pattern-lenses spec

Beads ticket: `catherine-mx1.1` (P2, lane:afk). Parent epic: `catherine-mx1` (pattern lenses).
Branch: `main`. Base commit: see `.wai/projects/catherine/runs/catherine-mx1.1.state`.

## Why

Delegation rationale: the spec is a bounded authoring task whose output is a committed
repo artifact, consumed by the foundation ticket `catherine-mx1.2`; the lead keeps the
pipeline moving instead of writing prose inline. Espectacular requires the spec to exist
before the foundation's contract tests can declare correspondence.

## Desired outcome

Author the openspec change `add-pattern-lenses` describing a `pattern-lens` capability:

- A named registry of pattern lenses, each carrying: stable id, GoF pattern name,
  universal abstraction, law kind (`identity` | `totality` | `coverage` | `order` |
  `delta`), and a scope string copied from the disposition table.
- A replay gate: running a lens over its kata fixture reproduces the recorded ledger
  verdicts (finding class per fault); parity/killed rows stay silent.
- Fail-closed unknown pattern id: a labeled usage error, never empty success
  (failure-taxonomy class A).
- Findings use the existing P2 taxonomy kinds and witnesses; no new finding kinds.
- 12 executed patterns from `docs/ref/pattern-kata-dispositions.md` are in scope:
  Composite, Null Object, Builder, Adapter, Decorator, Proxy, Visitor, Interpreter,
  Memento, State, Chain of Responsibility, Strategy. Disposed/unexpressible rows are
  explicitly out of scope (owned by epic `catherine-r5k`).

Requirements are stated behaviorally; the five seam requirements (registry, replay,
fail-closed, taxonomy, pipeline integration) plus one requirement per pattern is
acceptable granularity. Each requirement gets at least one `#### Scenario:`.

## Out of scope

- No Rust code, no test code, no contracts in `.espectacular/` (they land with the
  foundation tests in `catherine-mx1.2`).
- Do not edit `ledger/katas/` (sealed) or `docs/ref/pattern-kata-dispositions.md`.
- No detection-rate claims, no new heuristics, no prover/rewriting scope.

## Exact files (create)

- `openspec/changes/add-pattern-lenses/proposal.md`
- `openspec/changes/add-pattern-lenses/design.md`
- `openspec/changes/add-pattern-lenses/tasks.md`
- `openspec/changes/add-pattern-lenses/specs/pattern-lens/spec.md`

Format precedent: `openspec/changes/add-declarative-keys/` (proposal/design/tasks/specs
deltas) and `openspec/changes/add-bogart-pilot/` (scenario style).

## Required content anchors

- Read before writing: `docs/ref/pattern-kata-dispositions.md` (laws + scopes),
  `docs/ref/ploeh-grounding.md` §6, `ledger/katas/*/` (`kata.series_verdict`,
  `run.commands`, `run.findings`), `crates/catherine-analysis/src/rule_lens.rs`
  (existing taxonomy mapping), `crates/catherine-cli/src/cli.rs` (`Commands`),
  `.espectacular/AGENTS.md`.
- Design doc must decide: module path (`crates/catherine-analysis/src/pattern/`),
  lens metadata struct, registry representation, CLI surface
  (`catherine pattern <model.ct.json> [--id <pattern>]`, plus a `--list` form), and the
  replay harness shape (`crates/catherine-analysis/tests/pattern_replay.rs` reading a
  checked-in expectation table derived from the ledgers).
- Tasks doc: red→green→refactor, one checked item per child ticket
  (`catherine-mx1.2` … `catherine-mx1.15`), reference each beads id.
- No secrets, no dates in prose beyond what exists, no invented evidence: cite the
  disposition table and ledger paths only.

## Completion criteria (runnable; exit 0 = done)

```bash
openspec validate --specs --changes
ah check
test -f openspec/changes/add-pattern-lenses/proposal.md
test -f openspec/changes/add-pattern-lenses/design.md
test -f openspec/changes/add-pattern-lenses/tasks.md
test -f openspec/changes/add-pattern-lenses/specs/pattern-lens/spec.md
git status --short openspec/changes/add-pattern-lenses
```

`git status` shows only the four new files (untracked is expected; the orchestrator
commits). `ah check --changes add-pattern-lenses` may report `no-tests-declared` for
scenarios whose tests land in `catherine-mx1.2` — that is expected; any other structural
finding is a failure.

## Spawn model

Agent: `task` (harness default model). Context ceiling: ≤15% of the model window;
the task is four Markdown files, so stop reading once the anchors above are covered.

## Verification (orchestrator, after return)

`openspec validate --specs --changes`; `ah check`; inspect the four files against the
anchors; no ledger or disposition edits.

## Follow-up threshold

If the change cannot be authored without a semantics decision the disposition table does
not settle (for example a new law), stop and report the exact question instead of
inventing a law.