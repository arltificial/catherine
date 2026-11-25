# Design: add-verus-verification-lane

Grounded in the smoke run of this cycle (2026-11-25): Verus release
`0.2026.11.22.1ce6a59` downloaded and run against rustup toolchain
`1.100.0` on this machine; 34 guide example files verified (29 pass, 5
documented-fail); a 1050-line proof model of the kernel walk
(`families.rs` key/equation logic + `compose.rs` filter) written and
verified — 29 VCs, 0 errors, `--rlimit 200`, ~2s per VC, ~15
proof-iteration cycles to green. The probe is **captured into this change** as
`probe-seed/lib.rs` (plus `probe-seed/spike-serde/` — the
external-crate-linking spike) because `/tmp` is tmpfs on this machine and
wiped at reboot; the capture task is done, and apply must seed from
`probe-seed/`, never re-derive from `/tmp`. Verus is **not** in the
Cargo workspace today; provisioning is a task, not an assumption.

## Goals / Non-Goals

**Goals:**
- Full functional contracts (`requires`/`ensures`, quantifier clauses)
  machine-checked over the kernel proof model, unbounded by SMT.
- Fixture parity keeping the model pinned to the shipped Rust kernel.
- A guide-example tripwire so toolchain bumps fail loudly.
- A nightly CI job with the same lane separation as the Kani gate.

**Non-Goals:**
- Proving the kernel sources directly — Verus proves the model; parity is
  the bridge (Decision 1).
- Phase-1 coverage of the coproduct and pullback check families — phase 1
  proves the equation/key walk; the stop gate may fund the rest.
- Replacing any existing mode: the Lean twin stays the semantic reference,
  hegel the sampled-property net, Kani the bounded checker on real Rust.
- Any runtime use of Verus or SMT in the engine (anti-goal, mirrors the
  Kani lane's Decision 1).

## Decision 1: Verus proves the model — parity fixtures are the honesty backbone

Kani harnesses execute the real Rust; Verus verifies `verus!`-annotated
code, and wrapping the kernel modules in `verus!` is a rewrite, not an
annotation pass (the engine leans on serde envelopes, `BTreeMap`
iteration, and iterator-heavy walks that Verus's subset neither parses
nor proves today). So the lane proves a **proof model**: spec mirrors of
the walk's semantics plus an executable walk that verifiably implements
them. The honest cost — a model can drift from the kernel — is paid down
by Decision 4's parity fixtures: the model's executable walk runs the
same fixture corpus as `catherine check`, and CI diffs the findings.
Alternative rejected: porting the kernel to Verus-compatible form to
prove the real thing — fails the effort budget (the probe's model-only
proof already took ~15 iterations; annotating serde/BTreeMap code would
be a kernel rewrite under a foreign compiler), and a drifted model
without parity is exactly the naked-assumption pattern the Lean twin's
parity oracle exists to kill.

## Decision 2: lane placement outside the Cargo build; toolchains side-by-side

`crates/catherine-verus` holds Verus source, fixtures, and provisioning —
no `Cargo.toml`, not a workspace member: the Verus driver is its own
rustc and would fight the workspace's `rust-toolchain.toml` pin
(`1.98.1`). Provisioning installs the rustup toolchain **named**
`1.100.0-x86_64-unknown-linux-gnu` (the Verus shim hard-codes the name —
observed in the smoke run: version-matching `stable` is not enough)
alongside the workspace toolchain, plus the pinned release zip recorded
in `verus-version.toml`. Precedent: Kani's bundled nightly rides outside
the workspace pin. `just verify-verus` checks the pin before running and
refuses any other installed release; `just ci` stays Verus-free.

> **Amendment at apply** (2026-11-25): the parity driver
> (`crates/catherine-verus/parity/`) is a **detached cargo project**
> with its own empty `[workspace]` table — the one `Cargo.toml` under
> the lane, required by the serde_json dependency Decision 4 settles.
> It is never a workspace member and never built by `cargo build`/`just
> ci`; the "not a workspace member" substance of this decision holds
> verbatim. Everything else under `crates/catherine-verus/` (model,
> probe-seed archive, fixtures) remains plain files with no manifest.

## Decision 3: proof architecture — declarative mirrors, return-style witnesses, encoded conventions

The probe's green proof fixes the architecture:

- **Recursive spec mirrors as the proof spine** (`collected_upto`,
  `emission`, `walk_pos`, `domain_count`): the loop invariant is stated
  against the mirror and stays quantifier-free; all ∀/∃ work lives in
  small induction lemmas, never in loop invariants.
- **`opaque` + `reveal`** on semantic spec fns: keeps them uninterpreted
  until a branch needs their guards, so trivial branch VCs stay trivial.
- **Return-style witnesses**: contracts that need "the position exists"
  return `-> (j: int)` from each branch instead of ensuring an ∃ — the
  probe's ensured-∃ form failed on Z3's MBQI witness synthesis even with
  the witness facts in context; the return-style rewrite verified
  deterministically.
- **Explicit casts at ghost/exec boundaries**: ghost-mode arithmetic
  promotes to `int` (`row - 1`), `Seq::len()` is `nat` while
  `Seq::index` takes `int` in this vstd — every mirror boundary carries
  an explicit `as usize` / `as int` cast with its range proof
  (`0 <= row - 1 < 2^64`) stated.
- **Trigger discipline**: non-recursive spec fns inline at use sites, so
  their names are dead as triggers; only recursive mirror fns carry
  `#![trigger ...]`, and every multi-variable quantifier states one
  trigger set covering all quantified variables (multiple attributes are
  alternative sets, not conjuncts — observed misfailing otherwise).
- **`assert forall…by` does not assume its premise**: premise-dependent
  lemmas are invoked inside an explicit `if premise {…}` in the by-block
  (observed failure).
- **Owned `Vec` sinks, not `&mut`**: `assert(x == v[i])` across a `&mut`
  push needed borrow-reveal gymnastics; an owned sink with
  `assert_forall`-checked equality against the mirror verified directly.

These become lane conventions in `docs/verus-lane.md`; new proof units
that violate them are rewritten, not special-cased.

## Decision 4: model parity via a Verus-built executable over the shared fixture corpus

`verus!` code is real Rust under the hood — and the spike settles the
linking question: `cargo verus build` compiles external crates under the
Verus driver and produces a runnable binary (spike `probe-seed/spike-serde/`:
serde_json + 22 transitive crates built, one verified model fn called from
a plain-Rust driver, binary parsed JSON and ran — `1 verified, 0 errors`).
The lane therefore builds a small parity binary that runs the model's
executable walk over the fixture corpus documents and prints findings;
`scripts/check-verus-lane.sh` diffs that output against `catherine check`
for the same corpus. Divergence → counterexample protocol. This mirrors
the Lean twin's parity oracle and is the lane's answer to "you proved a
model, not the code". Bounds: fixtures are the existing gap-suite corpus —
no new document format; the spike also fixes the integration pattern:
fixture IO and orchestration stay **plain Rust outside the `verus!` macro**
(inside it, even `println!` needs `assume_specification`), and verified
fns carry `requires` — which are **unchecked at external call sites**, so
the trusted driver must establish the model's preconditions; recorded as
an honesty bound of the parity harness. Fallback if a future Verus
release breaks crate linking: the kernel dumps a normalized sidecar the
Verus binary reads with zero-dependency code; revisit Decision 4 then.

## Decision 5: outcome taxonomy and disposition protocol mirror the Kani lane

Verus is sound and per-obligation: no UNDETERMINED — the honest
three-valued outcome is **verified** / **error** (counterexample or
compile failure) / **resource-exhausted** (rlimit). Exhausted units fail
the lane unless a disposition (raised `--rlimit`, split obligation) is
recorded in the lane README; silent exhaustion is the Verus-shaped
version of a naked "no violation found". Attribution: a failing proof
resolves as a model defect (contract misstated — fix model/proof) or a
kernel defect (parity confirms the model describes behavior the kernel
lacks — beads ticket + regression fixture). The guide tripwire is
provoked RED-first: flipping one recorded expected outcome must fail the
lane before the corpus counts.

## Decision 6: CI setup — a `verus` job beside the Kani job in the nightly

`.github/workflows/verify.yml` gains a `verus` job on the same triggers
(weekly cron + `workflow_dispatch`): checkout → pinned rustup toolchain
install (`rustup toolchain install 1.100.0 --profile minimal`) → pinned
release download (URL + sha256 assembled from `verus-version.toml`,
cached with `actions/cache` keyed on the pin so an upstream asset
deletion cannot go red spuriously; the provisioning script verifies the
checksum) → Install just (pinned version, matching the Kani job's pin) →
`just verify-verus`. Cost is bounded and measured (whole probe lane ≈ 2s per
VC, the 34-file guide corpus a few minutes on the runner); weekly cadence
matches the Kani job. `just ci` transitive closure stays Verus-free
(spec scenario pins it). A stop-gate retire archives the job together
with the model and recipe — a retired lane must not leave a failing
nightly behind (Kani lane precedent).

## Decision 7: stop gate reuses the existing ledger decision kind

Same mechanics as the Kani lane's Decision 6: after the walk-correctness
and parity families land, the gate records extend / hold / retire as a
terminal `decision` entry — extend funds phase 2 (compose/filter
functional spec family), hold maintains phase-1 scope, retire archives
with lessons kept. Given the probe's clean run, the honest prior: the
lane's yield is contracts and proof conventions, not kernel bugs; the
gate decides on evidence.

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| Model drifts from the kernel and the proofs rot into false comfort | Parity fixtures over the shared corpus (Decision 4) gate every lane run; divergence is loud, protocol-attributed |
| Proof-engineering cost per family exceeds the probe's (~15 iterations for one walk) | Phase 1 ports the already-green probe model (cheap); phase 2 is stop-gate funded; conventions (Decision 3) cut iteration count |
| Verus release churn breaks proofs or guide examples silently | Pin + guide tripwire corpus; a bump that changes any outcome fails the lane until expectations are deliberately re-recorded |
| Verus shim's hard-coded toolchain name couples provisioning to a rustup detail | `verus-version.toml` records both the release and the toolchain name; provisioning script installs exactly that; provisioning check runs before every lane run |
| Runner cost grows as families accumulate | Per-unit rlimits + measured budget in the lane README; weekly cadence; stop gate bounds total spend |
| Overlap confusion with Lean/Kani/hegel roles | Lane README states the division: Lean = semantic reference theorems, hegel = sampled properties, Kani = exhaustive-within-bounds on real Rust, Verus = full functional contracts on the model |
