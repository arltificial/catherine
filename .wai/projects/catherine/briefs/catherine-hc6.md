# Brief: catherine-hc6 — bogart pilot corpus, models 1–3 (first real-domain dogfood)

Ticket: P9. Works in `../bogart` (separate repo: `at/bogart`, Clojure, own beads DB, own AGENTS.md). Read bogart's `AGENTS.md` FIRST (POC policy, spk lint rules, no-restating-objective rule) and `docs/objective.org` pointer discipline. catherine repo paths are READ-ONLY for you except nothing — all writes land in bogart.

## Scope (closes THIS ticket)

- openspec/changes/add-bogart-pilot tasks: 1 (RED fixture), 2, 3, 4 (models 1–3), 7 (CI wiring), 8 (live run + triage), 9 (pilot report). Tasks 5/6 (models 4/5) are P7/P8-gated adjuncts — DO NOT implement, DO NOT stub.
- Model 2 stays spec-only if it needs rule-lens machinery (P8, not landed): land it as far as P3+P4 capability allows and record the expressibility gap in the pilot report — never paper over.
- Findings vocabulary: P2 taxonomy names; overlap/gap map to the kinds `add-rule-lens` design.md Decision 1 specifies (violation / underconstraint).

## Target paths (in ../bogart unless noted)

- `../bogart/test/` or `../bogart/test-resources/` — models + RED fixture corpus (follow bogart's existing test layout)
- `../bogart/.github/workflows/ci.yml` — NEW: bogart has no workflows dir; create it
- `../bogart/docs/` — pilot report per bogart docs conventions (check existing docs/ layout first)
- catherine repo: `.wai/projects/catherine/research/catherine-hc6-pilot-findings.md` — triage notes mirrored home (findings fired, real vs noise + cause per finding)

## RED first (task 1)

Seeded bogart-like corpus with a DRIFTED anchor: build the minimal fixture (a `.ct.json` model of the FTS-anchor invariant + a small instance with one drifted `block → edition` anchor) and demonstrate catherine check FIRES violation + dangling with witness rows BEFORE any live-corpus run. Command shape: `cargo run -p catherine-cli -- check <model.ct.json>` from the catherine repo (catherine is built at whatever rev main pins at dispatch; exit codes per P3 contract: violation 3, dangling 4). Save the RED evidence output in the research file.

## Models 1–3 (per DESIGN §5 pilot table + proposal)

1. FTS-anchor equation: `block → document → edition = block → edition` + totality of `block → document` → violation + dangling on drifted denormalized anchors. Ground the arrow/object names in bogart's ACTUAL data shape (read bogart corpus/schemas — `data/`, `src/`, openspec docs on corpus storage). Names must match bogart reality, not invented shapes.
2. Identity-domain disjointness (aviso vs section-acquisition id formats): table-form model; overlap/shadow only insofar as P3+P4 express it; record the gap.
3. Capture selector relation: `capture_ambiguous` (multiple matches, no tie-break) / `capture_unavailable` (no match) → overlap/gap as expressible; record the gap.

## CI wiring (task 7)

`../bogart/.github/workflows/ci.yml`: checkout bogart + checkout catherine at the PINNED rev given at dispatch time (orchestrator supplies the SHA — do not guess); cargo build of `catherine-cli` with a cached cargo target dir (actions/cache keyed on Cargo.lock contents you record); run `catherine check` per model with class exit codes; fail the build on violation/dangling exits (3/4); keep the workflow minimal — no docker, no services. You cannot test the workflow live (push is orchestrator/user-gated) — validate syntax with `actionlint` if present, else state so.

## Live run + triage (task 8) + report (task 9)

- Run models 1–3 against bogart's LIVE corpus data (bogart is a POC; the frozen/local corpus path is acceptable per its POC policy — live fetch is NOT authorized, do not fetch).
- Triage EVERY finding: real defect (with bogart source pointer) vs noise (with cause: model imprecision / finding granularity) — never silently suppressed. Findings that are real bogart defects: file them in BOGART's beads (`bd create` in ../bogart, bogart- prefix happens automatically) — do not fix them.
- Pilot report: per model — findings fired, real/noise accounting, defect links, expressibility gaps. Store in bogart docs + mirrored summary in the catherine research file.

## Discipline

Edit only in bogart (+ the two artifact files in catherine's .wai). bogart commits: you MAY commit in bogart per its conventions (lowercase prefix style like `pilot: models 1-3 + ci wiring`) — but NEVER push, NEVER run bogart's heavier gates (its justfile checks are orchestrator-side; run only what you need to validate the models: the catherine check runs). NEVER touch catherine repo code or catherine bd. catherine-hc6 close is orchestrator-reserved.

## Acceptance (report with evidence)

1. RED fixture evidence: drifted-anchor corpus → violation + dangling fire, output saved.
2. Models 1–3 as .ct.json grounded in bogart's real schema names.
3. CI workflow file with pinned catherine rev + cache + class-exit-code gate.
4. Live-corpus run outputs + full real/noise triage.
5. Pilot report in bogart docs + mirrored research file.
6. Bogart commits made (list SHAs); nothing pushed; bogart defects filed in bogart bd (list IDs).
