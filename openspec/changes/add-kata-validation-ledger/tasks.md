# Tasks: add-kata-validation-ledger

One red→green→refactor cycle (beads `catherine-9vv`).

## RED — the ledger refuses bad science

- [ ] 1. Ledger validator rejects: kata entry without a prior negative
      control; verdict outside the closed vocabulary; entry without
      model-hash references

## GREEN

- [ ] 2. Negative control: port FizzBuzz (kata-6) fixture; run; record
      verdict (expected: no categorical advantage) as ledger entry #1
- [ ] 3. Port Bowling (kata-1): temporal-semantics fixture; expected
      partial result recorded honestly
- [ ] 4. Port Gilded Rose: state-shape invariants as path equations +
      totality
- [ ] 5. Port Vending Machine: strongest structural result (6/6)
- [ ] 6. Browser fixtures wired into the web view (load, check, findings
      panel per kata)
- [ ] 7. Ledger entries content-addressed; append-only; supersession
      supported
- [ ] 8. Project-verdict rollup entry; gates wired: P7/P8 beads blocked
      unless `survived`
- [ ] 9. Expressibility-gap field recorded per kata (injectivity-flavored
      invariants named, not papered over)

## REFACTOR

- [ ] 10. Extract kata-fixture harness shared with P8/P9 work

## Validation

- [ ] 11. All four ledger entries present, ordered, validator green;
      `just ci` green; `openspec validate add-kata-validation-ledger
      --strict`; `bd close catherine-9vv` + fresh export
