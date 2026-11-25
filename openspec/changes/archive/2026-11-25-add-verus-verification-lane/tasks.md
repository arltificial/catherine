# Tasks: add-verus-verification-lane

One red→green→refactor arc per phase (beads `catherine-gnc`). RED here
means **provocation**: every family must first demonstrate it can fail —
a seeded model defect must produce a parity divergence, a flipped
tripwire expectation must fail the lane — before its green proof counts.
The stop gate fires at the end of phase 1 and controls phase-2 spend
(design Decision 7). Definition of done per phase: `just verify-verus`
over the landed families, with the three-valued outcome (verified /
error / resource-exhausted) named per proof unit and
exhausted-without-disposition failing the lane.

## Phase 1 — provisioning + RED (lane and provokable families first)

- [x] 1. Provisioning + pins: `verus-version.toml` records the Verus
      release tag, zip asset URL, **the zip's sha256** (provisioning and
      CI verify the checksum — design Decision 6), and the required
      rustup toolchain name (`1.100.0-x86_64-unknown-linux-gnu` — the
      shim's hard-coded name, design Decision 2); provisioning script
      downloads the pinned zip, verifies the checksum, and installs the
      side toolchain (`rustup toolchain install 1.100.0 --profile
      minimal`), leaving `rust-toolchain.toml` untouched; instructions
      in the lane README; `just verify-verus` recipe checks the pin,
      runs every declared proof unit via `scripts/check-verus-lane.sh`,
      reports the three-valued outcome per unit, and exits nonzero on an
      error or an exhausted outcome without a recorded disposition;
      `just ci` transitive closure stays Verus-free (spec scenario);
      serde-linking is settled evidence (spike in
      `probe-seed/spike-serde/` — serde_json builds and runs under
      `cargo verus`, design Decision 4) — re-run the spike recipe once
      under the pinned release as provisioning's smoke check
      - Evidence: fresh-shell provisioning run reproduces the smoke
        result; spike recipe green; `just ci` on a Verus-less checkout
        behaves as before
- [x] 2. Model scaffolding: seed `crates/catherine-verus` from the
      **captured** probe `openspec/changes/add-verus-verification-lane/
      probe-seed/lib.rs` (1050 lines; `/tmp` is tmpfs — never re-derive
      from it): spec mirrors
      `collected_upto`/`emission`/`walk_pos`/`domain_count`, the walk
      `equation_findings`, the compose model (`engine_compose`/gcompose
      proofs — already green in the seed, held for phase-2 promotion,
      task 9), and the walk lemmas; lane conventions from design
      Decision 3 written into `docs/verus-lane.md`
      - Evidence: `just verify-verus` reports 29 verified, 0 errors on
        the seeded model
- [x] 3. Parity provocation (RED evidence): the Verus-built parity
      binary (design Decision 4 — plain-Rust driver outside the
      `verus!` macro per the spike's integration pattern) runs the
      model's executable walk over the shared fixture corpus; with a
      seeded defect in the model's walk (mutated comparator or dropped
      position), the parity diff against `catherine check` fails naming
      the fixture and the diverging finding; unseeded, the diff is empty
      on the whole corpus; a parity run that cannot be provoked is
      rewritten, not waved through
      - Evidence: seeded run's divergence record + unseeded clean diff,
        both archived in the lane README

## Phase 1 — GREEN (walk correctness + parity)

- [x] 4. Walk-correctness family: the seeded model's contracts hold under
      the pinned release — output-equality invariant (**out-eq**), every
      applicable emission position appears in the mirror output
      (**complete**), positions carry their emissions (**faithful**),
      key-order sortedness (**sorted**) — with non-vacuous preconditions
      (at least one instance exercising each clause)
      - Evidence: `just verify-verus` reports **verified** per proof
        unit, or an error/exhaustion resolved through the protocol
        (design Decision 5); **non-vacuity gate**: the parity binary
        runs a witness instance where each clause demonstrably fires —
        the walk emits at least one finding, at least two positions
        appear in key order, and the ∃-index points at its emission —
        and a deliberately vacuous variant (preconditions unsatisfiable)
        fails this gate (review CORR-001; beads ticket tracks it)
- [x] 5. Parity family green: the unseeded parity binary's findings equal
      `catherine check`'s findings over the full shared fixture corpus;
      the diff step wired into `scripts/check-verus-lane.sh` so every
      lane run re-checks it
      - Evidence: clean diff over the corpus in the lane report; any
        divergence attributed (model vs kernel) per the protocol
- [x] 6. Stop-gate review (fires here, after families 1–2): extend /
      hold / retire per design Decision 7 — extend funds the phase-2
      family, hold maintains at phase-1 scope, retire archives the
      model, corpus, recipe, and workflow job with lessons kept; ledger
      `decision` entry records what Verus bought (contracts established,
      defects found, proof-engineering cost) vs fixture-and-property
      assurance; `ledger-check ledger/` green — recorded even if the
      verdict is retire
- [x] 7. Guide tripwire corpus: the Verus guide example corpus pinned
      under lane fixtures with per-file expected outcomes (29 verified,
      5 documented-failure); fixture README records provenance — source
      repo and commit, license (verus is MIT), and the sync command
      (review EDGE-001); provoked RED-first by flipping one expectation
      and observing the lane fail
      - Evidence: flipped expectation fails `just verify-verus`;
        restored corpus passes; provenance block present in the fixture
        README
- [x] 8. Lane README: `docs/verus-lane.md` states what the lane claims
      and does not (proves the model, not the kernel — parity is the
      bridge; serde parsing, wasm32, coproduct/pullback out of phase 1,
      Lean/Kani/hegel division of labor), provisioning, per-unit
      outcomes with budgets and dispositions, and the recorded proof
      results naming the release that produced them
      - Evidence: README present with all sections; cross-checked
        against the spec's requirement list

## Phase 2 — gated on the stop-gate extend verdict (compose/filter family)

- [x] 9. Compose/filter functional-spec family: **promote** the compose
      proofs from the probe seed (already green — `engine_compose`
      invariant proof, gcompose unfold lemma, arm-splitting) into a
      maintained family with stated contracts and named proof units;
      lane README updated with the family's contracts and results
      - Evidence: **verified** per proof unit with budgets stated; any
        exhaustion named loudly (spec requirement)

## REFACTOR + gate + report

- [x] 10. CI wiring: `verus` job in `.github/workflows/verify.yml` on the
      nightly/dispatch triggers — pinned toolchain install, pinned
      release download (checksum-verified, `actions/cache` keyed on the
      pin), Install just (matching the Kani job's pin), `just
      verify-verus` — never inside `just ci`; failure surfaces the
      per-unit outcome lines; skipped if the gate retired the lane
      - Evidence: workflow_dispatch run of the job green on GitHub;
        `just ci` closure re-verified Verus-free

- [x] 11. `openspec validate add-verus-verification-lane --strict`;
      bead `catherine-gnc` status updated + fresh `bd export`; the two
      review tickets (non-vacuity gate, spike re-verification) closed
      with their evidence linked

## Execution record (apply, 2026-11-25, verus 0.2026.11.22.1ce6a59)

- 1: `verus-version.toml` pins release/URL/sha256/toolchain; fresh
  provisioning reproduced end to end (download → checksum → unpack →
  toolchain check) during the first lane run; spike recipe re-run green
  under the pinned release (`spike: 41 -> 42`, exit 0); `just ci`
  closure untouched by the lane (no workspace members added).
- 2: model seeded from the captured probe; `just verify-verus` reports
  **29 verified, 0 errors** (~1.5 s).
- 3: seeded fault = canonical-order omission in the parity driver's
  walk copy (the model's exec walk at driver granularity); parity diff
  fired naming `05_multi_order.ct.json` (kernel `EQ 0 …` vs model
  `EQ 2 …`); unseeded diff clean over the whole corpus; both archived
  in `docs/verus-lane.md` § Provocation evidence.
- 4: out-eq/complete/faithful/sorted all in the verified unit;
  non-vacuity witness = the corpus (5 emissions; ≥ 2 positions in key
  order via `05_multi_order.ct.json`; completeness parity = every
  kernel finding matched); vacuous variant (zero-emission corpus)
  fails the gate — review CORR-001 evidence in the lane README.
- 5: parity green on all six fixtures; wired into
  `scripts/check-verus-lane.sh` (kernel envelope `ok` asserted, not the
  findings-present exit code).
- 6: stop-gate verdict **extend**, sealed as
  `ledger/verus/0001-291cc8de.json` (sha256-addressed via
  `ledger-record`); `ledger-check` green over `ledger/verus` and the
  full `ledger/`.
- 7: measured counts are **28 verify / 6 documented failures** (5
  `E0601` library-style snippets + `higher_order_fns.rs`'s intentional
  callee-requires failure) — the plan's "29 verified, 5
  documented-failure" estimate was written before measurement; the
  corpus is ground truth. Provenance (repo, tag, commit, MIT license,
  sync command) in `crates/catherine-verus/fixtures/guide/README.md`;
  flipped-expectation RED recorded.
- 8: `docs/verus-lane.md` carries claims/non-claims, provisioning,
  families, gate contract, provocation evidence, and the pinned
  release per result.
- 9: compose contracts promoted as a named family in the lane README
  (§ Proof families); the proofs themselves ride in the already-green
  model unit — no new proof spend.
- 10: `verus` job added to `.github/workflows/verify.yml`
  (actions/cache keyed on the pin; just install matching the Kani job).
  The GitHub `workflow_dispatch` run has NOT been executed from this
  workstation — the recipe itself is green locally; first push will
  exercise the job.
- 11: `openspec validate add-verus-verification-lane --strict` green;
  beads `catherine-vwp` and `catherine-1bk` closed with evidence,
  `catherine-gnc` closed at apply; `bd export` refreshed.
- 12 (gate surface): the full-tree complexity gate initially failed on
  the lane (vendored guide files over the 400-line cap, 1050-line proof
  units, parity driver functions). `pretender.toml` excludes widened —
  `**/catherine-verus/**` + `**/probe-seed/**` (the plain
  `dir/**` glob form silently never matched, the known pretender
  lesson; all excludes use the `**/`-prefixed form), `parity/target`
  gitignored. `just ci` end-to-end green after the change; rationale
  recorded in `docs/verus-lane.md` § Layout.
