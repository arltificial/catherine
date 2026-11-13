# Change: Add domain-probe validation — pre-registered non-kata probes with baseline arms
> Evidence: pinky dossier `ct-modelable-domains` dispatch synthesis (2026-11-12,
> 12-provider blind survey); catherine ledger entries 0001–0005 + rule-lens
> ledger 0001–0003; E4 (docs/DESIGN.md §0).

## Why

The dispatch synthesis ranked migration Δ the strongest unique-value axis
(no incumbent emits named-row lost-data witnesses or non-invertibility
counterexamples) **and** unanimously flagged that the claim is
capability-absence + defect-family existence, not a measured comparison:
catherine has migration fixtures but no ledger entry with a
conventional-baseline arm — the exact science format the project requires
of every unique-detection claim (the kata ledger pattern). Simultaneously,
the synthesis showed rule-lens gap/overlap/shadow analysis is **commodity
for authored DMN tables** (Signavio, OpenRules, Calvanese et al.
arXiv:1603.07466); the P8 unique-detection result generalizes only to
*extracted* tables that no DMN analyzer can ingest — a claim that is
currently unscoped in README/USING and untested on real extracted tables.

The kata ledger cannot carry these runs: its rollup is sealed (project
verdict `survived (scoped)`), its control-first ordering is
FizzBuzz-specific, and its catalog is kata-scoped. The dossier's open
question — "can the catalog format be reused for non-kata domain probes,
or does it need a lighter sibling?" — is answered here: a lighter sibling
entry kind (`probe`) in the same crate, same content-addressing, same
closed verdict vocabulary, never touching the sealed rollup.

## What Changes

- **`catherine-ledger` gains a `probe` entry kind** (lighter sibling of
  `kata`): pre-registered expectation catalog (fault → class → expected
  catherine outcome, committed before the run, referenced by hash),
  executed run record, **named conventional-baseline arm** (a
  unique-detection claim without a recorded baseline run is refused —
  parity is the ceiling without one), expressibility gaps, closed verdict
  vocabulary (`killed | survived | inconclusive | invalid-run`),
  content-addressed ids, append-only supersession.
- **Validator rules**: a probe entry without a catalog hash is refused; a
  `unique_detections` claim without a baseline arm is refused; probe
  entries are refused inside the kata ledger series (separate
  `ledger/probes/` sequence); the sealed kata rollup is never recomputed
  or extended by probe entries.
- **Inaugural probe 1 — migration Δ at real scale**: bogart-scale
  migration fixtures (beyond the 3–4-row pins in `fixtures/migration/`),
  pre-registered lost-data/non-invertibility expectations, baseline arm =
  the conventional practice the synthesis named (statement-level
  migration linting à la Atlas/Flyway + row-count/checksum dry run), run
  through `catherine migrate`, recorded as probe ledger entries. Answers:
  does counterexample quality hold at real schema complexity, and is the
  unique band real against the baseline?
- **Inaugural probe 2 — rule lens on extracted tables**: extract a real
  policy/eligibility table from the bogart/Normai corpus (prose/PDF —
  the ingest surface no DMN analyzer has), run `catherine rules` /
  discovery against declared examples, baseline arm = manual review of
  the same table, recorded as probe ledger entries. Answers: does the
  P8 unique-detection result survive restriction to extracted tables?
- **Docs scoping after evidence lands**: README/USING rule-lens claims
  repositioned to the extracted-table scope (DMN parity acknowledged with
  citation); migration claims cite the probe ledger entries; USING.md
  banner flips ILLUSTRATED → EXECUTED with captured runs (coordinated
  with the docs-drift ticket).

## Impact

- Affected specs: **`domain-probes` (new capability)**; consumes
  `migration`, `rule-lens`, `cli`, `findings` unchanged; `kata-validation`
  untouched (sealed rollup invariant explicitly preserved).
- Affected code: `crates/catherine-ledger` (entry kind, validator,
  catalog, bins), `fixtures/probes/` (new), docs (post-evidence).
- Spectacular: new scenarios need contract TOMLs under
  `.espectacular/domain-probes/` at implementation time.
- Beads: one implementation ticket; coordination (not dependency) with
  the docs-drift ticket.
