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
`crates/catherine-kani/src/lib.rs`, with `shapes.rs`/`soundness.rs`),
running over the wrapped engine in `crates/catherine-engine` — the
dependency-light interned execution core (`model`, the epistemic
label, and the `engine`), the only kernel code the lane compiles:
`cargo kani -p catherine-kani` pays for the kernel's execution path
and nothing else (ledger `0003`). The claims: the kernel's
post-parse execution path — `TypedInstance::build`, path
composition, the full `check` walk — panic- and overflow-free
within the declared harness bounds, with non-vacuous bounds (each
check family exercised by at least one document in the bounded
space); every finding `check` emits carries a witness that
independently re-evaluates as claimed.

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
  banner, and a kill at the memory cap is likewise undetermined — the
  kernel's cgroup SIGKILL (rc 137) or the scope-level systemd TERM
  that precedes it (rc 143; seen 2026-11-23 on `soundness_equation` at
  the reduction wall). Both still require a recorded disposition below
  or the lane fails.
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

## 2026-11-19 continuation (beads catherine-6g2): the engine-crate split and the solver-shaped engine

The recorded revisit triggers were executed in order. Trigger 2 first
(the standalone-crate wrap): the engine moved out of the kernel into
`crates/catherine-engine` (`model`, the epistemic label, `engine`;
serde behind an optional feature, off for the lane) and the harnesses
moved to a new workspace member `crates/catherine-kani` that depends
only on it — `cargo kani -p catherine-kani` compiles the execution
core, the harnesses, and nothing else. `catherine-core` wraps both
modules verbatim (`pub use catherine_engine::{engine, model}`), so
every downstream path is unchanged; the parsed-flavor adapters
(`compile_typed`, rendering, the epistemic-label re-export) stayed in
the kernel, which keeps the base crate dependency-light. Full suite,
clippy, fmt green across the new layout.

Then the wall was profiled (CBMC verbosity, in-crate vs standalone
replicas) and the engine was made solver-shaped, one measured toxin at
a time:

- **Slice sorts**: `sort_unstable`/`sort_by_key` internals
  (`smallsort::insert_tail`, `heapsort::sift_down`,
  `swap_nonoverlapping_chunks`) unwind to the cap under symbolic
  length — the dominant VCC source in every walker. Replaced by a
  bounded insertion sort (`engine::insertion_sort_by`, plain
  single-element moves, capacity-asserted) and a plain dedup
  (`dedup_sorted`) at every engine sort site. Emitted order is
  unchanged: each family's sort keys are distinct within its domain.
- **Binary-search membership** over symbolically-populated vectors
  (dangling exclusion, coproduct images, pullback projections):
  replaced with linear scans — the domains are ≤ a handful of entries.
- **Iterator-collect chains over symbolic content** (`instance_for`'s
  cell synthesis, lane entry tables): rewritten as plain loops with
  capacity-asserted appends (the lane.rs lesson, now applied
  everywhere the lane compiles).
- **Build report vectors**: plain `Vec::push` measured 3.7x the wall
  time and 3x the memory of the capacity-asserted append standalone
  (`build_reports_plain_push` 56.2 s / 2.95 GB vs
  `build_reports_bounded_push` 15.2 s / 0.96 GB); `engine_build` now
  uses `push_bounded` for both report vectors.
- **Witness soundness upgraded to for-all**: the sampling form
  (`kani::any` index, `assume(!empty)`, symbolic `% len`) is replaced
  by a for-all loop over emitted findings — a strictly stronger claim
  (every emitted witness re-evaluates, not one sampled witness) with a
  cheaper formula (no division, no symbolic heap indexing).
- **Harness-side drop glue**: `mem::forget` at harness tails (none of
  the structures carries a panicking drop; drop/dealloc modeling
  dominated symex — 154 s of a 300 s budget on build alone).
- **Per-harness unwind**: the all-families sortedness harness declares
  `#[kani::unwind(48)]` (its finding space exceeds the lane default's
  16 iterations for the sort loops); every other harness stays within
  16.

Measured state after all of it: the machinery completes — symex for
`build_panic_freedom` runs to conversion (152 s symex, 24 k VCCs
remaining after slicing) — but the solve phase still exceeds the 300 s
wall, and the walkers+soundness formulas remain above it. The lane
outcome is unchanged (all 11 harnesses undetermined, dispositions
recorded), the parsed kernel is byte-identical and fully green, and
the construction proof stands. The refined split obligation: symbolic
execution volume (allocation modeling + VCC count), not toolchain
modeling — the next lever is cfg(kani)-shaped arena structures or a
solver-phase budget decision, both recorded in the ledger.

## 2026-11-21: the wall is propositional-reduction memory (beads catherine-6g2)

Four instrumented runs of `build_panic_freedom` (systemd memcg caps,
logs in `target/kani-ab/`) correct this arc's "the solve phase still
exceeds the 300 s wall": the solver phase is never reached. The shape
is identical at every cap — symex clean (88.7-94.7 s, zero unwind
warnings), `Generated 72353 VCC(s), 24066 remaining after
simplification` (the 24 k figure, reproduced exactly), Convert SSA
~5.5 s, then the propositional reduction is kernel-OOM-killed
(constraint MEMCG, ~100% CPU): at 8 GB with the default solver, at
8 GB with kissat, at 8 GB with kissat plus `--slice-formula`, and at
24 GB with kissat (diagnostic cap). No solver runs in any capped run —
the "solve exceeds the wall" attribution was an inference from a
killed log whose last line is `Runtime Convert SSA`, the same
reduction-death signature. Ledger `0004-c7563980` (supersedes 0003)
records the consequences: solver tuning is retired as a lever at this
formula size, the budget stays closed (a bigger cap buys only a bigger
reduction — 24 GB still OOM), and the live lever is the arena shape —
cfg(kani) fixed-capacity tables and indices on the proven path — gated
on `build_panic_freedom` reaching the solver at the 8 GB standard.
Nothing is cited as proved; the parsed kernel stays byte-identical and
fully green.

## 2026-11-21: the arena arc — the first proved harnesses (beads catherine-6g2)

Two levers, in order, under the lane standard:

1. **The arena** (`engine/store.rs`): one `Bounded<T, CAP>` table type —
   array-backed under `cfg(kani)`, `Vec`-backed otherwise, identical
   call surface — behind `EngineInstance.cells`, `EngineTyped.tables`,
   and the build reports. Measured on the all-families build harness:
   symex 88.7 → 70.6 s, program steps 1 146 825 → 726 468, VCCs after
   simplification 24 066 → 11 917; the propositional reduction now
   completes and the solver runs (17 properties solved individually)
   before the per-property loop exceeds 8 GB. Caps are proof
   obligations discharged against the computed exact report bound
   (`REPORT_CAP`), never silent truncation; production keeps parsed
   sizes and carries no cap.
2. **Partitioning the build obligation per family** (Decision 4):
   `build_panic_freedom_{equation,key,coproduct,pullback}` over the
   per-family shapes. All four **proved** at the standard — the lane's
   first verified results: 18.7 s / 1.24 GB, 17.6 s / 1.20 GB, 18.7 s /
   1.29 GB, 36.5 s / 1.89 GB (`target/kani-timings.txt`, logs in
   `target/part.log`).

The walker harnesses (check/soundness families) moved off the memory
wall (~2.1-2.5 GB peak, was OOM) onto a solve-time wall on their
remaining heap machinery, and the lane's first counterexample landed
(check_coproduct_panic_freedom) — attributed to unwinding assertions
on allocator-model loops in `dangling_findings`' collect/`any` chain,
not kernel logic (dispositions above). The next lever is the walker
arena; the all-families build space stays declared with its
disposition, its claim carried by the four per-family proofs.
Ledger `0005` records the extension.

## 2026-11-23: walker arena round 2 — the dangling exclusion redesigned, the compose wall isolated (beads catherine-6g2)

Two measured steps, under the lane standard (Kani 0.68.0 / CBMC 6.11.0,
unwind 16, 300 s wall, 8 GB scope):

1. **The temporaries arena** (the recorded revisit trigger): every
   walker temporary became a fixed-cap `Bounded` table — key tuples and
   groups, coproduct images, coverage groups, projection buckets,
   fibers (`ARROWS_CAP` joins `ROWS_CAP`/`REPORT_CAP` as a proof
   obligation), `Bounded::sort_pushed_by`/`dedup_pushed` replace the
   slice sorts on pushed prefixes, `CompactViolation::Coproduct`'s
   witness carries a bounded table instead of a heap `Vec`, the
   pullback families emit straight into the walk's report vector (the
   per-pullback temp `Vec` + `extend` — the counterexample trace's
   `extend_desugared` — is gone), and `fiber`'s plain `Vec::push`
   (the measured 3.7× toxin) is gone. Parsed flavor: identical
   behavior over the same walkers, pinned by the full contract suite.
2. **The dangling exclusion redesigned**: the per-cell membership scan
   over the violations table (a symbolic-length inner loop re-enumerated
   per cell — the symex wall, measured: a dangling-only probe hung past
   150 s alone) is replaced by per-slot violation flags derived in ONE
   pass over `typing_violations`, then flat reads per cell
   (`slot None && !flagged` — the identical partition, emission order
   unchanged). Measured: the dangling family now **verifies — 47.7 s,
   SUCCESSFUL** at the standard (probe `dangling_findings` alone over
   the equation shape).

What remains, measured: with the scan gone, the rest of the walk
(equation/key/coproduct/pullback families over one harness) still
exceeds the 300 s wall in symex; the trace attributes it to
`engine_compose`'s symbolic row reads under the equation family's
forks, with Kani's pointer-validity model (`same_allocation`,
`to_isize`) charged at every re-expansion. Revisit trigger: compose's
resolution shape — the probe-matrix precedent (symbolic-index reads
over concrete-bound arrays prove in seconds) does not transfer through
the `Vec`-shaped table access + the per-row fork structure; the next
lever is a concrete-bound, non-`?`-chained resolution walk
(`CompiledEquation` paths as bounded tables) in both the kernel walker
and the soundness re-derivation. The contract suite, parity, and the
four proved build harnesses are untouched and green.

## 2026-11-23: the compose resolution walk — equation paths as bounded tables (beads catherine-6g2)

The recorded compose-side trigger (round 2's "what remains"), executed
under the lane standard:

1. **`CompiledEquation` paths as bounded tables**: the lhs/rhs paths
   moved from heap `Vec<Step>` to `Bounded<Step, ARROWS_CAP>` —
   array-backed under `cfg(kani)` (the cap a proof obligation
   discharged by the declared shapes' path lengths), `Vec`-backed and
   arbitrary-sized in production. An equation selected at a symbolic
   index (the soundness half's witness dispatch) no longer carries two
   heap buffer pointers into the walk.
2. **The concrete-bound, non-`?`-chained resolution walk** in both
   `engine_compose` and the soundness half's independent `compose`:
   slots read by array index against the path's concrete `len`; every
   read's failure arm returns `None` explicitly (undeclared step, step
   past the table space, unresolved cell) — the identical partition of
   the old `get`-chain walk without slice machinery under symbolic
   pointers. Parsed flavor byte-identical: the full workspace suite is
   green, and the four proved build harnesses are untouched.

Measured same-day pair, pre-change tree vs post-change tree, both at
the standard (Kani 0.68.0 / CBMC 6.11.0, unwind 16, 300 s wall, 8 GB
scope):

| harness | pre-change | post-change |
|---|---|---|
| check_equation_panic_freedom | 300 s timeout in symex — memcmp churn; the slice-path compose loop unrolled to the unwind cap | symex clean **48.1 s** — 946 448 steps, 46 245 VCCs → 9 692 after slicing — then budget-killed at the 8 GB scope in the reduction (rc 143, ~6.0 GB peak) |
| soundness_equation | 300 s timeout in symex — memcmp churn | symex clean **59.0 s** — 1 211 749 steps, 70 835 VCCs → 19 897 after slicing — then budget-killed at the 8 GB scope in the reduction |

The compose-side wall is cleared: symex completes and the formula is
bounded. What remains is the `0004`-class wall — propositional-reduction
memory at ~10-20 k post-slicing VCCs (solver tuning is retired there;
the budget stays closed). Revisit trigger (recorded, not funded):
per-property partitioning of the equation-family harnesses (Decision
4's build precedent) or a further formula-mass reduction on the family
walk. The other eight undetermined harnesses were not re-run this
round and keep their 2026-11-23 attributions.

Two lane repairs landed with the round: the budget classifier now
recognizes the scope-level systemd TERM (rc 143) that precedes the
cgroup SIGKILL (rc 137) — a MemoryMax breach is undetermined, never a
harness "error"; and `soundness.rs` at the round-2 HEAD did not compile
under `cfg(kani)` (a lost `as u64` cast — the 2026-11-23 recorded runs
were made on uncommitted tree state the commit did not capture), so
the committed lane is compilable again.

## 2026-11-24: the equation shape partitioned per family — the dangling family proves both halves, the findings arena (beads catherine-6g2)

The recorded trigger (ledger `0007` hole 2), executed in two measured
steps under the lane standard:

1. **Per-family partitioning** (Decision 4 at walker and dispatch
   granularity): the equation shape's obligations split per family —
   `check_dangling_panic_freedom` declared (the dangling walk is its own
   harness), `check_equation_panic_freedom` narrowed to the equation
   walk (`engine_dangling_findings`/`engine_equation_findings`, the
   lane-only partition entries), and the soundness dispatch split per
   witness kind (`soundness_equation`/`soundness_dangling` carry only
   their own arm; the sibling families' obligation code leaves their
   formulas). The lane's declared count is 16.
2. **The findings arena** (the recorded trigger's second arm — formula
   mass on the family walk): the partition entries emit into
   `Bounded<…, REPORT_CAP>` through the `FindingSink` sink (slot writes
   + `sort_pushed_by`, the stable insertion order), replacing the
   findings heap-`Vec`, the slice insertion sort, and the symbolic
   slice iteration — the last heap/slice machinery on the lane's walk
   surface, and the toxin the walk-partition alone did not remove
   (the `Vec`-partition round budget-killed all four harnesses, with
   `check_dangling` dying in the SOLVE phase at 3 156 post-slicing
   VCCs — formula size was no longer the axis).

Proved (the lane's 5th and 6th proofs — the first walker-family and
the first witness-soundness results):

- check_dangling_panic_freedom: proved — 3:30 wall, 4.96 GB peak
  (symex 83.7 s, 4 837 VCCs after slicing, solver 0.58 s).
- soundness_dangling: proved — 3:59 wall, 5.92 GB peak (symex 85.7 s,
  5 029 VCCs after slicing, solver 1.29 s).

The proved pairs run at ~5-6 GB peak inside the 8 GB scope: the
solve loop's memory genuinely grows per property toward the cap, and
the dangling family clears it with room to spare only after the arena.
The equation partitions remain solve-phase-walled — check_equation
(symex 85.7 s, 8 545 VCCs after slicing) and soundness_equation
(symex 99.8 s, 18 097 VCCs) budget-killed in the solver — and the
`--slice-formula` probe on check_equation was negative (identical
post-slicing VCCs, no verdict, killed at the cap), so the 0004
solver-tuning retirement extends to this wall class as measured. The
mechanism the round pins: harness death tracks the per-property solve
loop's growth (property count and per-property instance mass), not raw
VCC count — `check_dangling` proved with MORE VCCs (4 837) than it
died with before the arena (3 156). Revisit trigger (recorded, not
funded): a Kani/CBMC release whose per-property solve loop stops
growing memory per property — re-probe `check_equation_panic_freedom`
first — or a further property-count reduction on the equation walk's
formula. The six other undetermined harnesses (key, coproduct,
pullback families, the all-families walk and build) were not re-run
this round; the same partition-plus-arena pattern is their recorded
path.

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

Outcomes as of 2026-11-24 (Kani 0.68.0 / CBMC 6.11.0, unwind 16, 300 s
wall, 8 GB scope; post-partition, post-findings-arena engine):

Proved (the build walk's panic/overflow freedom over each family's
declared space, partitioned per design Decision 4, plus the dangling
family's walk and witness obligations — the lane's 5th and 6th proofs,
2026-11-24, see the arc above):

- build_panic_freedom_equation: proved — 18.7 s, 1.24 GB peak.
- build_panic_freedom_key: proved — 17.6 s, 1.20 GB peak.
- build_panic_freedom_coproduct: proved — 18.7 s, 1.29 GB peak.
- build_panic_freedom_pullback: proved — 36.5 s, 1.89 GB peak.
- check_dangling_panic_freedom: proved — 3:30 wall, 4.96 GB peak
  (2026-11-24, post-partition and post-arena).
- soundness_dangling: proved — 3:59 wall, 5.92 GB peak (2026-11-24,
  post-partition and post-arena).

Undetermined:

- build_panic_freedom: undetermined — split obligation: per-property
  propositional encoding memory; the arena halved the formula (symex
  70.6 s, 11 917 VCCs after simplification) and the solver now runs
  (17 properties solved individually), but the per-property loop still
  exceeds 8 GB partway through. The claim is carried by the four
  per-family proofs above.
- check_sortedness_panic_freedom: undetermined — at the 2026-11-23
  re-runs (300 s wall, post-walker-arena engine); not re-run since; its
  recorded path is the same partition-plus-arena pattern (2026-11-24
  arc above).
- check_dangling_panic_freedom: proved (see above).
- check_equation_panic_freedom: undetermined — post-partition,
  post-arena measurement (2026-11-24, see the arc above): symex clean
  85.7 s, 48 953 VCCs → 8 545 after slicing, budget-killed in the
  solve phase; the `--slice-formula` probe was negative. The wall is
  the per-property solve loop's growth, not formula size; trigger: a
  release whose solve loop stops growing, or a property-count reduction
  on the equation walk.
- check_key_panic_freedom: undetermined — at the 2026-11-23 re-runs
  (300 s wall); not re-run since; recorded path: partition plus arena.
- check_coproduct_panic_freedom: undetermined — at the 2026-11-23
  re-runs (300 s wall); not re-run since; recorded path: partition
  plus arena.
- check_pullback_panic_freedom: undetermined — at the 2026-11-23
  re-runs (300 s wall); not re-run since; recorded path: partition
  plus arena.
- soundness_dangling: proved (see above).
- soundness_equation: undetermined — post-partition, post-arena
  measurement (2026-11-24, see the arc above): symex clean 99.8 s,
  69 849 VCCs → 18 097 after slicing, budget-killed in the solve
  phase. Same split obligation as its check half.
- soundness_key: undetermined — at the 2026-11-23 re-runs (300 s
  wall); not re-run since; recorded path: partition plus arena.
- soundness_coproduct: undetermined — at the 2026-11-23 re-runs (300 s
  wall); not re-run since; recorded path: partition plus arena.
- soundness_pullback: undetermined — at the 2026-11-23 re-runs (300 s
  wall); not re-run since; recorded path: partition plus arena.

Counterexample (Decision 5 protocol: attribution recorded, obligation
never weakened):

- check_coproduct_panic_freedom: counterexample 2026-11-21, attributed
  to unwinding assertions on allocator-model loops, not kernel logic:
  the trace ended in `dangling_findings`' `violated` collect/`any`
  chain (`slice::Iter::any` and `Vec::extend_desugared` at the unwind
  cap 16, with `kani::mem::cbmc::same_allocation` and `usize::to_isize`
  from Kani's allocator model). RESOLVED 2026-11-23 by walker-arena
  round 1: the `violated` collect + vestigial sort were removed
  (membership scans the bounded violations table directly) and the
  coproduct image resolve loop went plain — the re-run shows no FAILED
  verdict; the harness returns to undetermined (solve >300 s, 2.95 GB
  peak), disposition above.

Phase-2 families (taxonomy disjointness, search honesty) were **not**
written: the stop-gate verdict (ledger `ledger/kani/`) does not fund
them, and every harness above shares the same wall — more harnesses
would multiply unverifiable spend, not assurance. The provocation pass
(task 3 RED evidence) is unblocked as of 2026-11-21: the partitioned
lane produced its first counterexample (attributed above), and the
provocation machinery (`scripts/provoke-kani-harnesses.sh`, one
anchored seed per harness) is landed so the pass runs per harness as
the budget reaches it.
  2026-11-19): all 11 harnesses, per-harness invocation, unwind 16,
  300s wall, 8 GB scope - every one times out at ~5.8-6.3 GB peak
  RSS (records in `target/kani-timings.txt`, logs and time outputs
  in `target/kani-amh/`).
