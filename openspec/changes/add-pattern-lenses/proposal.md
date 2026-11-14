# Change: Pattern lenses (named, scoped pattern → abstraction law checks)

> Grounding: `docs/ref/pattern-kata-dispositions.md` (the GoF →
> universal-abstraction disposition table) and `docs/ref/ploeh-grounding.md`
> §6 (the HYPOTHESIZED candidate dictionary). The kata suite under
> `ledger/katas/` executed each in-scope law on a modeled fixture and
> recorded a pre-registered, scoped verdict; this change turns the twelve
> **executed** rows into a named product surface.

## Why

The twelve executed rows of the disposition table already run. Their
evidence is a ledger entry per fault (`ledger/katas/<kata>/`) with the
fixture, the executed conventional baseline, the per-fault finding class,
the unique set, and the expressibility gap. What the product does not have
is a name for any of it: there is no seam that says "this is the Composite
law, this is its abstraction, this is its scope", no way to look one up by
id, and no gate that a later lens change must pass. Today a caller can only
re-run the individual `catherine check` / `catherine rules` commands the
ledger entry records, with the fixture and flags copied by hand.

Two properties of the evidence force a spec before the code:

- **The results are scoped, not wins.** Adapter, Decorator, and Proxy are
  recorded coverage-only: a complete cross-product sample collapses every
  unique detection, so the unique result is a sampling fact, not a defect
  class. Composite and State record invariants the current surface cannot
  express (associativity; unreachable-state detection), Builder's
  duplicate-row case is parity, and every near-floor control self-negates.
  A surface that did not carry the scope string would present a scoped
  result as a detection-rate claim the ledger does not license.
- **The out-of-scope rows are named gaps.** Construction patterns
  (Abstract Factory, Factory Method, Facade, Iterator, Template Method),
  the identity rows (Singleton, Flyweight), and the killed
  `weak-lens-set` rows (Prototype, Bridge, Command, Observer) are
  disposed, unexpressible, or parity. They are owned by epic
  `catherine-r5k`; the registry must not quietly absorb them, and a
  re-open requires naming a new law, not adding a fixture.

The honesty gate is part of the evidence: `ledger/katas/negative-control-bundle/`
records pattern-lookalike and procedural fixtures where the kernel must
stay silent, plus a Flyweight identity case where no lens exists. A
pattern surface that fired there would be bought with false positives.

## What Changes

- **New capability `pattern-lens`**: a static registry in
  `crates/catherine-analysis/src/pattern/` of twelve lenses, each carrying
  a stable id, the GoF pattern name, its universal abstraction, one law
  kind (`identity | totality | coverage | order | delta`), a scope string
  copied from the disposition table, and a run method returning standard
  findings. `--list` prints the registry's metadata.
- **Twelve executed patterns, one lens each**: Composite, Null Object,
  Builder, Adapter, Decorator, Proxy, Visitor, Interpreter, Memento,
  State, Chain of Responsibility, Strategy. Each lens's law and scope are
  the disposition row's own text; each lens's fixture is the kata's
  `fixtures/katas/<kata>/` model.
- **CLI surface**: `catherine pattern <model.ct.json> [--id <pattern>]
  [--values <column=v1,v2,...>]`, plus `--list`, through the same
  envelope and exit-code table as the other analysis commands. An unknown
  pattern id is a labeled `bad_argument` usage error (exit 2), never empty
  success (failure-taxonomy class A: no silent drop).
- **Replay gate**: `crates/catherine-analysis/tests/pattern_replay.rs`
  reads a checked-in expectation table derived from `ledger/katas/` and
  asserts, per kata fault, the finding class the ledger records. Parity
  rows stay silent: the gate has no unique-detection output and no
  detection-rate assertion, and a killed/gap row expects the silence the
  ledger records for it.
- **Findings reuse the P2 taxonomy**: the existing kinds and witnesses,
  emitted through the existing rule-lens mapping. No new finding kinds, no
  new heuristics.

## Non-goals

- **No detection-rate claim.** Parity with conventional tests is the
  expected outcome (E4); the executed uniques are scoped and several are
  sampling gaps. The change licenses a structural, schema-level claim
  only.
- **No lens for a disposed or unexpressible row.** Abstract Factory,
  Factory Method, Facade, Iterator, Template Method, Singleton, Flyweight,
  and the killed `weak-lens-set` rows stay out; epic `catherine-r5k` owns
  the gap work.
- **No new heuristics, prover, or rewriting.** A lens runs the recorded
  law over the model it is given; widening a law beyond its recorded scope
  is a different change with a named law first.
- **No taxonomy or `rule_lens` semantics change.** Budget expiry remains
  an honest `UNKNOWN` with the explored scope, not "clean".
- **No ledger or disposition-table rewrite.** The ledgers are sealed; each
  child lens ticket carries only its own single-row implemented-status
  edit in the disposition table.

## Impact

- Affected specs: `pattern-lens` (new). `rule-lens`, `findings`, `cli`,
  and `analysis` are reused as-is; the CLI addition is a new subcommand
  with the shipped exit table, not a change to an existing requirement.
- Affected code: `crates/catherine-analysis/src/pattern/` (module,
  registry, twelve lens modules), `crates/catherine-analysis/tests/pattern_replay.rs`,
  `crates/catherine-analysis/tests/pattern_reference.rs` (test-only seam
  proof), `crates/catherine-cli/src/{cli,pattern}.rs`, README and
  `docs/USING.md` command lists, `.espectacular/pattern-lens/` contracts.
- Evidence: replaying each lens over `fixtures/katas/<kata>/` reproduces
  the per-fault classes recorded in `ledger/katas/<kata>/`; the sealed
  ledgers and `ledger-check ledger/katas` stay untouched.
- Beads: `catherine-mx1.2` (registry and replay seam) through
  `catherine-mx1.15` (cross-lens refactor).