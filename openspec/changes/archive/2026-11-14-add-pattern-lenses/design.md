# Design: pattern lenses

The seam's shape is fixed by the epic briefs for `catherine-mx1.1` and
`catherine-mx1.2` and is not re-opened here: module path
`crates/catherine-analysis/src/pattern/`, a static `PatternMeta` +
`PatternLens` registry, the `catherine pattern` CLI surface, and the
replay harness at `crates/catherine-analysis/tests/pattern_replay.rs`.
This document records those decisions and fills in what the disposition
table and the briefs leave open: the exact metadata values, the law-kind
assignment per lens, the taxonomy mapping, the expectation table's
derivation, and the fail-closed routes.

Grounding: `docs/ref/pattern-kata-dispositions.md` (laws, scopes,
dispositions), `docs/ref/ploeh-grounding.md` §6 (HYPOTHESIZED candidate
dictionary), `ledger/katas/*/` (`kata.series_verdict`,
`kata.run.commands`, `kata.run.findings`, `kata.expected`,
`kata.comparison.unique_detections`), and
`crates/catherine-analysis/src/rule_lens.rs` (the existing taxonomy
mapping).

## Decision 1 — module layout: one module per lens under `src/pattern/`

```
crates/catherine-analysis/src/pattern/
├── mod.rs                      # LawKind, PatternMeta, LensInput, LensOutcome,
│                               # PatternLens, LensError, registry(), find(), run()
├── composite.rs
├── null_object.rs
├── builder.rs
├── adapter.rs
├── decorator.rs
├── proxy.rs
├── visitor.rs
├── interpreter.rs
├── memento.rs
├── state.rs
├── chain_of_responsibility.rs
└── strategy.rs
```

- `mod.rs` owns the seam and the registry literal; each lens module owns
  exactly one `PatternLens` impl and its `'static PatternMeta`, so a lens's
  law and scope text live next to its code and cannot drift from the
  registry entry by editing a second file.
- The registry in `mod.rs` is the only list of shipped lenses; a lens that
  is not in it is not reachable from `find`, `run`, or `--list`.
- Every module, the registry test, and the replay harness carry the crate's
  `//!` Purpose / Responsibilities / Rationale header (the file-header
  criterion every child ticket restates).
- Rejected: a new `catherine-patterns` crate. The lenses reuse
  `catherine-analysis`'s bounds (`ScopeCaps`), the rule-lens mapping, and
  the kernel's findings type; a separate crate would either duplicate the
  mapping or add a dependency edge with no second consumer, and the fixed
  module path is the briefs' decision.
- Rejected: a runtime plugin table (`HashMap` built at startup, feature
  gated registration). The registry is a static slice; no lens is optional
  at runtime, and a build-time omission then surfaces as the same
  fail-closed unknown-id error instead of a silently smaller `--list`.

## Decision 2 — `PatternMeta` carries the law's honesty boundary

```rust
pub enum LawKind { Identity, Totality, Coverage, Order, Delta }

pub struct PatternMeta {
    pub id: &'static str,          // stable kebab-case id, the bead/ticket name
    pub name: &'static str,        // GoF pattern name
    pub abstraction: &'static str, // universal abstraction, disposition-table wording
    pub law: LawKind,              // the registry's coarse law filter
    pub scope: &'static str,       // the disposition row's law scope, verbatim
}
```

All five fields are `&'static` because they are documentation-grade
metadata: `--list` prints them, the replay table keys on `id`, and the
scope string is the honesty boundary the kata verdict recorded. A scope
string is part of the lens's contract, not a comment — `--list` emits it,
and a lens whose result is coverage-only, unexpressible-widened, or parity
must say so in that field.

`LawKind` is one value per lens even where the disposition row names two
faces (Composite: "identity/totality equations"): the enum is the
registry's coarse filter, and `scope` keeps the row's full wording. No
lens is claimed to check only one equation because its `LawKind` is
singular.

| id | name | abstraction | law | scope (disposition-table wording) |
|---|---|---|---|---|
| `composite` | Composite | monoid (identity) | `Identity` | identity/totality equations; associativity unexpressible |
| `null-object` | Null Object | monoid identity element | `Identity` | nil-identity equations (identity/totality for the nil element) |
| `builder` | Builder | monoid (accumulation) | `Order` | emitted part traces to a declared step; splice order equation |
| `adapter` | Adapter | wrapper law preservation | `Totality` | link-level delegation/field laws; coverage-only |
| `decorator` | Decorator | wrapper law preservation | `Order` | stacking transparency equation; coverage-only |
| `proxy` | Proxy | wrapper law preservation | `Identity` | identity-preservation equation; coverage-only |
| `visitor` | Visitor | sum type / exhaustiveness | `Coverage` | variant coverage + handler classes; no evaluation semantics |
| `interpreter` | Interpreter | sum type / recursive dispatch | `Coverage` | coverage of the variant domain |
| `memento` | Memento | Δ save/restore pair | `Delta` | lost-data enumeration + round-trip non-invertibility proof |
| `state` | State | state shape / discovery questions | `Totality` | transition laws, declared-cell totality; unreachable-state detection unexpressible |
| `chain-of-responsibility` | Chain of Responsibility | catamorphism / handler order | `Order` | rule-lens gap/shadow/overlap/derivable over handler order |
| `strategy` | Strategy | rule lens above the floor | `Coverage` | branch selection as rule regions |

Law-kind assignment rule: the kind names the invariant family the row's
recorded law belongs to. `Identity` for the monoid/nil identity rows
(Composite, Null Object) and the proxy identity law; `Totality` for
declared-cell/link totality (Adapter, State); `Order` for the equations
that constrain sequence (Builder's splice order, Decorator's stacking
order, Chain of Responsibility's handler order); `Coverage` for the
variant/region-domain rows (Visitor, Interpreter, Strategy); `Delta` for
the only save/restore pair (Memento). The assignment never widens the
law — it labels it.

Each lens's evidence and fixture (from its ledger entry's `fixture` and
`run.commands`):

| id | evidence | fixture root | recorded op |
|---|---|---|---|
| `composite` | `ledger/katas/composite-null-object/` | `fixtures/katas/composite-null-object/` | `check` (+ lost-cell `migrate` on the flatten fault) |
| `null-object` | `ledger/katas/composite-null-object/` | `fixtures/katas/composite-null-object/` | `check` |
| `builder` | `ledger/katas/builder-monoid/` | `fixtures/katas/builder-monoid/` | `check`, `migrate`, `redundant` |
| `adapter` | `ledger/katas/wrapper-laws/` | `fixtures/katas/wrapper-laws/` | `check` |
| `decorator` | `ledger/katas/wrapper-laws/` | `fixtures/katas/wrapper-laws/` | `check` |
| `proxy` | `ledger/katas/wrapper-laws/` | `fixtures/katas/wrapper-laws/` | `check` |
| `visitor` | `ledger/katas/visitor-sum-types/` | `fixtures/katas/visitor-sum-types/` | `rules` |
| `interpreter` | `ledger/katas/visitor-sum-types/` | `fixtures/katas/visitor-sum-types/` | `rules` |
| `memento` | `ledger/katas/memento-roundtrip/` | `fixtures/katas/memento-roundtrip/` | `migrate` |
| `state` | `ledger/katas/state-discovery/` | `fixtures/katas/state-discovery/` | `check`, `underconstraint` |
| `chain-of-responsibility` | `ledger/katas/cor-strategy-rules/` | `fixtures/katas/cor-strategy-rules/` | `rules` |
| `strategy` | `ledger/katas/cor-strategy-rules/` | `fixtures/katas/cor-strategy-rules/` | `rules` |

Interpreter and Visitor share the `visitor-sum-types` evidence because
the ledger executed one fixture for both abstractions (sum-type coverage
and exhaustive handler classes are the same surface read two ways); their
scopes differ, and neither lens may claim the other's findings.

## Decision 3 — static registry, exact ids, deterministic order

```rust
pub fn registry() -> &'static [&'static dyn PatternLens];
pub fn find(id: &str) -> Option<&'static dyn PatternLens>;
pub fn run(id: &str, input: &LensInput<'_>, caps: &ScopeCaps)
    -> Result<LensOutcome, LensError>;
```

- **One canonical spelling per id**, kebab-case as in the bead titles and
  the table above; lookup is exact and case-sensitive. No aliases, no
  normalization (`chain_of_responsibility`, `NullObject`, `CoR` do not
  resolve). A near-miss is the same fail-closed path as an unknown id:
  accepting spellings would make a typo silently select a law, and the
  registry is small enough that the error can list the known ids.
- `registry()` is ordered by `id`; a seam test asserts that literal order,
  so `--list`, the unknown-id diagnostic, and the replay table's iteration
  are byte-stable.
- `run` returns `Err(LensError::UnknownPattern { id, known })` for an
  unregistered id; `known` is the registry's id list so the CLI can print
  the remediation without reading the module's internals. This is the
  failure-taxonomy class-A surface: an absent lens is a labeled error, not
  an empty result.
- A duplicate id in the registry literal is a programming error caught by
  a seam test (the registry is a compile-time literal; the test is the
  guard that two modules cannot claim one id).

## Decision 4 — `LensInput`/`LensOutcome`: no new analysis path

```rust
pub struct LensInput<'a> {
    pub model: &'a catherine_core::model::Model,
    pub values: &'a [String], // the CLI's `--values column=v1,v2,...` spellings
}

pub struct LensFindings {
    pub checks: Vec<Finding>, // the P2 taxonomy's findings
    pub migrations: Vec<catherine_core::migrate::MigrationFinding>, // the migrate surface
}

pub enum LensOutcome {
    Findings(LensFindings),
    Unknown { scope: String }, // the exact scope that could not be decided
}

pub trait PatternLens {
    fn meta(&self) -> &'static PatternMeta;
    fn run(&self, input: &LensInput<'_>, caps: &ScopeCaps) -> LensOutcome;
}
```

- **Two finding channels, two kernel shapes.** A completed run carries the
  P2 taxonomy's checks and the migration engine's lost-data findings side by
  side. `migrations` holds `catherine_core::migrate::MigrationFinding`
  (`lost_row`/`lost_cell`, label `executed`) verbatim — the migrate
  surface's own type, not another taxonomy kind. The Δ lenses' recorded
  multisets include those kinds (composite/null-object's flatten fault,
  builder's dropped step, memento's round trips), so a `Vec<Finding>`-only
  outcome could not carry three of the twelve disposition rows; splitting by
  surface keeps the taxonomy closed and each channel's serde encoding the
  kernel's own (the CLI prints `checks` under `data.findings` and
  `migrations` under `data.migrations`).
- **Rule-table lenses** (`visitor`, `interpreter`, `chain-of-responsibility`,
  `strategy`) bind their finite domain with
  `catherine_core::rule_lens::RuleDomain::from_presentation` from
  `input.values` and reuse `catherine_analysis::rule_lens::rule_lens` — one
  analysis path, the same one the ledger's `rules` commands recorded. They
  fill `checks` and leave `migrations` empty.
- **Equation/Δ lenses** reuse the kernel operations their kata commands
  recorded: `check` (equation and structural findings), `migrate` (lost-row
  and lost-cell enumeration), `redundant` (Builder's duplicated-law
  witness), `underconstraint` (State's schema-level questions). The seam
  composes these calls; it does not re-implement any of them.
- `caps` is the shared `ScopeCaps`; a lens that enumerates a domain can
  return `Unknown { scope }` on budget expiry. UNKNOWN is a completed
  bounded run (the `rules` precedent), never "clean".
- No lens returns a partial finding list: a run either completes with its
  findings or reports the scope it could not decide.

## Decision 5 — findings reuse the P2 taxonomy, mapping unchanged

The mapping is `rule_lens.rs`'s, verbatim:

| structural fact | finding kind | label | witness |
|---|---|---|---|
| rule gap / uncovered variant | `underconstraint` | `hypothesized` | `UncoveredInputs` (class + dense example indices) |
| overlap under `DeclaredDisjoint` | `violation` | `executed` | `EquationWitness` reused as the disjointness claim (lower rule index, witness input index, two rule indices) |
| shadowed handler/rule | `redundant` | `executed` | rule name against the earlier covering rules |
| derivable branch | `redundant` | `executed` | rule name + its two combining factors |
| equation violation (`parent∘top = top`, splice/stack order, proxy identity) | `violation` | `executed` | `EquationWitness` |
| dangling cell / lost row / lost cell | `dangling` or `lost_*` as `check`/`migrate` already emit | `executed` | kernel witness |

- Five finding kinds total (`violation`, `dangling`, `contradiction`,
  `underconstraint`, `redundant`) plus the migration class's lost-data
  findings; `pattern` introduces none. The output is sorted by the
  taxonomy's total order, like every other producer.
- Rejected: a `pattern` field on `Finding`. The taxonomy is shared by
  `check`, `findings`, `rules`, `diff`, `migrate`, and the web panel; a
  kind-level pattern tag would make one finding list carry two
  classification axes and force every consumer to learn the new one. The
  lens id belongs to the command invocation, not to the finding.
- Rejected: emitting pattern findings from plain `check`. The kata verdicts
  include parity and killed rows; running lenses on the default path would
  make every fixture emit pattern findings, change existing exit codes, and
  destroy the "parity rows stay silent" gate. A lens runs only when named.

## Decision 6 — CLI surface and the fail-closed routes

```
catherine pattern <model.ct.json> [--id <pattern>] [--values ...] [--list]
```

- `--list` prints the registry — id, GoF name, abstraction, law kind,
  scope — in `registry()` order, and runs no lens. The model positional is
  required by the parser for one command shape but is not read on this
  path, so listing never depends on a document.
- `--id <pattern>` runs exactly that lens over the model. `--values` is the
  same spelling `rules` accepts (`column=v1,v2,...`, repeatable); its
  syntax is parsed CLI-side (a malformed spelling is the existing labeled
  `bad_argument` error, exit 1), and it binds the rule-table lenses'
  finite domain. A syntactically valid `--values` that does not bind the
  declared columns, or that is supplied to a lens with no rule surface, is
  an unevaluable request: the lens reports `Unknown { scope }` naming the
  mismatch — never a silently ignored flag, never an empty findings list.
- A run without `--id` (and without `--list`) is the same labeled usage
  error as an unknown id. There is no implicit "run every lens": one
  invocation with mixed scopes would make the replay gate's per-lens
  granularity unobservable and would present coverage-only results beside
  scoped ones.
- **Exit contract** (no change to the shipped table):
  - findings → the minimum present class code (3..=7), exactly as `check`;
  - any migration finding → 8, the table's lost-data class
    (`crate::cli::LOST_DATA_EXIT_CODE`), exactly as `migrate`; the lost
    data is the run's own result, so it is not outranked by the checks'
    minimum;
  - `Unknown { scope }` → 0 with `outcome: "unknown"` and the scope in the
    payload (a completed bounded run, never "clean");
  - unknown or missing `--id` → 2 (usage class) with a labeled
    `bad_argument` diagnostic naming the id and listing the known ids;
  - a document that fails to parse or type → the existing labeled
    parse/build error, exit 1, before any lens runs (`load_model`).
- **Unevaluable input is never empty success.** The seam's `run` is
  infallible for a built model (Decision 4), so an input the named lens
  cannot evaluate — no `rule_presentation` for a rule-table lens, `--values`
  that do not bind the declared columns or that name no rule surface, a
  document with no `schema_map` for the Δ lens — is reported as
  `LensOutcome::Unknown { scope }` whose scope string names the missing
  surface — for example, the scope reads `no rule_presentation section;
  nothing evaluated`, or `schema_map absent; no lost-data enumeration`.
  The CLI renders it as `outcome: "unknown"` + scope with zero findings and
  zero migrations — labeled exactly like the bounded non-answer, and never a
  bare `findings: []` presented as clean. A malformed document still takes
  the exit-1 error route, so "parsed but unevaluable" and "did not parse"
  stay distinguishable.
- Rejected: routing the unevaluable case through the `rules` command's
  `no_rule_presentation` error. `pattern` names a lens, and the lens's
  required surface is its `scope`'s; the seam signature (Decision 4) has no
  error arm from a lens, and inventing one would change the fixed shape.
  The distinction is kept observable: parse failures are errors, an
  unevaluable surface is a labeled UNKNOWN.

## Decision 7 — replay harness and the checked-in expectation table

```
crates/catherine-analysis/tests/pattern_replay.rs        # harness
crates/catherine-analysis/tests/pattern_expectations.rs  # table (checked in)
```

- The table is derived from the sealed ledger entries at the epic's base
  commit, one expectation per recorded fault:
  `fixture`, `values`, `fault`, the recorded `catherine_class`, the
  `(kind, label)` multiset the entry's `run.findings` records for that
  fixture, and a `parity` flag for faults the entry's
  `comparison.unique_detections` does not count (the baseline caught them).
  `kata.expected[].catherine_class == "gap"` is a **silent** row: the lens
  must not emit that fault's class. A fixture whose recorded multiset is
  empty (the identity control in `memento-roundtrip`, the clean arms) must
  produce zero findings.
- The harness asserts, per table row: the run's `(kind, label)` multiset —
  composed across both channels, `checks` plus `migrations` — equals the
  entry's recorded multiset, and the parity flags are reported as parity
  only. A row recording `lost_row`/`lost_cell` is therefore satisfiable only
  by the lens's `migrations` list; a checks-only or empty outcome fails it
  loudly rather than passing silently. It has **no unique-detection output
  and no detection-rate assertion**: parity rows stay silent in the gate's
  result, which is what "parity, no claim" means operationally.
- Test names are the pattern ids, so the per-lens Meter is
  `cargo test -p catherine-analysis --test pattern_replay <pattern-id>`
  and the whole suite is the same command without the filter.
- The table is checked in, not recomputed from `ledger/katas/` at test
  time: the ledger is sealed evidence, and a live read would let an edit
  move the gate silently and would make the test binary depend on files
  outside the crate. A mismatch is a lens bug (or a spec bug) — the fix is
  never to edit the expectation row to match the output.
- Rejected: JSON expectation file. The table is a `&'static` Rust literal in
  the test crate: a typo in a pattern id or kind fails to compile, while a
  JSON table would only fail at runtime and could drift from the harness's
  schema unnoticed.

## Decision 8 — the honesty boundary is part of every lens

- No lens, CLI payload, or test emits a unique-detection count or a
  detection-rate comparison. The ledger's `comparison` field is evidence
  the replay records as parity, never as a win.
- `scope` is printed by `--list` and carried by the run's payload, so a
  coverage-only result (Adapter, Decorator, Proxy) cannot be read as a
  defect class, and the unexpressible faces (Composite's associativity,
  State's unreachable-state detection) are stated where the lens is named.
- The negative-control bundle (`ledger/katas/negative-control-bundle/`)
  remains the adversarial check: `check` on the Flyweight, procedural-CRUD,
  and lookalike fixtures stays silent. No pattern lens may turn those into
  findings; the replay table does not include them, and adding a lens for
  an unexpressible row is a `catherine-r5k` decision, not a lens ticket.

## Rejected alternatives (summary)

| Alternative | Why rejected |
|---|---|
| New finding kind per pattern | `rule_lens.rs` Decision 1 already pins diagnostics onto the five existing kinds with witness encodings; a new kind would break `findings` stability and the exit table for every consumer. |
| Pattern findings on plain `check` | Parity/killed rows would fire on the default path, change existing exit codes, and remove the silence the honesty gate depends on. |
| Runtime plugin registration | Fixed static registry; no lens is optional at runtime, and dynamic registration hides a missing lens behind a smaller list instead of a labeled unknown id. |
| Aliases / id normalization | A typo would resolve to a law silently; exact ids keep lookup class-A fail-closed. |
| Recomputing expectations from the ledger at test time | The ledger is sealed; a live dependency lets evidence edits move the gate and couples the test binary to the repo layout. |
| A `--pattern` flag on `rules` | `rules` already carries a projection contract (`--project/--columns/--action/--values`); a separate subcommand keeps that contract untouched and gives `--list` a home. |
| Lenses for disposed/unexpressible rows | Named gaps owned by `catherine-r5k`; a row needs a new law, not a new fixture. |
| Pattern id on the finding | One list would carry two classification axes; the lens id is the invocation's, not the finding's. |

## Falsifiers

| Claim | Test |
|---|---|
| Registry metadata is complete and unique per id | seam test: twelve ids, literal order, five non-empty fields, no duplicate id |
| `find`/`run` fail closed on an unknown id | seam test: `Err(UnknownPattern)` names the id and the known ids |
| Unknown id on the CLI is exit 2 with a labeled diagnostic | CLI test: exit 2, `bad_argument`, no stdout envelope |
| `--list` runs no lens and prints all twelve with scopes | CLI test: list output, no findings payload |
| Rule-table lenses reuse the rule-lens mapping | replay: `visitor`, `cor`, `strategy` kinds equal the ledger's `run.findings` |
| Equation/Δ lenses reuse the kernel ops | replay: `composite`, `wrapper-laws`, `memento` kinds equal the ledger's |
| Parity rows stay silent | replay: parity flags reported, no unique-detection output; `builder` duplicate-row row, `visitor` F1/F2, `cor` behavioural faults |
| Killed/gap rows stay silent | replay: `state` unreachable row yields no `violation`/`dangling`; `memento` identity control yields zero findings |
| An unevaluable document is not empty success | seam/CLI test: `Unknown { scope }` naming the missing surface |
| A lens cannot widen its recorded law | review: each lens's `scope` string is the disposition table's, and the replay table's faults are the ones the ledger recorded |