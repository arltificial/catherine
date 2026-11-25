# Change: Add the Verus verification lane — functional-correctness proofs over the kernel model
> Beads: `catherine-gnc` · context: the Kani lane
> (`add-kani-verification-lane`) gives bounded crash-freedom on the real
> Rust; the Lean twin gives semantic theorems over the Lean mirror. The
> remaining gap: nothing machine-checks **full functional correctness
> with quantitative specs** (`requires`/`ensures`, universal and
> existential claims about the walk's output) against an SMT solver.
> Feasibility is proven, not assumed: a smoke run this cycle (Verus
> release `0.2026.11.22.1ce6a59` + rustup toolchain `1.100.0`) verified
> all 29 VCs of a proof model of the `families.rs`/`compose.rs` walk —
> mirror equality plus the full quantifier contract
> (∀i applicable ⟹ ∃j `out[j] == emission(i, r)`, sortedness) — and
> verified the Verus guide corpus itself (34 example files: 29 verified,
> 5 intentionally-failing teaching examples failing exactly as the guide
> documents).

## Why

The verification stack has two assurance modes and neither states
functional correctness as an SMT-checked contract: Kani proves
panic/overflow-freedom and witness soundness *within bounds* on the real
Rust, and the Lean twin proves theorems about a hand-written semantic
mirror with fixture parity as the only bridge to the shipped code. Verus
adds the missing mode: per-function `requires`/`ensures` contracts proven
unbounded by Z3, in the repo's own language (Rust syntax), with 1–4s
feedback per proof obligation (measured in the smoke run). The walk
model's proof also forced out a reusable set of proof conventions
(trigger discipline, return-style witnesses, ghost/exec cast boundaries)
that only exist today as tribal knowledge.

## What Changes

- New `verify-verus` lane: `crates/catherine-verus` holds Verus-annotated
  proof models of the kernel checks — deliberately **outside the Cargo
  workspace build** (the Verus driver is its own rustc; wrapping the
  serde/BTreeMap-heavy kernel modules in `verus!` is a rewrite, not an
  annotation pass) — verified directly by the pinned Verus release,
  runnable as `just verify-verus`, never inside `just ci` (same lane
  separation as the Kani and Lean gates).
- Pinned toolchain provisioning: `verus-version.toml` records the Verus
  release tag + zip asset; provisioning installs the rustup toolchain
  **named** `1.100.0` the Verus shim requires (side-installed;
  `rust-toolchain.toml` stays `1.98.1`, untouched — same discipline as
  Kani's bundled nightly).
- Proof families, in priority order:
  1. **Walk functional correctness** — the smoke run's 29-VC proof ported
     into the lane: spec mirrors (`collected_upto`, `emission`, `walk_pos`,
     `domain_count`), the output-equality invariant, and the quantifier
     clauses (every applicable emission position appears in the mirror
     output; positions sorted by emission key order).
  2. **Model-parity fixtures** — a Verus-built executable runs the proof
     model's executable walk over the same fixture corpus the kernel
     answers, and CI diffs its findings against `catherine check` output.
     This is the lane's honesty backbone (the Verus counterpart of the
     Lean parity oracle): Verus proves the *model*, parity fixtures keep
     the model pinned to the kernel.
  3. **Guide tripwire corpus** — the pinned Verus guide examples live as
     lane fixtures with expected outcomes (29 verified, 5
     documented-failure), so a Verus/toolchain bump that breaks guide
     examples fails the lane instead of surfacing as a mystery.
- Outcome taxonomy per proof unit mirrors the Kani lane's three-valued
  protocol: **verified** / **error** (concrete counterexample or compile
  failure) / **resource-exhausted** (rlimit) — exhausted without a
  recorded disposition fails the lane.
- Counterexample protocol: a parity divergence or failing VC resolves as
  exactly one of a **model defect** (fix the proof model) or a **Rust
  kernel defect** (beads ticket + regression fixture through the existing
  protocol, never silenced).
- Stop gate: after families 1–2 land, record extend / hold / retire as a
  ledger terminal `decision` entry (existing kind, validated by
  `catherine-ledger` since the Lean change) — the verdict gates phase-2
  spend (compose/filter functional spec family).
- CI setup included: `.github/workflows/verify.yml` gains a `verus` job
  (pinned release download + rustup `1.100.0` install + `just
  verify-verus`) on the same nightly/dispatch triggers as the Kani job;
  `scripts/check-verus-lane.sh` reports per-proof-unit outcomes;
  `just ci` transitive closure stays Verus-free (spec scenario).
- New openspec capability `verus-verification-lane`.

## Capabilities

### New Capabilities
- `verus-verification-lane`: functional-correctness assurance over the
  kernel proof model — lane separation (`verify-verus` vs `ci`), pinned
  release + side-toolchain provisioning, walk-correctness proof family,
  model-parity fixtures, guide tripwire corpus, outcome/counterexample
  protocols, and the ledger stop gate.

### Modified Capabilities

## Impact

- Affected code: new `crates/catherine-verus` (Verus source + fixtures;
  no Cargo.toml, not a workspace member — the Cargo build is untouched);
  `verus-version.toml`; `justfile` gains `verify-verus`;
  `scripts/check-verus-lane.sh`; `.github/workflows/verify.yml` gains the
  `verus` job; `docs/verus-lane.md` lane README. The kernel changes only
  if parity fixtures or proofs surface a real defect (filed separately
  through the protocol, out of this change's scope).
- Affected specs: `verus-verification-lane` (new). No existing
  requirement is modified — `kernel-checks`, `findings`, `composition`
  contracts are consumed as proof obligations, not edited; the
  `kani-verification-lane` spec is untouched (separate lane, separate
  toolchain, separate workflow job).
- Not affected: runtime dependencies (Verus never enters the Cargo
  workspace — no runtime surface), the Lean twin, hegel properties, the
  wasm target (Verus verifies native Rust semantics only), Kani harnesses.
- Beads: implements `catherine-gnc`.
