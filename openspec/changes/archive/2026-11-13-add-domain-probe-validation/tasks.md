# Tasks: add-domain-probe-validation

Revision 2 (post Rule-of-5): tasks re-anchored on the rule-lens reuse
pattern (design Decision 1 revised) — no new entry kind; the new
validator surface is the baseline-arm rule. One red→green→refactor arc
per task group (beads `catherine-1od`). Validation runs use the
committed harness path (`catherine_ledger::harness::run`) and the
existing `ledger-check <ledger-dir>` surface, never a bespoke re-run.

## RED — the ledger refuses baseline-less uniqueness

- [x] 1. Validator rejects, with named violations: any entry (probe or
      kata) with non-empty `unique_detections` and no baseline arm
      naming a committed baseline artifact; a `survived` verdict on an
      entry whose baseline arm is labeled cited-only (inconclusive is
      the ceiling) — pinned by crate tests before any probe support
      exists. Regression-pin (no new code, existing violations must
      fire): a probe-directory sequence without a self-negating floor
      control → `NegativeControlMissing`/`ControlDidNotSelfNegate`; an
      unregistered probe id → empty expectations, no derivable verdict.
      Spectacular contract stubs generated alongside (`ah scenario new
      add-domain-probe-validation domain-probes …` for all 9 scenarios,
      test entries pointing at these RED tests).

## GREEN — probe ledger directories + baseline-arm rule

- [x] 2. Baseline-arm fields on the comparison shape: baseline artifact
      identity (in-tree path + hash, or tool name + pinned revision),
      execution mode (`executed | cited`), detections on the same fault
      set; validator rules from task 1 wired to the real fields;
      `ledger-record` refuses the same shapes the validator does
- [x] 3. Probe catalog: expectation rows (fault → class → expected
      outcome) per probe id, committed before the run, referenced from
      the entry by catalog hash; harness derives probe verdicts against
      the catalog exactly as the kata path does
- [x] 4. Round-trip tests: a fixture probe ledger under
      `ledger/probes/<name>/` validates green; every task-1 rejection
      fires on the mutated ledger; `ledger-check ledger/` output is
      byte-identical before and after probe recording

## GREEN — inaugural probe 1: migration Δ at real scale

- [x] 5. Bogart-scale migration fixtures in `fixtures/probes/migration/`,
      built up from the existing `fixtures/migration/bogart_rehearsal.ct.json`
      toward real pilot-corpus schema complexity; pre-registered
      lost-data and non-invertibility expectations committed first;
      floor control fixture = a loss fully visible to row-count/checksum
      dry run (probe must show zero unique detections there)
- [x] 6. Baseline harness committed and version-pinned BEFORE the run
      (rule_lens_baseline.py pattern): statement-level migration lint
      (Atlas/Flyway-class at a pinned revision, or a frozen script
      encoding exactly its documented checks) + row-count/checksum dry
      run, over the same fault set; baseline detections frozen as
      assertions
- [x] 7. Run `catherine migrate` across the fault set via the harness;
      derive unique set mechanically; record entries in
      `ledger/probes/migration/`; verdict by the recorded procedure —
      including the honest null if the baseline matches every detection

## GREEN — inaugural probe 2: rule lens on extracted tables

- [x] 8. Corpus check first: confirm a real policy/eligibility table
      exists in the bogart/Normai corpus; if none does, record
      `inconclusive` naming the missing corpus (or select and record an
      alternative real source) — never fabricate a table to flatter the
      probe. Extract the table (prose/PDF source recorded), author
      declared input→output examples, pre-register expectations
- [x] 9. Freeze the manual-review protocol into a committed scripted
      assertion set BEFORE the run (the rule-lens 45-assertion pattern)
      — the baseline arm is executed and replayable, never a live "we
      reviewed it" note; run `catherine rules` + discovery; derive the
      unique set; record entries; scope verdict stated (extracted
      tables, not authored DMN)

## Docs scoping — after evidence lands (not before)

- [x] 10. README/USING rule-lens claims repositioned to the
      extracted-table scope with the DMN-parity citation (Calvanese et
      al. arXiv:1603.07466; Signavio/OpenRules); migration claims cite
      the probe entries; USING.md banner flips ILLUSTRATED → EXECUTED
      with captured probe runs (coordinated with `catherine-p8d`, which
      owns status-drift fixes only)

## REFACTOR

- [x] 11. Tidy: shared catalog/expectation plumbing between kata and
      probe paths where duplication appeared; contract TOMLs under
      `.espectacular/domain-probes/` complete (every task-1 stub
      populated, no `no-toml` findings remain); `ah check --changes
      add-domain-probe-validation` green; separate commit from feature
      work
