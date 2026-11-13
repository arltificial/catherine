# Tasks: add-domain-probe-validation

One red→green→refactor arc per task group (beads ticket filed with this
change). Validation runs use the committed harness path
(`catherine_ledger::harness::run`), never a bespoke re-run.

## RED — the ledger refuses probe-shaped bad science

- [ ] 1. Validator rejects, with named violations: a `probe` entry whose
      `catalog_hash` does not resolve to a committed catalog; a
      non-empty `unique_detections` without a baseline arm; a `probe`
      entry inside the kata ledger sequence; any recomputation or
      supersession of the sealed kata rollup triggered by probe
      recording — pinned by crate tests before any probe support exists

## GREEN — probe entry kind + validator support

- [ ] 2. `EntryContent` gains kind `probe`: catalog hash, pre-registered
      expectations, baseline arm (tool/practice, detections on the same
      fault set, derived unique set), run record, gaps, closed verdict
      vocabulary, content-addressed id — reusing the kata shapes where
      they already fit; `ledger-check`/`ledger-record` accept a
      `--probes` ledger root (`ledger/probes/`)
- [ ] 3. Probe catalog format: the kata catalog's expectation rows
      (fault → class → expected outcome) scoped per probe, committed
      before the run; catalog hash referenced from the entry
- [ ] 4. Round-trip tests: probe ledger validates green with a fixture
      probe; every task-1 rejection fires on the mutated ledger; kata
      ledger `ledger-check` output is byte-identical before and after
      probe recording

## GREEN — inaugural probe 1: migration Δ at real scale

- [ ] 5. Bogart-scale migration fixtures in `fixtures/probes/migration/`
      (real schema complexity from the pilot corpus, beyond the 3–4-row
      pins in `fixtures/migration/`), with pre-registered lost-data and
      non-invertibility expectations committed first
- [ ] 6. Conventional-baseline arm executed and recorded: statement-level
      migration linting (Atlas/Flyway-class) + row-count/checksum dry
      run over the same fault set; unique set derived mechanically
- [ ] 7. Probe entries recorded in `ledger/probes/`; verdict derived by
      the recorded procedure — including the honest null if the
      baseline matches every detection

## GREEN — inaugural probe 2: rule lens on extracted tables

- [ ] 8. Extract a real policy/eligibility table from the bogart/Normai
      corpus (prose/PDF source recorded), author declared input→output
      examples, pre-register expectations (gap/overlap/shadow vs manual
      review baseline)
- [ ] 9. Run `catherine rules` + discovery against the extracted table;
      baseline arm = manual review of the same table by the same fault
      list; record entries; scope verdict stated (extracted tables, not
      authored DMN)

## Docs scoping — after evidence lands (not before)

- [ ] 10. README/USING rule-lens claims repositioned to the
      extracted-table scope with the DMN-parity citation (Calvanese et
      al. arXiv:1603.07466; Signavio/OpenRules); migration claims cite
      the probe entries; USING.md banner flips ILLUSTRATED → EXECUTED
      with captured probe runs (coordinated with the docs-drift ticket)

## REFACTOR

- [ ] 11. Tidy: shared expectation/catalog types between kata and probe
      paths where duplication appeared; spectacular contract TOMLs under
      `.espectacular/domain-probes/` for every new scenario; `ah check`
      green; separate commit from feature work
