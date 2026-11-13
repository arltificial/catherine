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

The kata ledger's rollup is sealed (project verdict `survived
(scoped)`) — but the project already proved the lighter sibling pattern
in-tree: `ledger/rule-lens/` is a separate ledger directory of
unmodified kata-shaped entries with its own self-negating floor control
and a recorded executed baseline arm, green under the same validator.
The dossier's open question — "can the catalog format be reused for
non-kata domain probes, or does it need a lighter sibling?" — is
answered here: **reuse the kata shape in a separate ledger directory
(the rule-lens pattern)**; the one genuinely new rule is that a
unique-detection claim requires a committed, executed baseline arm.

## What Changes

- **Probe ledger directories** (`ledger/probes/<name>/`) reuse the kata
  entry shape, content-addressing, closed verdict vocabulary, and
  append-only supersession — no new entry kind (design Decision 1
  revised after Rule-of-5 review found the rule-lens precedent
  unaddressed). Each probe series carries its own floor control: a
  fault set where the conventional baseline catches everything, on
  which the probe must show zero unique detections.
- **One new validator rule, ledger-wide**: a non-empty
  `unique_detections` without a baseline arm naming a committed,
  version-pinned baseline artifact (frozen script or pinned tool
  revision) is refused; a cited-but-not-executed baseline caps the
  verdict at `inconclusive` on uniqueness; no baseline arm caps at
  parity (empty unique set). This retroactively hardens the kata
  science — all existing entries already comply.
- **Validator rules preserved by regression pins**: probe sequences
  without a self-negating floor control are refused by the EXISTING
  violations; the sealed kata rollup is never recomputed, extended, or
  superseded by probe recording (byte-identity test).
- **Inaugural probe 1 — migration Δ at real scale**: bogart-scale
  migration fixtures built up from `bogart_rehearsal.ct.json`,
  pre-registered lost-data/non-invertibility expectations, baseline arm =
  a committed version-pinned harness encoding the conventional practice
  the synthesis named (statement-level migration linting à la
  Atlas/Flyway + row-count/checksum dry run), run through `catherine
  migrate`, recorded as probe ledger entries. Answers: does
  counterexample quality hold at real schema complexity, and is the
  unique band real against the baseline?
- **Inaugural probe 2 — rule lens on extracted tables**: extract a real
  policy/eligibility table from the bogart/Normai corpus (prose/PDF —
  the ingest surface no DMN analyzer has; if no real table exists, the
  probe records `inconclusive` naming the missing corpus, never a
  fabricated table), run `catherine rules` / discovery against declared
  examples, baseline arm = the manual-review protocol **frozen into a
  committed scripted assertion set before the run** (the
  `rule_lens_baseline.py` 45-assertion pattern — a live "we reviewed
  it" note is not a baseline arm), recorded as probe ledger entries.
  Answers: does the P8 unique-detection result survive restriction to
  extracted tables?
- **Docs scoping after evidence lands**: README/USING rule-lens claims
  repositioned to the extracted-table scope (DMN parity acknowledged with
  citation); migration claims cite the probe ledger entries; USING.md
  banner flips ILLUSTRATED → EXECUTED with captured runs (coordinated
  with the docs-drift ticket).

## Impact

- Affected specs: **`domain-probes` (new capability)**; consumes
  `migration`, `rule-lens`, `cli`, `findings` unchanged;
  `kata-validation` spec untouched (sealed rollup preserved by
  byte-identity test), though the baseline-arm rule applies to kata
  entries too.
- Affected code: `crates/catherine-ledger` (baseline-arm fields +
  validator rule + probe catalogs; NO new entry kind),
  `fixtures/probes/` (new), baseline harness scripts (committed,
  pinned), docs (post-evidence).
- Spectacular: new scenarios need contract TOMLs under
  `.espectacular/domain-probes/` at implementation time.
- Beads: one implementation ticket; coordination (not dependency) with
  the docs-drift ticket.
