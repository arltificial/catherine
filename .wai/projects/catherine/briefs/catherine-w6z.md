# Brief: catherine-w6z — vampiro seam-composition gate (pre-push) + B2 false-negative feedback

Ticket: P2. vampiro 0.5.0 targets failure class A but missed all review bugs — law-axis false negative to report upstream.

## Target paths

- `lefthook.yml` — pre-push block ONLY, outside `# ddl:managed:start/end` markers
- `versions.ddl.toml` — `[tools]` gains `vampiro = "0.5.0"` with a one-line comment (same pin discipline as the others; conformance test requires the file to keep existing — additive change only)
- `.wai/projects/catherine/research/catherine-w6z-vampiro-feedback-body.md` — the filed feedback body (verbatim), so the record survives even if upstream filing is blocked

## Contract

1. Pre-push gate: new lefthook pre-push command `vampiro-seam-gate`: `vampiro check -p crates --mode gate --severity-threshold high` (read `vampiro check --help` output reproduced here: `-p/--path`, `--full`, `--mode guidance|tiered|gate`, `--severity-threshold low|medium|high`, `--base/--target` diff scope). Gate mode exits non-zero on findings at/above threshold — that is intended (complementary gate; does NOT replace catherine-348/za6/3zs). Place it adjacent to `ah-check-run-tests`.
2. Pin: `vampiro = "0.5.0"` in versions.ddl.toml (installed version verified 0.5.0).
3. Feedback body (the deliverable that survives): document the B2 false-negative as a concrete law-axis use case:
   - Class: cross-origin join of two same-typed values in one call — `TypedInstance::build(&model.schema, &second.instance)` compiles and "composes" (shape-valid: `(&Schema, &Instance)`) while violating the provenance law "schema and instance come from the same document" (the catherine-18t cross-schema conflation bug, fixed in catherine commit 8f3fbcc).
   - Evidence: `vampiro check --full -p crates` on catherine @ 268c142 reported 0 findings, including `crates/catherine-cli/src/runners.rs` where the bug lived.
   - Ask: law/prove integration in the CLI (`prove` is currently a placeholder) so a declared provenance law ("arg0 schema and arg1 instance share one document origin") catches same-typed cross-origin joins; re-evaluate as pre-commit gate if it lands.
   - File via `vampiro feedback --kind bug` — run with `--dry-run` FIRST, save the printed body to the research file, then attempt the real filing. If real filing fails (auth/network), record the exact failure in the report and leave the body file as the deliverable — do NOT loop retries.

## Discipline

Edit only. Running `vampiro feedback --dry-run` and `vampiro feedback` (once) is allowed — they are the ticket's deliverable, not a gate. NEVER: fmt/clippy/cargo test/gates/`just ci`, git commits, bd mutations (either repo). Do NOT run `vampiro check` yourself — the orchestrator verifies the gate live.

## Acceptance (report with evidence)

1. lefthook pre-push block contains the vampiro gate outside managed markers.
2. versions.ddl.toml pin added.
3. Feedback body saved; dry-run output verbatim; real filing attempted once with outcome recorded.
4. Exact commands for orchestrator verification.
