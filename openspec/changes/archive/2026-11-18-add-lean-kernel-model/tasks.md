# Tasks: add-lean-kernel-model

One red→green→refactor arc per phase (beads `catherine-xvh`). The Lean
project builds with `lake build` and the oracle with `lake exe
lean-parity`; both commands are the definition of done for their tasks.

## Phase 1 — RED (harness before semantics)

- [x] 1. `lean/` Lake project (Lean 4 std — no Mathlib), module
      stubs `Catherine/{Schema,Instance,EvalPath,Findings,Check}.lean`;
      provisioning included: elan install + committed `lean/lean-toolchain`
      pin so a fresh machine/CI runner reproduces the toolchain;
      `lake build` green on stubs
- [x] 2. Parity harness `lake exe lean-parity` (named apart from the
      existing wasm/ledger parity surfaces): loads canonical fixture
      documents from `fixtures/`, runs Rust `catherine check --json`
      (pinned rev, `--json` envelope), runs Lean `check`, compares
      normalized finding multisets — red gate: every *findings-bearing*
      fixture diverges under empty stubs; clean fixtures (e.g.
      `fixtures/parity/empty_instance`, zero outcomes) stay green and are
      recorded as trivially agreeing — the stub must never mute a red
      corpus
      - RED evidence: exits 1 over 190 docs — `rust_checkable=180`,
        `skipped_parse=10`, `trivially_agreeing=102`,
        `findings_bearing_agreeing=0`, `divergences=79`.

## Phase 1 — GREEN (formalization to parity)

- [x] 3. `Schema.lean` + `Findings.lean`: objects/arrows/paths/equations
      with constraint status; finding kinds with mandatory witnesses and
      the three constructible labels (`proved`/`illustrated`
      unrepresentable); taxonomy total order
- [x] 4. `Instance.lean`: dense row spaces, partial morphism tables,
      total typing-enforcing builder with the three landing places
      (value / dangling / typing violation), build-report ordering
- [x] 5. `EvalPath.lean` + `Check.lean`: `compose` with `None`
      short-circuit; the four finding families including
      typing-violation-cell accounting; sort under the taxonomy order
- [x] 6. Parity green over the full `fixtures/` corpus — zero unexplained
      divergence; any mismatch resolved through the divergence protocol
      (design Decision 4) with beads recorded
      - GREEN evidence: `lake exe lean-parity` exits 0 over 190 docs —
        `rust_checkable=180`, `skipped_parse=10`,
        `lean_only_presentation=1`, `trivially_agreeing=102`,
        `findings_bearing_agreeing=79`, `divergences=0`; the same command
        with `--check-build` exits 0 with `build_mismatches=0` across the
        seven `fixtures/parity/*` cases. No divergence was encountered, so
        no Decision 4 disposition was needed.

## Phase 2 — proofs + CI wiring (theorem statements first, then
## `sorry`-free)

- [x] 7. State the theorems in Lean (`Soundness`, `Completeness`,
      `DanglingDiscipline`) — compile with `sorry`; each statement cites
      the spec section it mirrors (`kernel-checks`, `findings`)
      - Statements: `Catherine.Proofs.soundness` (four-way semantic
        witness disjunction, plus the four exclusion theorems
        `check_no_naturality`/`check_no_redundant`/
        `check_no_contradiction`/`check_no_underconstraint`),
        `Catherine.Proofs.equation_completeness` (declared equation →
        applicable row → semantic violation ⇒ exact finding in `check`),
        `Catherine.Proofs.dangling_discipline` (the
        `DanglingDiscipline` bundle: no repair, no violation through an
        unresolved path, unresolved key cell in no collision, partial
        image suppresses coverage while complete-image disjointness is
        emitted, typing cell accounted once, carried-cell trichotomy);
        label lemma `Catherine.Proofs.labelFor_agrees` proved.
        `Catherine.Proofs.Basic` defines the semantic predicates.
      - RED evidence (intended proof-phase red): `lake env lean
        Catherine/Axioms.lean` exits 0 and prints
        `'Catherine.Proofs.soundness' depends on axioms: [propext,
        sorryAx, Classical.choice, Quot.sound]` (same for
        `equation_completeness` and `dangling_discipline`;
        `labelFor_agrees` does not depend on any axioms);
        `scripts/check-lean-axioms.sh` exits 1 naming all three
        theorems. `lake build` is green with `sorry` warnings only from
        `Catherine/Proofs/{Soundness,EquationCompleteness,
        DanglingDiscipline}.lean`; `lake exe lean-parity` and
        `--check-build` still exit 0 with `divergences=0` and
        `build_mismatches=0` (no semantic drift).
- [x] 8. Prove soundness: every emitted finding yields its semantic
      witness (violation on applicable row / unresolved cell accounting /
      duplicate key tuple / coproduct failure)
      - Evidence: `lean/Catherine/Proofs/Soundness.lean` is `sorry`-free;
        `lake env lean Catherine/Axioms.lean` prints `'Catherine.Proofs.soundness'
        depends on axioms: [propext, Classical.choice, Quot.sound]` (the
        four exclusion theorems likewise), and `scripts/check-lean-axioms.sh`
        exits 0 with no `sorryAx` and no `axiom` declaration.
- [x] 9. Prove equation completeness: semantic violation ⇒ reported,
      with the accepted→`executed` / proposed→`hypothesized` label
      agreement lemma
      - Evidence: `lean/Catherine/Proofs/EquationCompleteness.lean` is
        `sorry`-free; the audit prints `'Catherine.Proofs.equation_completeness'
        depends on axioms: [propext, Classical.choice, Quot.sound]` and
        `'Catherine.Proofs.labelFor_agrees' does not depend on any axioms`
        (the accepted/proposed/never-merged lemmas use at most `propext`);
        `scripts/check-lean-axioms.sh` exits 0.
- [x] 10. Prove dangling discipline: no repair, no violation report, no
      key-collision membership, no coverage participation; typing-
      violated cells accounted exactly once
      - Evidence: `lean/Catherine/Proofs/DanglingDiscipline.lean` is
        `sorry`-free; the audit prints `'Catherine.Proofs.dangling_discipline'
        depends on axioms: [propext, Classical.choice, Quot.sound]`, and
        `scripts/check-lean-axioms.sh` exits 0 with no `sorryAx` and no
        `axiom` declaration.
- [x] 11. CI wiring: elan install + `lake build` + `lean-parity` oracle +
      `sorry`-free check (`#print axioms` audit) as a CI step alongside
      the Rust suite
      - Evidence: `.github/workflows/ci.yml` installs elan pinned by the
        committed `lean/lean-toolchain` and runs `just lean`; the `lean`
        recipe in `justfile` is `lake build` + `lake exe lean-parity` +
        `scripts/check-lean-axioms.sh`, and that script exits 0.

## REFACTOR + gate + report

- [x] 12. Stop-gate review: extend / hold / retire per design Decision
      5; ledger entry recording the decision and what the proofs bought
      (defects found, holes found) vs fixture-only assurance
      - Decision: **hold** (maintain at the landed phase-2 scope).
        Recorded as `ledger/lean/0001-cbef9219.json`
        (`sha256:cbef92198253dbb94ea46ff010ab51d4d3dd44711e2f6121dafbbc5559469802`);
        `ledger-check ledger/lean` exits 0. Accounting: parity pinned 180
        Rust-compared documents (79 findings-bearing, 102 trivially
        agreeing, 0 divergences, 0 build mismatches); the three sorry-free
        theorems added the universal contract statements and surfaced no
        defect and no hole, so no extension is funded and nothing is
        archived. Extend/retire counterfactuals are in the entry.
- [x] 13. README in `lean/`: what the twin claims, what it does not
      (keys/coproduct completeness, search, migration), how to run the
      oracle, divergence protocol
      - `lean/README.md`: claims (normalized-multiset parity over the
        dynamic corpus, three theorems, axiom audit), non-claims
        (keys/coproduct completeness, search/Δ-migration/rule lens, Lean
        parser, CLI/wasm surface), run instructions and corpus counts
        verified against a fresh oracle run, divergence protocol, stop-gate
        pointer to `ledger/lean/0001-cbef9219.json`.
- [x] 14. `openspec validate add-lean-kernel-model --strict`; bead
      status updated + fresh `bd export`
      - `openspec validate add-lean-kernel-model --strict`: valid.
        `catherine-xvh` closed; `bd export` refreshed. Full gates at
        final HEAD: `lake build`, `lean-parity` and `--check-build`
        (divergences=0, build_mismatches=0), axiom audit exit 0,
        `just ci` exit 0, `pretender check --mode gate` exit 0,
        `ah check` clean.

## Phase 3 — pullback catch-up (the 1.6 kernel grew after the twin landed)

- [x] 15. Formalize declared pullbacks in the twin: `Schema.Pullback` plus
      the mirror projection/encoding/decoding, the three findings witness
      arms (`PullbackWitness`: commutes, covers, unique) with the taxonomy
      order, `Check`'s commutes/covers/unique walk (dangling-leg rows
      withhold their own verdicts and coverage; a dangling cospan cell
      removes its pair from the decidable fiber), and the soundness
      disjunct with its semantic predicates
      - RED evidence: `lake exe lean-parity` aborted before any comparison
        with `fixtures/parity/pullback_square/expected.json: violation
        witness has no recognized variant` — the whole corpus went
        unverified (189 documents never compared).
      - GREEN evidence: `lake exe lean-parity` and `lean-parity
        --check-build` exit 0 over the then-current corpus —
        `total=193 rust_checkable=183 skipped_parse=10
        lean_only_presentation=1 trivially_agreeing=103
        findings_bearing_agreeing=81 divergences=0 build_mismatches=0`
        (`parity/pullback_square` now agrees: one commutes violation at
        apex row 0, both composed values 0 and 1). The commutes arm is
        the producer exercised by the corpus; covers and unique are
        modeled and proof-covered, with their own parity fixtures left to
        the corpus as it grows.
      - Proof evidence: `Catherine.Proofs.soundness` is a five-way
        disjunction; `mem_pullbackFindings_forward` feeds it;
        `scripts/check-lean-axioms.sh` exits 0 with 18 audited
        declarations, no `sorryAx`, no axiom declaration.
