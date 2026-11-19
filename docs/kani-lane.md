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

## Safe invocation (catherine-amh)

An uncapped crate-wide `cargo kani` run OOM-killed the workstation on
2026-11-18 (CBMC at 15.4 GB anon RSS; enclosing process at 41.8 GB +
5 GB swap on a 60 GB box; hard reset). The lane is budget-bounded now:

- **Every harness runs inside a systemd user scope** capped at
  `KANI_MEMORY_BUDGET_MB` (default 8192, swap disabled). The
  OOM-killer's target is the harness run, never the machine; a host
  without a working `systemd-run --user` refuses to run the lane
  rather than run unbounded.
- **`--harness-timeout` needs `-Z unstable-options`** (Kani 0.68.0);
  without it every `cargo kani` invocation errors instantly (exit 2,
  no `VERIFICATION:-` line). The 2026-11-18 timing pass' blanket
  `exit=1` in 1-2s per harness was this flag bug, not harness
  behavior.
- **Timeouts and budget-kills are UNDETERMINED, not counterexamples.**
  CBMC emits `VERIFICATION:- FAILED` both for a found counterexample
  and for "CBMC timed out"; the lane discriminates on the timeout
  banner, and a SIGKILL at the memory cap (rc 137) is likewise
  undetermined. Both still require a recorded disposition below or
  the lane fails.
- **Maintenance subsets**: `KANI_HARNESS_FILTER=<regex>` runs part of
  the declared list under identical enforcement and the identical
  disposition rule.
- **Per-harness resource record**: each lane run rewrites
  `target/kani-timings.txt` with exit code, peak RSS, and wall per
  harness.

Measured under the enforced budget (Kani 0.68.0 / CBMC 6.11.0,
`--default-unwind 16`, 300s wall, 8 GB cap): **all 11 declared
harnesses time out at ~5.8-6.3 GB peak RSS** - none completes, none
approaches the cap. The blowup is uniform (see the status note below
for the modeled-cost analysis); a bigger cap only buys a bigger
formula, so 8 GB / 300s stays the lane standard until the split
obligation clears.


## 2026-11-19: the interning arc (beads catherine-6g2) — construction proves, build/check still held

The revisit trigger recorded in the dispositions below was executed: a
kernel-side interned execution core (`src/engine/`, decomposed under the
complexity caps into `mod`/`compile`/`families`/`render`/`lane`) — dense
ids in canonical order, `Vec`-shaped tables, no `Ident`-keyed map on any
proven path, findings ordered by pure-int sort keys exactly equivalent to
`Finding::cmp` (id order is name order by construction). The parsed
surface is behaviorally unchanged and pinned by the full test suite
(169 core tests, the `ah check` contract suite, fixtures, parity);
`TypedInstance::build` delegates its classification to the engine, so the
kernel has one build implementation.

Measured probe matrix (Kani 0.68.0 / CBMC 6.11.0, unwind 16) — replaces
the 0001-era picture:

| Op class under the solver | Verdict |
|---|---|
| `BTreeMap` with symbolic content, any key type (`u32` included) | fatal (harness timeout) |
| `&str` in a sort comparator | fatal (memcmp churns to the unwind cap, even fully concrete) |
| `String` alloc/drop/swap in the harness path | fatal (even ~a dozen concrete names) |
| `Vec`-of-`Vec` over symbolic scalars, concrete loop bounds | proves in seconds |
| `Option<struct>` payloads | free |
| lane construction (`EngineSchema::lane` + `instance_for`) | **verifies ~26 s end-to-end** |

Construction is therefore the first in-budget verification this lane has
produced. `engine_build` and the check walkers over symbolic cells remain
undetermined: `probe_build_only` misses even a 380 s wall while the
near-identical standalone probe completes in ~15 s — the residual cost is
interprocedural `Vec` machinery over symbolic-length content in the
in-crate context (zero unwind markers; preallocation and branchless
appends did not clear it). Nothing is cited as proved; the harness
dispositions below carry the refined split obligation.

Revisit triggers (replace the 0001 ones): (1) a Kani release where
`vec_push_within_capacity` stabilizes (retiring the capacity-asserted
append helper) or whose alloc modeling stops exploring growth branches on
symbolic lengths — re-run the build probe first; (2) a standalone
verification crate wrapping the engine behind a `pub` API, if in-crate
symex overhead is confirmed as the delta (the standalone 10x gap is the
evidence to chase); (3) solver tuning via `--cbmc-args` if a profile
shows solver rather than symex time.

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
- Resource sweep under the enforced budget (catherine-amh,

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
  2026-11-19): all 11 harnesses, per-harness invocation, unwind 16,
  300s wall, 8 GB scope - every one times out at ~5.8-6.3 GB peak
  RSS (records in `target/kani-timings.txt`, logs and time outputs
  in `target/kani-amh/`).
