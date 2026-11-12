# Brief: catherine-9b5 — P5 wasm boundary + minimal web UI

Openspec change: `add-wasm-web-view` (read proposal.md + tasks.md first).
Ticket: P5, complexity l. RED→GREEN→REFACTOR = tasks.md tasks 1–9.

## Target paths (exclusive)

- `crates/catherine-wasm/**` — new crate (wasm-bindgen boundary over core+analysis)
- `web/**` — new static site (plain TS + HTML + SVG, no framework, no server)
- `justfile` — add wasm compile-gate target; wire into `ci`
- `.github/workflows/ci.yml` — wasm32 compile-only gate step
- `openspec/changes/add-wasm-web-view/tasks.md` — tick boxes as you complete tasks

## APIs to consume (read before writing)

- `crates/catherine-core/src/` — pub `Model`, parse entry, findings encoding
  (the kernel encoding the CLI forwards verbatim in `data.findings`)
- `crates/catherine-analysis/src/` — analysis entry producing findings
- `crates/catherine-cli/src/{cli,run,render}.rs` — the single execution path +
  genesis-vibes envelope you must match byte-for-byte in the parity test
- `crates/catherine-core/schema/ct-schema-v1.json` — source for generated TS types (task 5: generator script checked into repo; TS output must be derived, never hand-edited)
- `fixtures/parity/` + `crates/catherine-cli/tests/fixtures/` — fixture corpus for the parity test

## Contract invariants

- JSON in/out only at the boundary; no logic in the wasm crate
- Row-count cap enforced AT THE BOUNDARY with a typed error; kernel NOT invoked on oversize (RED task 1)
- Findings panel renders kind + epistemic label + witness — same taxonomy as CLI (RED task 2 golden DOM test)
- Kernel crates stay genesis-free and wasm-optional; wasm crate depends only on core+analysis+serde+wasm-bindgen
- No framework (no Leptos/Yew/react), no server, UI never the source of truth
- Editing surface in v1 = JSON textarea only; file load/save + SVG schema graph + findings panel
- proptest/structural assertions, never snapshots
- Language: web TS compiles via npm devDeps (typescript/esbuild ok, no registry surprises — npm works on this machine)

## Discipline (system contract)

- Edit only. You MAY run `cargo check`/`cargo check --target wasm32-unknown-unknown` for compile feedback.
- NEVER run: cargo fmt, clippy, cargo test, just ci, ah/pretender/bd/openspec gates, any git commit, any `bd` mutation (close/claim/export are orchestrator-reserved).
- Task 9 (`bd close`, export) is orchestrator-reserved: tick the tasks.md box for your own parts, note "bd close pending orchestrator" in your report.
- If any acceptance command is impossible as specified, STOP and report — do not substitute a weaker criterion.

## Acceptance (report each with evidence)

1. Task 1: boundary test exists — oversized document → typed cap error, kernel not invoked (test compiles; orchestrator runs it).
2. Task 2: golden DOM test — findings panel renders kind+label+witness per finding-class fixture (web test harness: vitest+jsdom acceptable).
3. Task 3: wasm exports parse/check/analyze JSON in/out.
4. Task 4: wasm32 compile gate in justfile `ci` + ci.yml step (compile-only).
5. Task 5: TS types generated from ct-schema-v1.json by a checked-in generator script.
6. Task 6: web/ file load/save, SVG schema graph, findings panel, JSON textarea.
7. Task 7: parity test — CLI path vs wasm exports produce identical finding encodings on the fixture corpus (Rust integration test, native compile).
8. Task 8: rendering helpers deduped; web/ framework-free.
9. tasks.md boxes ticked for tasks 1–8; report lists files touched + exact commands the orchestrator must run.
