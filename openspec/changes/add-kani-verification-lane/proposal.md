# Change: Add the Kani verification lane — bounded model checking of the Rust kernel itself
> Beads: `catherine-xdi` · context: `add-lean-kernel-model` landed the Lean
> assurance twin (theorems over the Lean mirror + fixture parity oracle);
> this change closes the remaining gap: nothing machine-checks the *shipped
> Rust* exhaustively — the parity oracle samples the fixture corpus and
> hegel samples inputs (`hegeltest =0.53.1` dev-deps in `catherine-core`,
> `catherine-analysis`).

## Why

The kernel that agents actually invoke is verified only by sampling: the
`lean-parity` oracle compares normalized findings over the fixture corpus
(190 docs, divergence protocol), and hegel properties run on generated
inputs — neither exhausts any input space. Kani (AWS's Rust model checker)
adds the missing mode of assurance: **exhaustive-within-bounds checking of
the real Rust code** — panic/overflow-freedom over adversarial model
documents, witness soundness on the shipped artifact (not the Lean
mirror), and the search's honesty invariants stated as proofs. It
complements, never replaces: the Lean twin stays the semantic reference,
hegel stays the sampling net, and Kani is explicitly **not a runtime
engine** — `search.rs` is already a hand-rolled bounded checker over
instances, and "no solver" is a documented anti-goal.

## What Changes

- New `verify` lane: `cargo kani` proof harnesses in `crates/catherine-core`
  and `crates/catherine-analysis` behind a `kani` feature + `#[cfg(kani)]`
  harness modules — deliberately **outside `just ci`** (same treatment as
  the Lean gate: a toolchain every checkout can't assume), runnable as
  `just verify` and wired as a separate nightly/optional CI step.
- Pinned Kani release (committed version pin next to the toolchain pin
  discipline of `rust-toolchain.toml`; Kani bundles its own nightly, so the
  workspace's `1.98.1` pin is untouched).
- Harness families, in priority order:
  1. **Panic/overflow-freedom** — `check` walk, `TypedInstance::build`,
     path composition over nondeterministic bounded `Schema`/`RawInstance`
     values (`kani::any()` on typed domain values, never serde JSON
     strings — parsing stays Rust-side and path-explosion-prone).
  2. **Witness soundness on real Rust** — every emitted finding's witness,
     re-evaluated by an independent re-check, genuinely fails (the Rust
     twin of `Catherine.Proofs.soundness`).
  3. **Taxonomy disjointness** — the five finding kinds mutually exclusive
     per emitted finding within bounded models.
  4. **Search honesty** — `Contradiction` ⟹ exhaustion completed within
     bounds and no clean fully-resolved candidate in scope; `Witness` ⟹
     fully resolved and clean; smallest-first minimality on small scopes.
- Counterexample protocol (Lean Decision 4, adapted): a Kani-confirmed
  failure is exactly one of a **harness defect** or a **Rust kernel
  defect** (beads ticket + regression fixture, never silenced); Kani's
  UNDETERMINED (unwinding bound) is reported per harness, never hidden.
- Stop gate: after the first two harness families, record extend / hold /
  retire in the ledger using the existing terminal `decision` kind
  (no schema extension needed this time — `catherine-ledger` already
  validates it since the Lean change).
- New openspec capability `kani-verification-lane`.

## Capabilities

### New Capabilities
- `kani-verification-lane`: bounded model-checking assurance over the
  Rust kernel — harness lane separation (`verify` vs `ci`), pinned
  toolchain provisioning, panic-freedom and witness-soundness harness
  families, counterexample attribution protocol, and the ledger stop gate.

### Modified Capabilities

## Impact

- Affected code: new harness modules under `crates/catherine-core` and
  `crates/catherine-analysis` (cfg-gated, no runtime code path changes);
  `justfile` gains a `verify` recipe; `.github/workflows/ci.yml` gains an
  optional/nightly verify step; one committed Kani version pin. The kernel
  changes only if a harness finds a defect (filed separately through the
  protocol, out of this change's scope).
- Affected specs: `kani-verification-lane` (new). No existing requirement
  is modified — `kernel-checks`, `findings`, `analysis`, `migration`
  contracts are consumed as harness obligations, not edited.
- Not affected: runtime dependencies (Kani is dev/verification-only, the
  Cargo workspace gains no Kani dependency outside cfg(kani) harness code),
  the Lean twin, hegel properties, the wasm target (Kani does not target
  wasm32 — the wasm crate is out of harness scope).
- Beads: implements `catherine-xdi`.
