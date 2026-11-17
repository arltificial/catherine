# Implementor brief — catherine-xvh (Lean 4 kernel model: assurance twin + parity oracle + proofs)

- **Ticket**: `catherine-xvh` — P2/epic, `complexity:xl`, `lane:design` (HITL by ticket note; this run is user-authorized autonomous). Claimed `in_progress`.
- **Openspec change**: `openspec/changes/add-lean-kernel-model` (proposal 59 lines, design 91, spec delta 6 reqs / 7 scenarios, tasks 14). Task 1 (Lake scaffold + elan pin + stubs) is DONE: `lake build` green, 14 jobs.
- **Base**: current `main` @ `16e1492` (release v0.2.0 + beads export). Worktree also carries the scaffold itself (uncommitted `lean/`, the change dir, `.gitignore` `lean/.lake/`, `.beads/issues.jsonl`). Nothing else may change outside `lean/`, the change dir, the two CI plumbing files, and beads export.
- **Reference semantics (referee)**: `openspec/specs/{kernel-checks,findings,model-document}/spec.md` are normative. Rust ground truth: `crates/catherine-core/src/{model/mod.rs,model/relations.rs,instance.rs,checks.rs,checks/coproducts.rs,findings.rs}` and the vendored parity gate `crates/catherine-core/tests/parity_gate.rs` + loader `crates/catherine-core/tests/common/mod.rs`.

## 1. Desired outcome

The Lean tree becomes a *usable* assurance twin, not a scaffold:

1. **Mirror-first kernel** in Lean 4, std only (no Mathlib): `Schema`, `Findings`, `Instance` (total typing-enforcing builder with three landing places), `EvalPath.compose` (None short-circuit), `Check.check` (dangling excluding typing-violated cells, equations, keys, coproducts; taxonomy-sorted).
2. **`lake exe lean-parity`** oracle: enumerates the check-applicable corpus dynamically (`fixtures/{katas,parity,probes}/**/*.ct.json`), projects each document to a trusted mirror, runs Lean `build`+`check`, runs Rust `catherine check --json`, compares normalized finding multisets, reports machine-readable agreement/divergence, exits non-zero on any divergence. `--emit-mirror <dir>` and `--check-build` are required flags.
3. **Three theorems**, `sorry`-free at the end, with `#print axioms` audit: `Catherine.Proofs.soundness`, `Catherine.Proofs.equation_completeness`, `Catherine.Proofs.dangling_discipline`.
4. **CI wiring**: `just lean` recipe + a CI step (elan pinned, `lake build`, `lake exe lean-parity`, axiom audit) alongside the Rust suite.
5. **Gate + docs**: `lean/README.md` (claims/non-claims/oracle/divergence protocol), stop-gate decision recorded as a ledger entry, `ro5u-review.md` in the change dir.

## 2. Completion criteria (runnable; exit 0 unless stated)

1. `cd lean && lake build` — green after every phase.
2. Red evidence (Phase 1, recorded once): `lake exe lean-parity` exits **1** with `findings_bearing_agreeing == 0`, `divergences > 0`, and every zero-finding fixture in `trivially_agreeing` (e.g. `parity/empty_instance`, `parity/zero_file`, `parity/single_intent`, `parity/small`, `parity/typing_allowed`, `parity/typing_violation`). The stub must never mute a findings-bearing fixture.
3. Green (task 6): `lake exe lean-parity` exits **0**; `lake exe lean-parity --check-build` exits **0** for `fixtures/parity/*` (Lean build reports/tables equal `expected.json`); `divergences == []`; the parse-skipped set is reported dynamically (currently 10 docs whose `catherine check` exits 1 — never hardcode the list).
4. Proofs: `lake env lean Catherine/Axioms.lean` prints no `sorryAx` (and no unexpected axioms); the three theorems are stated against the semantic notions (below). Before proofs land, the same command must FAIL with `sorryAx` — that is the proof-phase red.
5. `openspec validate add-lean-kernel-model --strict` — valid; `tasks.md` 14/14 ticks.
6. `ah check` — clean (0 findings) at apply time.
7. `pretender check --mode gate` — clean; every new Lean file carries Purpose/Responsibilities/Rationale headers (headers must follow `import` lines — Lean requires imports first).
8. `TMPDIR=$PWD/target/tmp just ci` — green (Rust suite unregressed; no Rust semantic change is in scope).
9. `openspec/changes/add-lean-kernel-model/ro5u-review.md` exists, records the rule-of-5-universal review of the full diff, **max_critical = 0**, every critical/high fixed, mediums fixed or deferred with written reason.
10. `bd close catherine-xvh` + fresh `bd export` — orchestrator-reserved; implementors never run `bd` mutations.

## 3. Frozen contracts

### 3.1 Mirror JSON (harness projection of a `.ct.json`; also `--emit-mirror` output)

```json
{
  "fixture": "katas/fizzbuzz.ct.json",
  "schema": {
    "objects": ["Divisor", "Output", "Rule"],
    "arrows": [{"name": "divisor", "src": "Rule", "tgt": "Divisor"}],
    "equations": [{"lhs": ["output"], "rhs": ["output"], "status": "accepted"}],
    "keys": [{"object": "Card", "arrows": ["card_inj"], "status": "accepted"}],
    "coproducts": [{"object": "Method", "arrows": ["card_inj", "transfer_inj"], "status": "accepted"}]
  },
  "instance": {
    "rows": {"Divisor": 3, "Output": 3, "Rule": 3},
    "values": {"divisor": [0, 1, 2], "output": [0, 1, 2]}
  }
}
```

Rules, matching Rust parse/normalization exactly:
- `objects` sorted; `arrows` sorted by name; `equations` canonical order `(lhs, rhs, rationale, status-as-string)`; `keys`/`coproducts` canonical order `(object, arrows, status-as-string)`.
- Dense row ids per object in **sorted row-name order** (0..n-1).
- `instance.values[arrow]` length = source object row count; each cell is a row id (`null` when the document's table has no cell or `null`). Resolution: arrow target's row-name map first, else the first declared object (sorted object order) whose map contains the name. An unresolvable name is a **harness error** naming fixture/arrow/row — never a divergence.
- Documents may omit `keys`/`coproducts`/`equations`/`instance.tables`; default to empty.
- Only these sections are read. `format_version`, `schema_map`, `instance_map`, rules, etc. are ignored by `check`.
- Trusted projection only: no validation, spans, or error recovery. Malformed JSON/shape = harness error exit 2.

### 3.2 Lean API (names frozen; semantics phase 1 = empty stubs)

- `Catherine.Schema`: `abbrev Ident := String`, `abbrev RowId := Nat`, `abbrev Path := List Ident`; `structure Arrow {src tgt}`; `inductive ConstraintStatus | accepted | proposed`; `structure Equation {lhs rhs : Path, status}`; `structure Key {object, arrows, status}`; `structure Coproduct {object, arrows, status}`; `structure Schema {objects : List Ident, arrows : List (Ident × Arrow), equations : List Equation, keys : List Key, coproducts : List Coproduct}` + total lookups (`arrow?`, `rowCount?`).
- `Catherine.Findings`: `EpistemicLabel := executed | hypothesized | unknown` (only these three constructible); `EquationWitness`, `KeyWitness`, `CoproductWitness {coproduct, row, injections}`, `ViolationWitness := equation | key | naturality | coproduct` (naturality shape defined, never produced by `check`), `DanglingWitness`, `RedundantWitness`, `ContradictionWitness`, `UnderconstraintWitness`, `FindingKind := violation | dangling | redundant | contradiction | underconstraint` (five, in this order), `Finding {label, kind}`; total `Ord` instance (kind declaration order → witness ids → label last); `labelFor : ConstraintStatus → EpistemicLabel`.
- `Catherine.Instance`: `TypingViolation {arrow, src, value}`, `DanglingCell {arrow, src}`, `RawInstance {rows : List (Ident × Nat), values : List (Ident × List (Option RowId))}`, `TypedInstance {rows, tables, typingViolations, dangling}` with `build : Schema → RawInstance → TypedInstance` (three landing places; table per declared arrow; reports ordered arrow-name then row). Phase 1 stub: empty tables/reports (zero findings).
- `Catherine.EvalPath.compose : TypedInstance → Path → RowId → Option RowId`. Phase 1 stub: always `none`.
- `Catherine.Check.check : Schema → TypedInstance → List Finding` (sorted by `Ord`). Phase 1 stub: `[]`.
- Kernel modules import nothing from `Lean`; `Catherine/Mirror.lean`, `Catherine/Parity.lean`, `Catherine/ParityMain.lean`, `Catherine/Axioms.lean` may `import Lean`.

### 3.3 Parity harness CLI

- `lake exe lean-parity [--emit-mirror DIR] [--check-build] [--corpus DIR] [--verbose] [--report PATH]`
- Workspace root: `LEAN_PARITY_ROOT`, else walk up from cwd for `Cargo.toml` + `crates/catherine-cli`.
- Rust binary: `CATHERINE_BIN`, else `<root>/target/debug/catherine`; absence is exit 2 with `cargo build -p catherine-cli` instruction.
- Per document: run `<bin> check <doc> --json`. Exit `0,3,4,5,6,7` → parse envelope `data.findings`; exit `1` → `skipped_parse` (record stderr first line); exit `2` or spawn failure → harness error.
- Comparison: normalized multiset of findings (kind + label + witness fields; violation variant inferred from witness fields: `equation` | `key` | `source_row` | `coproduct`). Never compare raw bytes or text output.
- `expected.json` in the fixture dir: if `"route": "presentation"`, the Rust parser is expected to reject the doc (assert exit 1) — Lean-only comparison against `expected.outcomes` (sorted by taxonomy order). With `--check-build`, compare Lean `build` reports/tables to `expected.{tables,dangling,typing_violations}` for every parity fixture whose route is `parse`.
- Report JSON: `{"ok", "corpus": {"total", "rust_checkable", "skipped_parse", "lean_only_presentation"}, "agreement": {"trivially_agreeing": [..], "findings_bearing_agreeing": N}, "divergences": [{"fixture", "rust": [...], "lean": [...], "side", "beads"}], "build_mismatches": [...]}`. Exit 0 iff `divergences == []` and (with `--check-build`) `build_mismatches == []`. Optional `lean/parity-dispositions.json` maps fixture → `{side, beads}`; it only annotates attribution, never suppresses a divergence (protocol: resolve, don't silence).

### 3.4 Theorem statements (task 7; names frozen)

- `Catherine.Proofs.soundness` — every `f ∈ check schema inst` carries its semantic witness: equation violation (applicable row, both composites succeed and differ, label = `labelFor`), key collision (two distinct ascending rows with an equal resolved tuple, label rule), coproduct coverage/disjointness (complete-image accounting as in `checks/coproducts.rs`), or dangling (cell is unresolved, not a typing-violated cell).
- `Catherine.Proofs.equation_completeness` — for any equation index and any applicable row (row of the lead arrow's source object) whose lhs/rhs composites both succeed and differ, `check` contains the exact `Violation(equation …)` finding with the status label.
- `Catherine.Proofs.dangling_discipline` — lemmas: dangling cells stay `none` in tables; no violation is reported for an equation path that short-circuits on a dangling/typing-violated cell; rows with an unresolved key cell are in no key-collision finding; a partial injection image suppresses coverage findings; a typing-violated cell is accounted exactly once (build report; its `none` cell is not a dangling finding); no cell is repaired.
- Each statement's docstring cites `openspec/specs/kernel-checks/spec.md` / `.../findings/spec.md`.
- Keys/coproduct **completeness** theorems are explicitly deferred (design Decision 3); do not state them.

## 4. TDD sequence (one arc; state transition evidence)

- **RED** — task 2: type surface + mirror + harness, semantics stubbed to empty. Observe `lake exe lean-parity` exit 1: every findings-bearing fixture diverges, clean fixtures trivially agree. Record the counts.
- **GREEN** — tasks 3–6: Schema/Findings, Instance, EvalPath/Check, then parity green (findings; `--check-build` for parity fixtures). No unexplained divergence; any Rust/Lean mismatch is resolved through Decision 4 and named in the report.
- **REFACTOR** — dedupe lemmas/report code; re-run both checks byte-identically.
- **PROOFS** — task 7 statements compile with `sorry` (axiom audit red), then tasks 8–10 remove `sorry` (audit green).
- **GATE** — task 12 ledger decision, task 13 README, ro5u review (max_critical = 0), task 14 validation.

## 5. Out of scope

No Cargo-dependency on Lean and no Lean dependency on Rust at runtime; no Mathlib; no parser in Lean (trusted projection only); no keys/coproduct completeness proofs; no bounded-search/migration/rule-lens theorems; no CLI/wasm/UI changes; no new finding kinds; no snapshot tests; no unrelated Rust edits. Rust changes only if parity proves a kernel defect — then file a bead and report, never patch quietly.

## 6. Files

`lean/**` (modules, lakefile, README, dispositions), `openspec/changes/add-lean-kernel-model/**` (tasks ticks, ro5u artifact), `justfile` + `.github/workflows/ci.yml` + `scripts/check-lean-axioms.sh` (task 11 only), `.beads/issues.jsonl` (export, orchestrator). Nothing else.