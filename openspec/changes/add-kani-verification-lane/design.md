# Design: add-kani-verification-lane

Grounded in the current implementation (2026-11-18 read):
`crates/catherine-core/src/{checks.rs (276 LOC),instance.rs,findings.rs}`,
`crates/catherine-analysis/src/search.rs`, `justfile`, `rust-toolchain.toml`,
and the landed `lean/` twin (parity oracle, Decision-4 divergence protocol,
ledger stop gate as `ledger/lean/0001-cbef9219.json`). Kani is **not
installed** on this machine; provisioning is a task, not an assumption.

## Decision 1: meta-level only — Kani never becomes a runtime engine

Kani verifies Rust *programs*; catherine's object level already has a
bounded checker over instances (`search.rs`: hand-rolled enumeration,
documented "no solver (ticket anti_goal)"). Running Kani/SMT at runtime to
analyze user models would duplicate `search`, violate the anti-goal, and
add a heavyweight dependency to the wasm-facing kernel. So: harnesses only,
`cfg(kani)` code, zero runtime surface. Alternative rejected: Kani as a
replacement for `search` — fails the honesty contract (`UNKNOWN { scope }`
is first-class there; a solver's incompleteness would have to be re-shaped
into it for no measured gain).

## Decision 2: harness inputs are typed domain values, not JSON

Panic-freedom harnesses synthesize nondeterministic `Schema`/`RawInstance`
values from bounded `kani::any()` scalars composed by deterministic
insertion loops (objects/arrows/rows/cells capped per harness) —
`kani::any()` never generates the map-bearing domain types themselves
(`rows`/`tables` are `BTreeMap`s in `instance.rs`), and serde strings are
never inputs. Feeding raw JSON strings through serde would explode
paths on string parsing and drag `serde_json` internals into the proof —
and `parse` is already Rust-side, pinned by golden-file fixtures and the
model-document spec; the Lean twin draws the same line (Decision 2 there:
"parse stays wholly Rust-side"). Consequence recorded honestly: this lane
proves the *execution* kernel over well-typed-but-adversarial values; the
post-parse invariant "parser output is a well-formed bounded document"
remains a property-test obligation, not a Kani proof.

## Decision 3: lane separation mirrors the Lean gate exactly

`just verify` = provisioning-checked `cargo kani` run over all harnesses,
failing on a counterexample, a harness error, or an
undetermined-without-disposition (Decision 5); `just ci` stays Kani-free
(spec scenario pins this). Precedent and
rationale are the Lean change's: the gate must stay green on every
checkout; expensive third-toolchain assurance rides a separate lane
(nightly/optional CI step). Kani bundles its own pinned nightly, so
`rust-toolchain.toml` (`1.98.1`) is untouched; the Kani release version is
committed next to it and checked by the verify recipe before running.

## Decision 4: harness families in two phases, provocation first (red→green→refactor)

- **Phase 1**: families 1–2 of the proposal (panic-freedom over the check
  path; witness soundness on emitted findings). RED discipline: before
  trusting a green proof, each harness is **provoked** — a seeded defect
  (a mutated comparator, an off-by-one in a scratch copy) must produce a
  concrete counterexample, proving the harness can fail. A harness that
  cannot be provoked proves nothing and is rewritten.
- **Phase 2 (runs only on the stop gate's extend verdict, Decision 6)**:
  taxonomy disjointness + search honesty invariants
  (`Contradiction` ⟹ completed exhaustion and no clean candidate in
  scope; `Witness` ⟹ fully resolved and clean; minimality on small
  scopes) — `catherine-analysis`, bounded small enough that the whole
  candidate space fits the unwinding bound.
- **Out of harness scope** (recorded, not hidden): serde parsing, the wasm
  crate (Kani does not target wasm32), migration Δ reindexing completeness
  (a follow-up candidate the stop gate may fund — its round-trip
  counterexample property is harnessable but proof-heavy).

## Decision 5: counterexample protocol = Lean Decision 4, adapted

A confirmed counterexample resolves as exactly one of: harness defect
(misstated obligation — fix the harness) or Rust kernel defect (beads
ticket + regression fixture, fixed outside this lane). Attribution is
recorded; obligations are never weakened to reach green. UNDETERMINED is a
legal per-harness outcome but must be loud in the lane report (spec
requirement), because a silent undetermined is the Kani-shaped version of
the naked "no violation found" the tool exists to eliminate. Concretely:
an undetermined harness fails the lane (nonzero exit) unless a disposition
— raised bound or split obligation — is recorded in the lane README.

## Decision 6: stop gate reuses the existing ledger decision kind

`catherine-ledger` gained a terminal `decision` kind for the Lean stop
gate; no schema extension. The gate fires at the end of phase 1 (after
the panic-freedom and witness-soundness families) and controls phase-2
spend: extend funds the taxonomy/search-honesty families (and possibly
the Δ migration follow-up), hold maintains at phase-1 scope, retire
archives the harness modules and the verify recipe with lessons kept in
the lane README and the ledger entry. The decision entry records what
Kani bought (defects found — hopefully zero, like the Lean parity run;
bounds reached; time cost) and picks extend / hold / retire. Given the Lean precedent
(parity + theorems surfaced no defect), the honest prior is that Kani's
yield will be in crash-freedom bounds and search invariants, not kernel
bugs; the gate makes that call on evidence either way.

## Risks

| Risk | Mitigation |
|------|-----------|
| Proof times balloon (minutes per harness) | Explicit per-harness bounds; nightly lane; stop gate bounds total spend |
| Kani/nightly version churn breaks the lane | Committed version pin + provisioning check in `just verify`; upgrades are deliberate pin bumps |
| Kernel dependencies (`serde_json` on envelopes, `hegeltest` dev-deps) leak into harness builds and block compilation | Harnesses exercise pure kernel functions over typed values; if a dependency still blocks a harness, scope the harness to the exercised functions and record the limitation in the lane README — never stub the kernel under test |
| False confidence from unprovokable harnesses | Provocation step is part of phase-1 RED (Decision 4); a harness must demonstrate it can fail |
| Overlap confusion with Lean/hegel roles | README in the lane states the division: Lean = semantic reference theorems, hegel = sampled properties, Kani = exhaustive-within-bounds over real Rust |
