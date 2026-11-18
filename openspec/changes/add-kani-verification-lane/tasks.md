# Tasks: add-kani-verification-lane

One red→green→refactor arc per phase (beads `catherine-xdi`). RED here
means **provocation**: every harness must first demonstrate it can fail
(seeded defect → concrete counterexample) before its green proof counts.
The stop gate fires at the end of phase 1 and controls phase-2 spend
(design Decision 6): phase 2 runs only on the extend verdict. Definition
of done per phase: `just verify` over the landed harnesses, with the
three-valued outcome (proved / counterexample / undetermined) named per
harness and undetermined-without-disposition failing the lane.

## Phase 1 — provisioning + RED (lane and provokable harnesses first)

- [ ] 1. Provisioning + pin: install the pinned Kani release; commit the
      version pin beside the toolchain-pin discipline
      (`rust-toolchain.toml` untouched — Kani's bundled nightly only);
      provisioning instructions in the lane README; `just verify` recipe
      that checks the pin, runs `cargo kani` over declared harnesses,
      reports the three-valued outcome per harness, and exits nonzero on
      a confirmed counterexample, a harness error, or an undetermined
      outcome without a recorded disposition; `just ci` transitive
      closure stays Kani-free (spec scenario)
- [ ] 2. Harness scaffolding: `#[cfg(kani)]` harness modules in
      `catherine-core` (panic-freedom, witness soundness) and
      `catherine-analysis` (search honesty), synthesizing bounded
      nondeterministic `Schema`/`RawInstance` values — `kani::any()` over
      scalar fields composed into domain values by deterministic
      insertion loops (never over the map-bearing domain types
      themselves, never serde strings — design Decision 2); builds under
      `cargo kani` even while assertions are stubs
- [ ] 3. Provocation pass (RED evidence): each phase-1 harness seeded with
      a defect in a scratch copy produces a concrete Kani counterexample;
      the unseeded harness on unmutated kernel code does not; provocation
      runs on non-vacuous bounds — the bounded space admits at least one
      document exercising each check family. A harness that cannot be
      provoked is rewritten, not waved through

## Phase 1 — GREEN (panic-freedom + witness soundness)

- [ ] 4. Panic/overflow-freedom family: `TypedInstance::build`, path
      composition, and the full `check` walk (equation, key, coproduct,
      pullback checks) proven panic- and overflow-free over the bounded
      typed value space, with bounds stated and non-vacuous
      - Evidence: `cargo kani` reports **proved** per harness, or a
        counterexample resolved through the protocol (design Decision 5)
- [ ] 5. Witness-soundness family: for bounded documents whose `check`
      emits findings, every finding's witness independently re-evaluated
      fails as claimed — the Rust counterpart of
      `Catherine.Proofs.soundness`, verified on the shipped artifact
      - Evidence: **proved** per harness; any counterexample attributed
        (harness vs kernel) and resolved per Decision 5
- [ ] 6. Stop-gate review (fires here, after the first two harness
      families): extend / hold / retire per design Decision 6 — extend
      funds the phase-2 families, hold maintains at phase-1 scope, retire
      archives the harness modules and the verify recipe with lessons
      kept; ledger `decision` entry records what Kani bought (defects
      found, bounds reached, time cost) vs fixture-and-property
      assurance; `ledger-check ledger/` green — recorded even if the
      verdict is retire
- [ ] 7. Lane README: what the lane claims, what it does not (serde
      parsing, wasm32, Δ migration completeness — design Decision 4 out
      list), division of labor with Lean (semantic reference) and hegel
      (sampled properties), how to provision and run; records the gate
      verdict, the recorded disposition for any undetermined harness
      (raised bound or split obligation), and the Kani release that
      produced each recorded proof result (evidence provenance — may
      differ from the current pin after later bumps)

## Phase 2 — gated on the stop-gate extend verdict (taxonomy + search honesty)

- [ ] 8. Taxonomy-disjointness harness provoked, then proven: within
      bounded models, the five finding kinds are mutually exclusive per
      emitted finding (no finding reports two kinds); lane README updated
      with the family's bounds and results
- [ ] 9. Search-honesty harnesses provoked, then proven
      (`catherine-analysis`): `Contradiction` ⟹ exhaustion completed
      within bounds and no clean fully-resolved candidate in scope;
      `Witness` ⟹ fully resolved and clean; smallest-first minimality on
      small scopes where the whole candidate space fits the unwinding
      bound; lane README updated with the family's bounds and results
      - Evidence: **proved** per harness with bounds stated; any
        UNDETERMINED named loudly in the lane report (spec requirement)

## REFACTOR + gate + report

- [ ] 10. CI wiring: optional/nightly workflow step running `just verify`
      (pinned Kani release install, provisioning check in the recipe),
      never inside `just ci`; failure surfaces the per-harness outcome
      lines — skipped if the gate retired the lane
- [ ] 11. `openspec validate add-kani-verification-lane --strict`; bead
      status updated + fresh `bd export`
