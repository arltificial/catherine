# Verus verification lane

Functional correctness proofs over the shipped kernel's equation walk,
checked by [Verus](https://verus-lang.github.io/verus/) — the
SMT-backed functional-proof sibling of the [Kani lane](kani-lane.md)
(bounded model checking) and the Lean twin (semantic reference + parity
oracle). Run: `just verify-verus`. The lane is deliberately **outside
`just ci`** — same precedent as `verify` and the Lean gate: a pinned
external toolchain every checkout cannot assume must not gate every
commit. Verus is **not a runtime engine** — the anti-goal "no solver"
stands; proof annotations only, zero runtime surface.

## Pin and provisioning

`verus-version.toml` pins the release (`0.2026.11.22.1ce6a59`), the zip
URL, its sha256, and the required rustup toolchain (`1.100.0`).
`scripts/check-verus-lane.sh` provisions idempotently into
`~/.local/share/verus/<release>/` (override the root with
`VERUS_HOME`): download, checksum-verify, unpack, install the pinned
toolchain with a minimal profile. The script refuses to run under any
other installed release; CI caches the install directory keyed on the
pin file. Bump deliberately — a checker drift on a rolling release must
not silently change recorded proof outcomes.

First fresh-runner CI exercise (workflow_dispatch, 2026-11-25, commit
`52745e9`): provisioning caught a real ordering defect — the shim's
`--version` pin check ran before the toolchain install and cannot
execute without it — fixed by installing the side toolchain first; the
job then ran green (`verus lane: green — 1 model units verified, parity
over 6 fixtures (5 emissions), 34 guide files as expected`).

## Layout

The lane sits outside the workspace's fail-closed complexity gate by
design: `pretender.toml` excludes `**/catherine-verus/**` and
`**/probe-seed/**` (same treatment as `**/lean/**` — external-checker
source outside the Cargo build; vendored guide examples and single-file
proof units the 400-line cap cannot hold), and
`crates/catherine-verus/parity/target` is gitignored. The gate still
covers every shipped workspace crate untouched.

- `crates/catherine-verus/model/lib.rs` — the verified model: the
  equation-family walk (`engine_compose`, `equation_findings`,
  `push_equation_finding`) over the kernel's engine types, with the
  full functional spec (walk output = exactly the applicable
  disagreeing emissions, in sorted key order) proved by Verus. Promoted
  unchanged from the probe; the probe source is archived verbatim at
  `crates/catherine-verus/probe-seed/lib.rs`.
- `crates/catherine-verus/parity/` — the parity driver: a detached
  cargo project (own empty `[workspace]` table; never a workspace
  member, never built by `cargo build`/`just ci`). Its `src/main.rs`
  carries the same exec logic as trusted plain Rust, projecting
  `.ct.json` documents into the model's typed values exactly the way
  the kernel compiles them (name-order nids via the kernel's
  `BTreeMap`s, canonical equation order `(lhs, rhs, rationale,
  status)`, dense row ids per object, dead steps for undeclared arrow
  names, equation domain = the lhs lead arrow's source object).
  Documents outside the verified fragment (per-object rows above
  `ROWS_CAP = 8`) exit 3 — a named boundary, never a silent drop.
- `crates/catherine-verus/fixtures/parity-model/` — six corpus
  documents exercising the family's behavior surface: divergence
  (violation emitted), full agreement, partial tables (dangling
  findings, no equation emission), zero-row domains, multi-equation
  canonical ordering, and the `proposed → hypothesized` label mapping.
- `crates/catherine-verus/fixtures/guide/` — the 34 Verus-guide
  examples, vendored verbatim from
  `verus-lang/verus@release/0.2026.11.22.1ce6a59`
  (`examples/guide/`), with `expectations.txt` recording each file's
  outcome under the pinned release (30 verify; 5 are library-style
  snippets the driver rejects with `E0601`; `higher_order_fns.rs`
  intentionally fails verification — the guide's own callee-requires
  teaching example).

## Proof families

The stop-gate verdicts: **extend** funds further proof families (the
landed verdict — compose contracts), **hold** maintains the lane at its
current families, **retire** archives the model, corpus, recipe, and
workflow job with conventions and lessons kept here and in the ledger
entry. The verdict is sealed either way in `ledger/verus/`.

**Compose family** (promoted from the probe seed under the stop-gate
extend verdict, ledger `ledger/verus/0001`): `engine_compose` — the
walk's path-composition primitive, copied verbatim from the kernel's
Vec flavor (`crates/catherine-engine/src/engine/compose.rs`,
`store.rs:230`) — is proven panic-free under the dense-id invariant,
and its ghost functional spec `gcompose` is proven equal to the exec
walk (the unfold lemma with arm-splitting). These are the compose-side
contracts the equation walk's correctness rests on.

**Equation-walk family** (phase 1): `equation_findings` with its spec
mirrors (`collected_upto`, `emission`, `walk_pos`, `domain_count`) and
the two lemmas that bridge spec and presentation — Lemma A (the walk's
output is exactly the applicable disagreeing emissions: validity,
completeness, sortedness in key order) and Lemma B (the family key
order equals the rendered `Finding` order, `key_lt` == `find_lt` —
the order-bridge claim the kernel's contract tests pin). 29
obligations, 0 errors, ~1.5 s wall under the pinned release.

## What the gate runs (`scripts/check-verus-lane.sh`)

1. **Model proof units** — every `model/*.rs` under the pinned
   release with `--crate-type lib`, parsed for
   `verification results::`; three-valued outcomes (VERIFIED / ERROR /
   EXHAUSTED via rlimit), each unit wall-capped at 300 s. A unit with
   no results line is an error, never silently green. Today: 1 unit,
   29 verified, 0 errors.
2. **Parity gate** — the driver and the kernel (`catherine check
   --json`) each walk every corpus document; normalized equation
   findings must match exactly. The kernel's nonzero exit on
   findings-present runs is its exit contract (2 + finding class);
   well-formedness is asserted via the envelope's `ok` field. The
   **non-vacuity witness gate** requires ≥ 1 corpus emission in total:
   two always-empty streams match trivially, so a corpus that emits
   nothing fails the gate.
3. **Guide tripwire** — every vendored guide file must verify (or
   fail) exactly as `expectations.txt` records, under the same pinned
   release. Outcome drift is a toolchain or upstream regression.

Per-unit wall times land in `target/verus-timings.txt`. Every declared
unit/corpus file/expectation line must produce an outcome — no silent
drops in either direction.

## Provocation evidence (recorded under the pinned release)

The gate is shown to fail on its fault classes, not just pass:

- **Guide drift**: flipping one expectation to `fail:E0601` fails with
  `guide outcome drift on assert_by_compute.rs`.
- **Driver drift**: neutralizing the driver's canonical equation sort
  reproduces the index-attribution fault class (kernel `EQ 0 …` vs
  model `EQ 2 …` on `05_multi_order.ct.json`) and fails the parity
  diff.
- **Vacuous corpus**: stripping every corpus divergence (0 emissions)
  fails the non-vacuity witness gate.

Note the fault class the parity gate intentionally does NOT catch:
mutating a corpus document moves kernel and driver together (both read
the same document), so parity stays green — that mutation class belongs
to the kernel's own contract tests, not to model/kernel drift.

## Division of labor

- **Lean twin** (`lean/`, `just lean`): the semantic reference and the
  fixture-corpus parity oracle over the same check-applicable corpus.
- **Kani lane** (`just verify`): bounded model checking — panic- and
  overflow-freedom, witness soundness on the shipped artifact.
- **Verus** (this lane): full functional correctness of the equation
  walk — the model's spec pins the walk's output equality, sortedness,
  and completeness at spec level, and the parity gate pins the same
  walk behaviorally against the shipped kernel on real documents.

## What the lane does not claim

- Only the **equation family** is characterized (design Decision 4's
  scope); keys, coproducts, and pullbacks remain Lean/Kani territory.
- The parity driver is trusted plain Rust outside any proof; its
  drift is exactly what the parity diff catches. The verified side of
  the contract lives in `model/lib.rs`.
- The guide corpus pins the *toolchain's* behavior on upstream
  examples; it makes no claim about the kernel.
- `higher_order_fns.rs`'s recorded failure is the guide's intentional
  teaching example, not a lane defect.
