# Kani verification lane

Bounded model checking of the shipped Rust kernel (`catherine-core`,
`catherine-analysis`) — exhaustive-within-bounds assurance the sampled
nets (fixture-corpus parity, hegel properties) cannot give. Run: `just
verify`. The lane is deliberately **outside `just ci`** (the Lean-gate
precedent: a toolchain every checkout cannot assume must not gate every
commit). Kani is **not a runtime engine** — the anti-goal "no solver"
stands; harnesses only, `cfg(kani)` code, zero runtime surface.

## Division of labor

- **Lean twin** (`lean/`, `just lean`): the semantic reference — theorems
  over the Lean mirror plus the fixture-corpus parity oracle.
- **hegel** (dev-deps, `hegeltest =0.53.1`): sampled properties over
  generated inputs.
- **Kani** (this lane): exhaustive-within-bounds over real Rust —
  panic/overflow-freedom, witness soundness on the shipped artifact,
  taxonomy disjointness, search honesty.

## What the lane claims — and what it does not

**What it claims today: nothing proved.** Every declared harness is
undetermined with a recorded split obligation (see below); the lane is
maintained as provisioning + scaffolding + provocation machinery, ready
to produce proofs when the recorded toolchain limitation clears. This
section states the claims the harnesses are WRITTEN to make — none of
them is currently backed by a completed proof, and no harness result
may be cited as assurance.

Designed claims (per harness, bounds stated in
`crates/catherine-core/src/kani_harnesses{,/}`): the kernel's post-parse
execution path — `TypedInstance::build`, path composition, the full
`check` walk — panic- and overflow-free within the declared harness
bounds, with non-vacuous bounds (each check family exercised by at least
one document in the bounded space); every finding `check` emits carries
a witness that independently re-evaluates as claimed.

Does **not** claim: serde parsing (parse stays Rust-side, pinned by
golden fixtures and the model-document spec — Decision 2), the wasm
crate (Kani does not target wasm32), Δ migration reindexing
completeness, or anything outside the declared bounds. An
undetermined outcome is a legal per-harness result but fails the lane
unless a disposition is recorded below.

## Provisioning

1. Read the pin: `kani-version.toml` (`release`, `bundled_nightly`).
2. Install exactly that release:
   `cargo install --locked kani-verifier@<release> && cargo kani setup`
   — this installs `cargo-kani`/`kani` and the release's bundled
   nightly (`rust-toolchain.toml` is never modified).
3. `just verify` re-checks the pin against the installed release before
   running and fails loudly on a mismatch.

CI: a separate optional/nightly workflow step only; `just ci` and its
transitive closure never invoke `cargo kani`.

## Per-harness results and provenance

Killed by the spec: every recorded proof result names the Kani release
that produced it, so pin bumps never silently re-date recorded evidence.

Release recorded: **Kani 0.68.0 (CBMC 6.11.0, bundled nightly
2026-08-21)** — the release the provisioning and scaffolding below were
validated under.

**2026-11-18 status: every declared harness is UNDETERMINED at the
practical budget.** No proof result is recorded; nothing is presented
as proved. Evidence (executed this session):

- The whole-kernel `check`-walk harness at unwind 32 ran 43 minutes
  without completing; a build-only harness at unwind 16 ran 13 minutes
  without completing; per-family minimal-bounds partitioning (11
  harnesses, 7-12 symbolic cells each) did not change the shape of the
  run.
- Controlled probe: a standalone crate whose entire harness is four
  `String`-keyed `BTreeMap` inserts plus one `kani::any::<u8>` assert —
  no catherine code, no symbolic maps — did not complete within
  minutes either; its trace is `memcmp` unrolling to the unwind cap
  inside the `String`-comparison model.

Conclusion recorded honestly: Kani 0.68's model of `String`-keyed
`std::collections::BTreeMap` operations (memcmp-based key comparison
over modeled buffers, node-split bookkeeping) dominates the symbolic
execution on this hardware, at every harness granularity this lane
tried. The kernel's own API surface (`Ident(String)` keys,
`BTreeMap`-shaped tables) is fixed — the cost is kernel-side heap
modeling, not harness-side input modeling.

## Undetermined dispositions

Every declared harness carries the same split obligation
(`<harness>: undetermined — split obligation: toolchain limitation,
revisit trigger recorded`):

- build_panic_freedom: undetermined — split obligation: toolchain
  limitation (see above); revisit on a Kani release whose
  `String`/`BTreeMap` models do not unroll memcmp to the unwind cap, or
  after a kernel-side ident-interning change (usize keys internally,
  `Ident` at the API edge) — that change would be its own beads ticket.
- check_sortedness_panic_freedom: undetermined — split obligation: same
  toolchain limitation; revisit triggers as above.
- check_equation_panic_freedom: undetermined — split obligation: same.
- check_key_panic_freedom: undetermined — split obligation: same.
- check_coproduct_panic_freedom: undetermined — split obligation: same.
- check_pullback_panic_freedom: undetermined — split obligation: same.
- soundness_equation: undetermined — split obligation: same.
- soundness_key: undetermined — split obligation: same.
- soundness_coproduct: undetermined — split obligation: same.
- soundness_pullback: undetermined — split obligation: same.
- soundness_dangling: undetermined — split obligation: same.

Phase-2 families (taxonomy disjointness, search honesty) were **not**
written: the stop-gate verdict (ledger `ledger/kani/`) does not fund
them, and every harness above shares the same wall — more harnesses
would multiply unverifiable spend, not assurance. The provocation pass
(task 3 RED evidence) is blocked by the same wall: a counterexample
requires the same full symbolic exploration a proof requires, and no
budget produced one; the provocation machinery
(`scripts/provoke-kani-harnesses.sh`, one anchored seed per harness) is
landed so the pass runs as soon as the revisit trigger fires.
