# Lean assurance twin

An external Lean 4 reference semantics for the `catherine-core` kernel:
the schema, the typed-instance build, path composition, the `check` walk,
and the findings taxonomy. It is an assurance artifact, not an engine —
nothing in the Cargo workspace depends on it, and it has no runtime
dependency on the Rust crates.

## What this is

`lean/` is a standalone Lake project, standard-library only (no Mathlib),
pinned by `lean/lean-toolchain` to `leanprover/lean4:v4.36.0` and
provisioned through elan. The kernel modules mirror the Rust shapes and
ordering contracts:

- `Catherine.Schema` — identifiers, dense row ids, paths, arrows,
  equations, keys, coproducts, `accepted`/`proposed` constraint status,
  and total lookups.
- `Catherine.Instance` — `RawInstance` to `TypedInstance`; `build` is
  total and typing-enforcing, with each raw cell landing in exactly one
  of three places: a stored morphism value, a dangling record, or a
  typing-violation report.
- `Catherine.EvalPath` — `compose`, with `Option` short-circuit on any
  unresolved cell; an empty path is the identity.
- `Catherine.Check` — dangling, equation, key, and coproduct findings,
  including the typing-violated-cell accounting that suppresses duplicate
  dangling reports, sorted under the taxonomy order.
- `Catherine.Findings` — the five finding kinds with mandatory witnesses,
  the naturality witness shape (defined but never produced by `check`),
  the three constructible labels (`executed`, `hypothesized`, `unknown`),
  and the total order.

`Catherine.Mirror` projects a `.ct.json` document into the trusted mirror
JSON (canonical schema order plus a dense partial instance) and decodes
that JSON again; `Catherine.Parity` / `Catherine.ParityMain` implement the
oracle. The kernel modules import nothing from `Lean`; the oracle entry
points use Lean's standard JSON and IO support. The oracle invokes the
Rust executable as a subprocess, so the two implementations share no
linked code at run time.

## What it claims

### Parity

`lake exe lean-parity` enumerates the check-applicable corpus dynamically
(`fixtures/{katas,parity,probes}/**/*.ct.json`), projects each document
to the trusted mirror, runs Lean `build` + `check`, runs Rust
`catherine check --json`, and compares the results as normalized finding
multisets — kind, label, and witness fields — never raw bytes or text
output. Documents that the Rust parser/build rejects (`check` exits 1)
are recorded as `skipped_parse` with their first stderr line, not as
agreement or divergence. Any other unexpected CLI exit is a harness
error.

### Theorems

Three theorems are stated over the semantic notions, not over the shape
of the walk:

- `Catherine.Proofs.soundness` — every finding in `check schema inst`
  carries its semantic witness: an equation violation on an applicable
  row whose composites both resolve and differ; a key collision between
  two distinct ascending rows with an equal resolved tuple; a coproduct
  coverage or disjointness failure under the complete-image accounting;
  or a dangling cell that is unresolved and not typing-accounted.
- `Catherine.Proofs.equation_completeness` — for any equation index and
  any applicable row whose lhs and rhs composites both succeed and
  differ, `check` contains the exact equation violation, with the status
  label (`executed` for accepted, `hypothesized` for proposed).
- `Catherine.Proofs.dangling_discipline` — dangling cells stay `none`;
  an equation path that short-circuits on a dangling or typing-violated
  cell emits no violation; a row with an unresolved key cell is in no
  key-collision finding; a partial injection image suppresses coverage
  findings; a typing-violated cell is accounted exactly once (its build
  report, not a dangling finding); no cell is repaired.

The statements cite `openspec/specs/kernel-checks/spec.md` and
`openspec/specs/findings/spec.md`. `scripts/check-lean-axioms.sh` is the
gate: it fails the build on any `declaration uses 'sorry'` warning,
elaborates `lean/Catherine/Axioms.lean` and asserts that every audited
declaration's axiom set is inside `{propext, Classical.choice,
Quot.sound}` (an unexpected name, including `sorryAx`, is reported by
name), and scans every `lean/Catherine/**/*.lean` source for an `axiom`
declaration. A proof claim cannot be accepted on trust.

## What it does not claim

- **No keys or coproducts completeness theorem is claimed.** Their
  findings are covered by soundness, but the completeness theorems are
  deferred (design Decision 3). Equation completeness is the only
  completeness claim.
- **Bounded search, Δ migration, and the rule lens are not modeled.**
  Those surfaces are follow-up work, not part of this twin.
- **Parsing and malformed-input rejection are not in Lean.** The twin
  consumes a trusted mirror projection. Identifier validation, spans,
  error recovery, and rejection stay Rust-side; malformed JSON or an
  unresolvable row name is a harness error (exit 2), never a semantic
  divergence.
- **Nothing is claimed about the CLI, wasm, or web surface.** The oracle
  calls the CLI only to obtain Rust findings; the twin is not wired into
  the product's runtime.

## How to run

Prerequisites: elan (it reads `lean/lean-toolchain` and fetches the
pinned toolchain), and a built Rust CLI for the oracle. The oracle looks
for `target/debug/catherine` under the workspace root; build it with
`cargo build -p catherine-cli`, or point `CATHERINE_BIN` at an existing
binary. The workspace root is found by walking up from the current
directory for `Cargo.toml` + `crates/catherine-cli`; override it with
`LEAN_PARITY_ROOT`.

Build and run from `lean/`:

```sh
cd lean
lake build
lake exe lean-parity
lake exe lean-parity --check-build
```

Oracle options (all accepted by `Catherine.Parity`; see
`lake exe lean-parity --help`):

| Option | Effect |
|---|---|
| `--check-build` | Also compare Lean `build` tables, dangling reports, and typing violations against `expected.json` for the `fixtures/parity/*` cases. |
| `--emit-mirror DIR` | Write the trusted mirror JSON of each projected document under `DIR`, creating subdirectories. |
| `--corpus DIR` | Use `DIR` as the corpus root (default: `<workspace>/fixtures`). |
| `--report PATH` | Write the machine-readable report JSON to `PATH`. |
| `--verbose` | Print each document's Rust exit code; print witness detail on divergences and build mismatches. |
| `--help`, `-h` | Print usage and exit 0. |

From the repository root, `just lean` runs `lake build`, the parity
oracle, and the axiom audit. It is deliberately separate from `just ci`
because the Lean toolchain is provisioned separately from the Rust
toolchain.

Exit codes:

- `0` — every compared document agrees, and with `--check-build` every
  build report agrees.
- `1` — at least one finding divergence, or with `--check-build` at least
  one build/table mismatch.
- `2` — harness error: unrecognized or incomplete flag, missing workspace
  root, corpus, or Rust binary, unreadable/invalid JSON, failed mirror
  projection or decode, malformed `check --json` envelope, or an
  unexpected `catherine check` exit code.
- `--help` exits `0`.

A Rust `check` exit of `1` is not an oracle failure: it is the
`skipped_parse` route for documents the Rust parser/build rejects.

### Report fields

`--report PATH` writes an object with:

- `ok` — true iff `divergences` and `build_mismatches` are both empty.
- `check_build` — whether the run used `--check-build`.
- `corpus.total` — documents enumerated.
- `corpus.rust_checkable` — documents whose findings were compared
  against `catherine check`.
- `corpus.skipped_parse` — array of `{fixture, stderr}` for documents
  whose Rust check exited 1; `stderr` is the verbatim first stderr line.
- `corpus.lean_only_presentation` — presentation-route fixtures compared
  against `expected.json` rather than against a Rust findings envelope.
- `agreement.trivially_agreeing` — fixtures whose Lean result and
  comparison result are both empty (Rust findings, or expected outcomes
  for presentation routes).
- `agreement.findings_bearing_agreeing` — count of fixtures whose Lean
  and comparison multisets are equal and non-empty.
- `divergences` — array of `{fixture, rust, lean, side, beads}`;
  `rust`/`lean` are the normalized finding renderings, `side` is the
  disposition attribution or `"unattributed"`, and `beads` is the ticket
  or `null`.
- `build_mismatches` — array of `{fixture, aspect, expected, lean}` for
  `tables`, `dangling`, or `typing_violations` mismatches under
  `--check-build`.

### Current corpus shape

A fresh run on the current tree reports:

```text
total=190 rust_checkable=180 skipped_parse=10 lean_only_presentation=1
trivially_agreeing=102 findings_bearing_agreeing=79 divergences=0
build_mismatches=0
```

- The corpus is the 190 `*.ct.json` documents currently under
  `fixtures/{katas,parity,probes}/**`. The list is enumerated at run
  time; no expected count or skip list is hardcoded.
- 180 documents are compared against `catherine check --json`.
- 10 are recorded as `skipped_parse` because the Rust parser/build
  rejects them: nine malformed or unbuildable fixtures, plus
  `parity/typing_violation`, the one presentation-route fixture. Its
  `expected.json` declares `"route": "presentation"`, so the oracle
  asserts that Rust rejects it and compares Lean findings against the
  expected outcomes instead.
- The agreement split is 102 trivially agreeing (no findings on either
  side; this includes the presentation fixture) and 79 findings-bearing
  agreeing (101 + 79 = the 180 Rust-compared documents).
- `--check-build` reports zero build mismatches across the seven
  `fixtures/parity/*` cases.

## Divergence protocol

A mismatch is never merged away. Design Decision 4 requires it to become
exactly one of:

1. a **Lean-model defect** — the twin deviates from the documented
   contract, with the normative specs as referee; or
2. a **Rust kernel defect** — the twin is right and the kernel is wrong;
   file its own beads ticket, fix Rust, and add a regression fixture.

`lean/parity-dispositions.json`, when present, maps a fixture display name
to `{side, beads}`. It only annotates attribution in the report. It never
suppresses a divergence: the oracle exits 1 whenever `divergences` is
non-empty, annotated or not. The fix is the disposition; the oracle stays
red until the semantic mismatch is gone.

## Stop gate

Design Decision 5 makes the assurance outcome a recorded decision, not a
silent drift. After the proof phase, the ledger records one of **extend**
(a follow-up change for search/migration), **hold** (maintain the model
at the landed scope), or **retire** (archive the Lean tree and keep the
lessons), together with the cost/benefit accounting: what the proofs
bought in defects or contract holes versus what fixture-only assurance
already pinned. The entry lives under `ledger/lean/` and uses the
`catherine-ledger` decision schema. The gate ran on 2026-11-17 and chose
**hold** — maintain the model at the landed scope, not extend or retire —
recorded in [`ledger/lean/0001-cbef9219.json`](../ledger/lean/0001-cbef9219.json);
its decision record carries the cost/benefit accounting (what parity
already pinned, what the proofs bought, and the extend/retire
counterfactuals).

## Pointers

- Change and task record: `openspec/changes/add-lean-kernel-model/`
  (`proposal.md`, `design.md`, `tasks.md`, spec delta).
- Design decisions: `openspec/changes/add-lean-kernel-model/design.md`
  — Decision 3 (theorem scope), Decision 4 (divergence protocol),
  Decision 5 (stop gate).
- Normative semantics: `openspec/specs/kernel-checks/spec.md` and
  `openspec/specs/findings/spec.md`.
- Fixture expectations: `fixtures/parity/*/expected.json`; the trusted
  mirror contract is implemented by `Catherine.Mirror`.