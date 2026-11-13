# ro5u review — add-wasm-web-view (catherine-9b5)

Full-diff rule-of-5-universal review, lead-side. Diff: `crates/catherine-wasm/**`
(274 src+test LOC), `web/**` (~570 TS LOC), `Cargo.toml`/lock (workspace
member + deps), `justfile` (`ci` gains wasm32 gate + `web-build` recipe),
`.github/workflows/ci.yml` (wasm32 compile-only step), tasks.md ticks.
Orchestrator verification: `just ci` green (fmt/clippy -D warnings/tests +
wasm32 check); `cargo test -p catherine-wasm` 4/4; parity 19 fixtures
identical; web vitest 3/3; `tsc --noEmit` clean; headless-browser smoke
load→check→render PASSED with screenshot (bowling fixture: violation row
rendered, SVG graph 4 nodes/5 edges); `openspec validate` 10/10 strict.

Five passes:

1. Correctness: cap enforced in `decode` before `parse` (kernel not
   invoked on oversize — proven structurally: the oversized fixture is
   also malformed JSON, so a `Parse` answer would prove the parser ran);
   findings encoding is the kernel serde output verbatim (parity-pinned);
   `analyze` composes the CLI's analysis ops in taxonomy order.
2. Contract drift (class B): TS `FindingEnc`/`BoundaryErrorEnc` mirror
   the Rust serde encodings and are pinned by the parity test + boundary
   test's code-tag assertion — the mirror cannot drift silently.
3. Tests: behavior changes carry tests (cap → boundary test; outcome
   discriminator → zero-size-rule + witness assertions; parity → corpus).
   No snapshots (structural DOM assertions in vitest; golden-DOM test
   builds expected tree in-test).
4. Negative matrix (catherine-348): no CLI flag/surface added — the
   boundary's malformed-input classes (oversize, parse position passthrough,
   config decode) are the covered negative classes for this surface.
5. Regression risk: workspace-only additions; kernel crates untouched
   (genesis-free invariant held — wasm crate deps: core, analysis, serde,
   serde_json, wasm-bindgen only).

Findings + dispositions:

- [FIXED during verification — see note] MEDIUM: `analyze` collapsed
  `SearchOutcome::Witness` into silence (a found witness indistinguishable
  from a vacuous run). Fixed: `AnalyzeReport.outcome` discriminator
  (`witness|contradiction|unknown`) + v1 no-witness-payload limit stated in
  the struct docs + TS mirror + two test assertions (zero-size → unknown
  empty scope; one-row → witness). Behavior-impacting → test added.
- [FIXED during verification] MEDIUM: browser glue never ran — bundled
  init fetched the .wasm at the wrong URL, and `index.html` had `#errors`
  where `app.ts` requires `#error` (startup abort, panels dead). The
  brief assigned the headless smoke to the orchestrator, and it caught
  both. Fixed: explicit `init("./pkg/catherine_wasm_bg.wasm")` in
  `web/main.mjs` (new browser-glue module, deliberately the only `pkg/`
  importer), id aligned, smoke green.
- [DEFERRED, LOW, reason] wiring-test gap: vitest covers `renderFindings`
  but not the index.html↔app.ts id contract; the executable evidence is
  the smoke (now pass recorded). A DOM id test would re-assert static
  HTML — tautology risk; accepted.
- Process note (orchestrator report mirrors this): the glue/id defects
  and the analyze-outcome fix were applied inline by the orchestrator
  because subagent dispatch was unavailable (OpenRouter 402 credit
  exhaustion) — rule-7 deviation, disclosed, not silent.

max_critical = 0. Verdict: PASS.
